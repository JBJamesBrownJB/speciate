//! Headless Spec Runner - Auto-discovers and runs all spec trials
//!
//! Run with: cargo test --features dev-tools --test spec_runner
//!
//! This test discovers all .toml files in specs/ and runs them headlessly,
//! evaluating assertions to determine pass/fail status.

#[cfg(feature = "dev-tools")]
mod runner {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::Instant;

    use speciate::trials::{
        Assertion, SpecConfig, TaggedEntities, TrialDirector, TrialSnapshot,
        PRODUCTION_DELTA_TIME,
    };
    use speciate::trials::loader::load_trial;
    use speciate::{BodySize, CritId, EntityTag, Position, SimulationBuilder, Target, Velocity};

    /// Discover all spec files in the specs/ directory
    fn discover_specs() -> Vec<(String, PathBuf)> {
        let mut specs = Vec::new();

        let specs_dir = PathBuf::from("specs");
        if !specs_dir.exists() {
            panic!("specs/ directory not found. Run from apps/simulation/");
        }

        // Optional category filter via SPEC_CATEGORY env var
        let category_filter = std::env::var("SPEC_CATEGORY").ok();

        // Walk through subdirectories (behavior/, physics/, performance/)
        for category_entry in fs::read_dir(&specs_dir).expect("Failed to read specs/") {
            let category_entry = category_entry.expect("Failed to read directory entry");
            let category_path = category_entry.path();

            if category_path.is_dir() {
                let category_name = category_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                // Skip if category filter is set and doesn't match
                if let Some(ref filter) = category_filter {
                    if category_name != *filter {
                        continue;
                    }
                }

                for spec_entry in fs::read_dir(&category_path).expect("Failed to read category dir") {
                    let spec_entry = spec_entry.expect("Failed to read spec entry");
                    let spec_path = spec_entry.path();

                    if spec_path.extension().map_or(false, |ext| ext == "toml") {
                        let spec_name = spec_path
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();

                        let full_name = format!("{}/{}", category_name, spec_name);
                        specs.push((full_name, spec_path));
                    }
                }
            }
        }

        specs.sort_by(|a, b| a.0.cmp(&b.0));
        specs
    }

    /// Parse a spec file into SpecConfig
    fn parse_spec(path: &PathBuf) -> Result<SpecConfig, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read spec file: {}", e))?;

        // Check if it's a spec format (has [meta]) or legacy trial format
        if content.contains("[meta]") {
            toml::from_str(&content)
                .map_err(|e| format!("Failed to parse spec TOML: {}", e))
        } else {
            // Legacy format - convert to spec format with no assertions
            let trial: speciate::trials::TrialConfig = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse trial TOML: {}", e))?;

            Ok(SpecConfig {
                meta: speciate::trials::MetaConfig {
                    name: trial.name,
                    description: trial.description,
                    timeout_seconds: 50.0,
                    seed: None,
                },
                variants: HashMap::new(),
                assertions: vec![Assertion::TicksCompleted { count: 100 }], // Basic sanity check
                spawns: trial.spawns,
            })
        }
    }

    /// Check if spec has any overlap-related assertions
    fn needs_overlap_detection(spec: &SpecConfig) -> bool {
        spec.assertions.iter().any(|a| {
            matches!(
                a,
                Assertion::NoOverlaps
                    | Assertion::MaxOverlaps { .. }
                    | Assertion::MaxOverlapDepth { .. }
                    | Assertion::MaxTicksWithOverlaps { .. }
            )
        })
    }

    /// Check if spec needs per-tick world queries (overlap or target assertions)
    /// Latency assertions only need timing, not creature data
    fn needs_per_tick_world_queries(spec: &SpecConfig) -> bool {
        needs_overlap_detection(spec) || needs_tag_tracking(spec)
    }

    /// Check if spec has any tag-based assertions
    fn needs_tag_tracking(spec: &SpecConfig) -> bool {
        spec.assertions.iter().any(|a| {
            matches!(a, Assertion::CreatureReachedTarget { .. })
        })
    }

    /// Create a minimal snapshot (just tick count and timing, no world queries)
    fn create_minimal_snapshot(ticks_run: u32, tick_duration_us: u64) -> TrialSnapshot {
        TrialSnapshot {
            ticks_run,
            creature_count: 0,
            tagged_entities: TaggedEntities::default(),
            overlaps: Vec::new(),
            tick_duration_us,
        }
    }

    /// Create a snapshot of the simulation world for assertion evaluation
    fn create_snapshot(
        sim: &mut speciate::Simulation,
        ticks_run: u32,
        spec: &SpecConfig,
        tick_duration_us: u64,
    ) -> TrialSnapshot {
        let world = sim.world_mut();

        // Count creatures
        let mut creature_query = world.query::<(&CritId, &Position, &Velocity, &BodySize)>();
        let creatures: Vec<_> = creature_query.iter(world).collect();
        let creature_count = creatures.len();

        // Only compute O(n²) overlap detection if spec has overlap assertions
        let overlaps = if needs_overlap_detection(spec) {
            let mut overlaps = Vec::new();
            for i in 0..creatures.len() {
                for j in (i + 1)..creatures.len() {
                    let (_, pos_a, _, size_a) = creatures[i];
                    let (_, pos_b, _, size_b) = creatures[j];

                    let dx = pos_a.x - pos_b.x;
                    let dy = pos_a.y - pos_b.y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let min_distance = size_a.radius() + size_b.radius();

                    if distance < min_distance {
                        let overlap_depth = min_distance - distance;
                        overlaps.push((i, j, overlap_depth));
                    }
                }
            }
            overlaps
        } else {
            Vec::new()
        };

        // Collect tagged entities (only if spec needs tag tracking)
        let tagged_entities = if needs_tag_tracking(spec) {
            let mut tagged = TaggedEntities::default();

            // Query all creatures with tags
            let mut tagged_query = world.query::<(&EntityTag, &Position, &Target)>();
            for (tag, pos, target) in tagged_query.iter(world) {
                let tag_str = &tag.0;

                // Record position
                tagged
                    .positions
                    .entry(tag_str.clone())
                    .or_insert_with(Vec::new)
                    .push((pos.x, pos.y));

                // Record target
                tagged
                    .targets
                    .entry(tag_str.clone())
                    .or_insert_with(Vec::new)
                    .push((target.x, target.y));
            }

            tagged
        } else {
            TaggedEntities::default()
        };

        TrialSnapshot {
            ticks_run,
            creature_count,
            tagged_entities,
            overlaps,
            tick_duration_us,
        }
    }

    /// Run a single spec and return pass/fail status
    fn run_spec(name: &str, spec: &SpecConfig) -> Result<(), String> {
        // Always use production delta_time (0.05s at 20Hz) for consistent physics
        let delta_time = PRODUCTION_DELTA_TIME;

        // Build simulation with 10km × 10km world bounds
        let mut sim = SimulationBuilder::new()
            .set_boundaries(10000.0, 10000.0)
            .build();

        // Load spawns into world
        {
            let world = sim.world_mut();

            // Try loading the spec via loader first (it handles spawns)
            if let Err(e) = load_trial(world, name) {
                return Err(format!("Failed to load spec: {}", e));
            }
        }

        // Create director
        let mut director = TrialDirector::new();
        director.start_trial(spec.clone());

        // Check if we need expensive per-tick world queries (only for overlap assertions)
        let needs_per_tick = needs_per_tick_world_queries(spec);

        // Run until wall-clock timeout (timeout_seconds is REAL time, not simulation time)
        let timeout_duration = std::time::Duration::from_secs_f32(spec.meta.timeout_seconds);
        let trial_start = Instant::now();
        let mut ticks_run = 0;

        while trial_start.elapsed() < timeout_duration {
            // Measure tick duration
            let tick_start = Instant::now();
            sim.update(delta_time);
            let tick_duration_us = tick_start.elapsed().as_micros() as u64;

            ticks_run += 1;

            // Use minimal snapshot if we don't need per-tick world queries
            let snapshot = if needs_per_tick {
                create_snapshot(&mut sim, ticks_run, spec, tick_duration_us)
            } else {
                create_minimal_snapshot(ticks_run, tick_duration_us)
            };
            director.on_tick(&snapshot);
        }

        // Always create full snapshot for final assertions
        let final_snapshot = create_snapshot(&mut sim, ticks_run, spec, 0);
        director.complete_trial(&final_snapshot);

        // Check result
        match director.result() {
            Some(result) if result.passed => Ok(()),
            Some(result) => {
                let mut msg = format!("Spec '{}' failed after {} ticks:\n", name, result.ticks_run);
                for assertion_result in &result.assertion_results {
                    let status = if assertion_result.passed { "PASS" } else { "FAIL" };
                    msg.push_str(&format!("  [{}] {}\n", status, assertion_result.message));
                }
                Err(msg)
            }
            None => Err(format!("No result available for spec '{}'", name)),
        }
    }

    // ========================================================================
    // Test: Run all discovered specs
    // ========================================================================

    #[test]
    fn run_all_specs() {
        let specs = discover_specs();

        if specs.is_empty() {
            panic!("No specs found in specs/ directory");
        }

        println!("\nDiscovered {} specs:", specs.len());
        for (name, _) in &specs {
            println!("  - {}", name);
        }
        println!();

        let mut passed = 0;
        let mut failed = 0;
        let mut errors: Vec<String> = Vec::new();

        for (name, path) in &specs {
            print!("Running spec '{}'... ", name);

            match parse_spec(path) {
                Ok(spec) => {
                    match run_spec(name, &spec) {
                        Ok(()) => {
                            println!("PASS");
                            passed += 1;
                        }
                        Err(e) => {
                            println!("FAIL");
                            errors.push(e);
                            failed += 1;
                        }
                    }
                }
                Err(e) => {
                    println!("ERROR: {}", e);
                    errors.push(format!("Spec '{}': {}", name, e));
                    failed += 1;
                }
            }
        }

        println!("\n========================================");
        println!("Results: {} passed, {} failed", passed, failed);
        println!("========================================\n");

        if !errors.is_empty() {
            println!("Failures:");
            for error in &errors {
                println!("{}", error);
            }
            panic!("{} spec(s) failed", failed);
        }
    }

    // ========================================================================
    // Individual spec tests (for debugging specific specs)
    // ========================================================================

    #[test]
    fn spec_catatonic_crowd_creature_count() {
        let spec_path = PathBuf::from("specs/behavior/catatonic-crowd.toml");
        if !spec_path.exists() {
            println!("Skipping: spec file not found");
            return;
        }

        let _spec = parse_spec(&spec_path).expect("Failed to parse spec");

        // Run just the creature count assertion
        let mut sim = SimulationBuilder::new()
            .set_boundaries(100.0, 100.0)
            .build();

        {
            let world = sim.world_mut();
            load_trial(world, "behavior/catatonic-crowd").expect("Failed to load trial");
        }

        // Count creatures
        let world = sim.world_mut();
        let count = world.query::<&CritId>().iter(world).count();

        // 50x50 grid = 2500 creatures
        assert_eq!(count, 2500, "Expected 2500 creatures, found {}", count);
    }

    #[test]
    fn spec_many_wanderers_dense_creature_count() {
        let spec_path = PathBuf::from("specs/performance/many-wanderers-dense.toml");
        if !spec_path.exists() {
            println!("Skipping: spec file not found");
            return;
        }

        let mut sim = SimulationBuilder::new()
            .set_boundaries(10000.0, 10000.0)
            .build();

        {
            let world = sim.world_mut();
            load_trial(world, "performance/many-wanderers-dense").expect("Failed to load trial");
        }

        // Count creatures
        let world = sim.world_mut();
        let count = world.query::<&CritId>().iter(world).count();

        // 200x250 grid = 50,000 creatures
        assert_eq!(count, 50_000, "Expected 50,000 creatures, found {}", count);
    }

    /// Focused test for 200k creature performance spec with detailed timing
    #[test]
    fn spec_200k_world_spread_performance() {
        let spec_path = PathBuf::from("specs/performance/many-wanderers-world-spread.toml");
        if !spec_path.exists() {
            println!("Skipping: spec file not found");
            return;
        }

        let spec = parse_spec(&spec_path).expect("Failed to parse spec");
        println!("\n=== 200K Performance Test ===");
        println!("Spec: {}", spec.meta.name);
        println!("Timeout: {}s (wall-clock)", spec.meta.timeout_seconds);

        // Time spawn (10km × 10km world)
        let spawn_start = Instant::now();
        let mut sim = SimulationBuilder::new()
            .set_boundaries(10000.0, 10000.0)
            .build();

        {
            let world = sim.world_mut();
            load_trial(world, "performance/many-wanderers-world-spread").expect("Failed to load trial");
        }
        let spawn_time = spawn_start.elapsed();
        println!("Spawn time: {:?}", spawn_time);

        // Count creatures
        let creature_count = {
            let world = sim.world_mut();
            world.query::<&CritId>().iter(world).count()
        };
        println!("Creatures spawned: {}", creature_count);

        if creature_count != 200_000 {
            panic!("Expected 200,000 creatures, got {}", creature_count);
        }

        // Run simulation and measure tick times
        let delta_time = PRODUCTION_DELTA_TIME;
        let num_ticks = 10; // Just run 10 ticks for quick feedback
        let mut tick_times = Vec::with_capacity(num_ticks);

        println!("\nRunning {} ticks...", num_ticks);
        for i in 0..num_ticks {
            let tick_start = Instant::now();
            sim.update(delta_time);
            let tick_time = tick_start.elapsed();
            tick_times.push(tick_time);
            println!("  Tick {}: {:?}", i + 1, tick_time);
        }

        let total_time: std::time::Duration = tick_times.iter().sum();
        let avg_tick = total_time / num_ticks as u32;
        println!("\nResults:");
        println!("  Total time: {:?}", total_time);
        println!("  Avg tick: {:?}", avg_tick);
        println!("  Target: 35ms");

        let avg_ms = avg_tick.as_millis();
        if avg_ms > 35 {
            println!("  FAIL: Avg tick {}ms exceeds 35ms target", avg_ms);
        } else {
            println!("  PASS: Avg tick {}ms within 35ms target", avg_ms);
        }
    }

    /// Focused test for 200k seeker performance spec with detailed timing
    #[test]
    fn spec_200k_seekers_performance() {
        let spec_path = PathBuf::from("specs/performance/many-seekers-world-spread.toml");
        if !spec_path.exists() {
            println!("Skipping: spec file not found");
            return;
        }

        let spec = parse_spec(&spec_path).expect("Failed to parse spec");
        println!("\n=== 200K Seekers Performance Test ===");
        println!("Spec: {}", spec.meta.name);
        println!("Timeout: {}s (wall-clock)", spec.meta.timeout_seconds);

        // Time spawn (10km × 10km world)
        let spawn_start = Instant::now();
        let mut sim = SimulationBuilder::new()
            .set_boundaries(10000.0, 10000.0)
            .build();

        {
            let world = sim.world_mut();
            load_trial(world, "performance/many-seekers-world-spread").expect("Failed to load trial");
        }
        let spawn_time = spawn_start.elapsed();
        println!("Spawn time: {:?}", spawn_time);

        // Count creatures
        let creature_count = {
            let world = sim.world_mut();
            world.query::<&CritId>().iter(world).count()
        };
        println!("Creatures spawned: {}", creature_count);

        if creature_count != 200_000 {
            panic!("Expected 200,000 creatures, got {}", creature_count);
        }

        // Run simulation and measure tick times
        let delta_time = PRODUCTION_DELTA_TIME;
        let num_ticks = 10;
        let mut tick_times = Vec::with_capacity(num_ticks);

        println!("\nRunning {} ticks...", num_ticks);
        for i in 0..num_ticks {
            let tick_start = Instant::now();
            sim.update(delta_time);
            let tick_time = tick_start.elapsed();
            tick_times.push(tick_time);
            println!("  Tick {}: {:?}", i + 1, tick_time);
        }

        let total_time: std::time::Duration = tick_times.iter().sum();
        let avg_tick = total_time / num_ticks as u32;
        println!("\nResults:");
        println!("  Total time: {:?}", total_time);
        println!("  Avg tick: {:?}", avg_tick);
        println!("  Target: 35ms");

        let avg_ms = avg_tick.as_millis();
        if avg_ms > 35 {
            println!("  STATUS: Avg tick {}ms exceeds 35ms target", avg_ms);
        } else {
            println!("  STATUS: Avg tick {}ms within 35ms target", avg_ms);
        }
    }
}

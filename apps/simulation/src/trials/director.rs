use std::collections::HashMap;

use super::{Assertion, SpecConfig};

const LATENCY_WARMUP_TICKS: u32 = 100;

/// Trial execution state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialState {
    Idle,
    Running,
    Completed,
    Failed,
}

/// Result of assertion evaluation
#[derive(Debug, Clone)]
pub struct AssertionResult {
    pub assertion: Assertion,
    pub passed: bool,
    pub message: String,
}

/// Result of a trial run
#[derive(Debug, Clone)]
pub struct TrialResult {
    pub spec_name: String,
    pub state: TrialState,
    pub ticks_run: u32,
    pub assertion_results: Vec<AssertionResult>,
    pub passed: bool,
}

/// Tracks entity tags for assertion evaluation
#[derive(Debug, Default, Clone)]
pub struct TaggedEntities {
    /// Maps tag -> list of entity positions (x, y)
    pub positions: HashMap<String, Vec<(f32, f32)>>,
    /// Maps tag -> list of target positions (x, y)
    pub targets: HashMap<String, Vec<(f32, f32)>>,
}

/// Overlap data: (entity_a_idx, entity_b_idx, penetration_depth)
pub type OverlapData = (usize, usize, f32);

/// Tracks trial state for assertion evaluation
#[derive(Debug, Clone)]
pub struct TrialSnapshot {
    pub ticks_run: u32,
    pub creature_count: usize,
    pub tagged_entities: TaggedEntities,
    pub overlaps: Vec<OverlapData>,
    pub tick_duration_us: u64,
}

/// TrialDirector manages spec trial lifecycle and assertion evaluation
#[derive(Debug)]
pub struct TrialDirector {
    state: TrialState,
    spec: Option<SpecConfig>,
    ticks_run: u32,
    ticks_with_overlaps: u32,
    max_overlap_count: usize,
    max_overlap_depth: f32,
    total_tick_time_us: u64,
    latency_ticks_run: u32,
    assertion_results: Vec<AssertionResult>,
}

impl Default for TrialDirector {
    fn default() -> Self {
        Self::new()
    }
}

impl TrialDirector {
    pub fn new() -> Self {
        Self {
            state: TrialState::Idle,
            spec: None,
            ticks_run: 0,
            ticks_with_overlaps: 0,
            max_overlap_count: 0,
            max_overlap_depth: 0.0,
            total_tick_time_us: 0,
            latency_ticks_run: 0,
            assertion_results: Vec::new(),
        }
    }

    pub fn state(&self) -> TrialState {
        self.state
    }

    pub fn ticks_run(&self) -> u32 {
        self.ticks_run
    }

    /// Start a new trial with the given spec
    pub fn start_trial(&mut self, spec: SpecConfig) {
        self.state = TrialState::Running;
        self.spec = Some(spec);
        self.ticks_run = 0;
        self.ticks_with_overlaps = 0;
        self.max_overlap_count = 0;
        self.max_overlap_depth = 0.0;
        self.total_tick_time_us = 0;
        self.latency_ticks_run = 0;
        self.assertion_results.clear();
    }

    /// Process one tick of the trial - tracks timing and overlap stats
    /// Does NOT auto-complete; runner must call complete_trial() with full snapshot
    pub fn on_tick(&mut self, snapshot: &TrialSnapshot) {
        if self.state != TrialState::Running {
            return;
        }

        self.ticks_run += 1;

        // Track tick timing (only after warm-up period to exclude spawn spikes)
        if self.ticks_run > LATENCY_WARMUP_TICKS {
            self.total_tick_time_us += snapshot.tick_duration_us;
            self.latency_ticks_run += 1;
        }

        // Track overlap statistics
        if !snapshot.overlaps.is_empty() {
            self.ticks_with_overlaps += 1;
        }
        if snapshot.overlaps.len() > self.max_overlap_count {
            self.max_overlap_count = snapshot.overlaps.len();
        }
        let tick_max_depth = snapshot
            .overlaps
            .iter()
            .map(|(_, _, depth)| *depth)
            .fold(0.0f32, f32::max);
        if tick_max_depth > self.max_overlap_depth {
            self.max_overlap_depth = tick_max_depth;
        }
    }

    /// Complete the trial and evaluate all assertions
    pub fn complete_trial(&mut self, snapshot: &TrialSnapshot) {
        if self.state != TrialState::Running {
            return;
        }

        let spec = match &self.spec {
            Some(s) => s,
            None => {
                self.state = TrialState::Failed;
                return;
            }
        };

        // Evaluate all assertions
        self.assertion_results = spec
            .assertions
            .iter()
            .map(|assertion| self.evaluate_assertion(assertion, snapshot))
            .collect();

        // Trial passes if all assertions pass
        let all_passed = self.assertion_results.iter().all(|r| r.passed);
        self.state = if all_passed {
            TrialState::Completed
        } else {
            TrialState::Failed
        };
    }

    /// Get the result of the trial
    pub fn result(&self) -> Option<TrialResult> {
        let spec = self.spec.as_ref()?;

        Some(TrialResult {
            spec_name: spec.meta.name.clone(),
            state: self.state,
            ticks_run: self.ticks_run,
            assertion_results: self.assertion_results.clone(),
            passed: self.state == TrialState::Completed,
        })
    }

    /// Evaluate a single assertion against the trial snapshot
    fn evaluate_assertion(&self, assertion: &Assertion, snapshot: &TrialSnapshot) -> AssertionResult {
        match assertion {
            Assertion::NoOverlaps => {
                let passed = self.max_overlap_count == 0;
                AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    message: if passed {
                        "No overlaps detected".to_string()
                    } else {
                        format!(
                            "Max {} overlaps detected (in {} ticks with overlaps)",
                            self.max_overlap_count, self.ticks_with_overlaps
                        )
                    },
                }
            }

            Assertion::MaxOverlaps { count } => {
                let passed = self.max_overlap_count <= *count;
                AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    message: if passed {
                        format!(
                            "Max overlaps per tick: {} (limit: {})",
                            self.max_overlap_count, count
                        )
                    } else {
                        format!(
                            "Max overlaps per tick: {} exceeds limit of {}",
                            self.max_overlap_count, count
                        )
                    },
                }
            }

            Assertion::MaxOverlapDepth { depth } => {
                let passed = self.max_overlap_depth <= *depth;
                AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    message: if passed {
                        format!(
                            "Max overlap depth: {:.2} (limit: {:.2})",
                            self.max_overlap_depth, depth
                        )
                    } else {
                        format!(
                            "Max overlap depth: {:.2} exceeds limit of {:.2}",
                            self.max_overlap_depth, depth
                        )
                    },
                }
            }

            Assertion::MaxTicksWithOverlaps { count } => {
                let passed = self.ticks_with_overlaps <= *count;
                AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    message: if passed {
                        format!(
                            "Ticks with overlaps: {} (limit: {})",
                            self.ticks_with_overlaps, count
                        )
                    } else {
                        format!(
                            "Ticks with overlaps: {} exceeds limit of {}",
                            self.ticks_with_overlaps, count
                        )
                    },
                }
            }

            Assertion::CreatureReachedTarget { tag } => {
                let positions = snapshot.tagged_entities.positions.get(tag);
                let targets = snapshot.tagged_entities.targets.get(tag);

                match (positions, targets) {
                    (Some(positions), Some(targets)) if !positions.is_empty() && !targets.is_empty() => {
                        // Check if any tagged creature reached its target
                        // Threshold accounts for creature radius (~0.5m) + target radius (~1.0m)
                        // Creature "arrives" when its edge touches the target's arrival zone
                        let arrival_threshold = 1.5; // center distance when edges touch
                        let reached = positions.iter().zip(targets.iter()).any(|(pos, target)| {
                            let dx = pos.0 - target.0;
                            let dy = pos.1 - target.1;
                            let distance = (dx * dx + dy * dy).sqrt();
                            distance <= arrival_threshold
                        });

                        AssertionResult {
                            assertion: assertion.clone(),
                            passed: reached,
                            message: if reached {
                                format!("Tagged creature '{}' reached target", tag)
                            } else {
                                format!("Tagged creature '{}' did not reach target", tag)
                            },
                        }
                    }
                    _ => AssertionResult {
                        assertion: assertion.clone(),
                        passed: false,
                        message: format!("No tagged creature '{}' found or no target set", tag),
                    },
                }
            }

            Assertion::CreatureCount { min, max } => {
                let count = snapshot.creature_count;
                let passed = count >= *min && count <= *max;

                AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    message: if passed {
                        format!("Creature count {} is within [{}, {}]", count, min, max)
                    } else {
                        format!("Creature count {} is outside [{}, {}]", count, min, max)
                    },
                }
            }

            Assertion::TicksCompleted { count } => {
                let passed = snapshot.ticks_run >= *count;

                AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    message: if passed {
                        format!("Completed {} ticks (required: {})", snapshot.ticks_run, count)
                    } else {
                        format!("Only {} ticks completed (required: {})", snapshot.ticks_run, count)
                    },
                }
            }

            Assertion::MaxAvgTickLatency { microseconds } => {
                // Use latency_ticks_run (excludes first 100 warm-up ticks)
                let avg_tick_us = if self.latency_ticks_run > 0 {
                    self.total_tick_time_us / self.latency_ticks_run as u64
                } else {
                    0
                };
                let passed = avg_tick_us <= *microseconds;

                AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    message: if passed {
                        format!(
                            "Avg tick latency: {}us (limit: {}us, {} ticks after warm-up)",
                            avg_tick_us, microseconds, self.latency_ticks_run
                        )
                    } else {
                        format!(
                            "Avg tick latency: {}us exceeds limit of {}us ({} ticks after warm-up)",
                            avg_tick_us, microseconds, self.latency_ticks_run
                        )
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trials::MetaConfig;

    fn minimal_spec(name: &str) -> SpecConfig {
        SpecConfig {
            meta: MetaConfig {
                name: name.to_string(),
                description: String::new(),
                timeout_seconds: 50.0,
                seed: None,
            },
            variants: HashMap::new(),
            assertions: Vec::new(),
            spawns: Vec::new(),
        }
    }

    fn empty_snapshot(ticks: u32) -> TrialSnapshot {
        TrialSnapshot {
            ticks_run: ticks,
            creature_count: 0,
            tagged_entities: TaggedEntities::default(),
            overlaps: Vec::new(),
            tick_duration_us: 0,
        }
    }

    // ========================================================================
    // State Machine Tests
    // ========================================================================

    #[test]
    fn test_director_starts_idle() {
        let director = TrialDirector::new();
        assert_eq!(director.state(), TrialState::Idle);
    }

    #[test]
    fn test_start_trial_transitions_to_running() {
        let mut director = TrialDirector::new();
        director.start_trial(minimal_spec("Test"));
        assert_eq!(director.state(), TrialState::Running);
    }

    #[test]
    fn test_complete_trial_transitions_to_completed() {
        let mut director = TrialDirector::new();
        director.start_trial(minimal_spec("Test"));

        let snapshot = empty_snapshot(100);
        director.complete_trial(&snapshot);

        assert_eq!(director.state(), TrialState::Completed);
    }

    #[test]
    fn test_on_tick_increments_ticks_run() {
        let mut director = TrialDirector::new();
        director.start_trial(minimal_spec("Test"));

        assert_eq!(director.ticks_run(), 0);

        let snapshot = empty_snapshot(0);
        director.on_tick(&snapshot);
        assert_eq!(director.ticks_run(), 1);

        director.on_tick(&snapshot);
        assert_eq!(director.ticks_run(), 2);
    }

    // ========================================================================
    // Assertion Tests
    // ========================================================================

    #[test]
    fn test_assertion_no_overlaps_passes() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::NoOverlaps);
        director.start_trial(spec);

        let snapshot = TrialSnapshot {
            ticks_run: 100,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: Vec::new(), // No overlaps
            tick_duration_us: 0,
        };

        director.complete_trial(&snapshot);

        assert_eq!(director.state(), TrialState::Completed);
        let result = director.result().unwrap();
        assert!(result.passed);
        assert!(result.assertion_results[0].passed);
    }

    #[test]
    fn test_assertion_no_overlaps_fails() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::NoOverlaps);
        director.start_trial(spec);

        // Simulate ticks with overlaps
        let snapshot_with_overlaps = TrialSnapshot {
            ticks_run: 1,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: vec![(0, 1, 0.5), (2, 3, 0.3)], // Has overlaps with depth
            tick_duration_us: 0,
        };
        director.on_tick(&snapshot_with_overlaps);

        let final_snapshot = TrialSnapshot {
            ticks_run: 100,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: Vec::new(),
            tick_duration_us: 0,
        };
        director.complete_trial(&final_snapshot);

        assert_eq!(director.state(), TrialState::Failed);
        let result = director.result().unwrap();
        assert!(!result.passed);
        assert!(!result.assertion_results[0].passed);
    }

    #[test]
    fn test_assertion_creature_count_passes() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::CreatureCount { min: 5, max: 15 });
        director.start_trial(spec);

        let snapshot = TrialSnapshot {
            ticks_run: 100,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: Vec::new(),
            tick_duration_us: 0,
        };

        director.complete_trial(&snapshot);

        assert_eq!(director.state(), TrialState::Completed);
        let result = director.result().unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_assertion_creature_count_fails_below_min() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::CreatureCount { min: 5, max: 15 });
        director.start_trial(spec);

        let snapshot = TrialSnapshot {
            ticks_run: 100,
            creature_count: 2, // Below min
            tagged_entities: TaggedEntities::default(),
            overlaps: Vec::new(),
            tick_duration_us: 0,
        };

        director.complete_trial(&snapshot);

        assert_eq!(director.state(), TrialState::Failed);
    }

    #[test]
    fn test_assertion_ticks_completed_passes() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::TicksCompleted { count: 100 });
        director.start_trial(spec);

        let snapshot = TrialSnapshot {
            ticks_run: 150,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: Vec::new(),
            tick_duration_us: 0,
        };

        director.complete_trial(&snapshot);

        assert_eq!(director.state(), TrialState::Completed);
    }

    #[test]
    fn test_assertion_ticks_completed_fails() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::TicksCompleted { count: 100 });
        director.start_trial(spec);

        let snapshot = TrialSnapshot {
            ticks_run: 50, // Not enough ticks
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: Vec::new(),
            tick_duration_us: 0,
        };

        director.complete_trial(&snapshot);

        assert_eq!(director.state(), TrialState::Failed);
    }

    #[test]
    fn test_assertion_creature_reached_target_passes() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::CreatureReachedTarget {
            tag: "seeker".to_string(),
        });
        director.start_trial(spec);

        let mut tagged = TaggedEntities::default();
        tagged.positions.insert("seeker".to_string(), vec![(10.0, 10.0)]);
        tagged.targets.insert("seeker".to_string(), vec![(10.5, 10.0)]); // Within threshold

        let snapshot = TrialSnapshot {
            ticks_run: 100,
            creature_count: 1,
            tagged_entities: tagged,
            overlaps: Vec::new(),
            tick_duration_us: 0,
        };

        director.complete_trial(&snapshot);

        assert_eq!(director.state(), TrialState::Completed);
    }

    #[test]
    fn test_assertion_creature_reached_target_fails() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::CreatureReachedTarget {
            tag: "seeker".to_string(),
        });
        director.start_trial(spec);

        let mut tagged = TaggedEntities::default();
        tagged.positions.insert("seeker".to_string(), vec![(0.0, 0.0)]);
        tagged.targets.insert("seeker".to_string(), vec![(100.0, 100.0)]); // Far from target

        let snapshot = TrialSnapshot {
            ticks_run: 100,
            creature_count: 1,
            tagged_entities: tagged,
            overlaps: Vec::new(),
            tick_duration_us: 0,
        };

        director.complete_trial(&snapshot);

        assert_eq!(director.state(), TrialState::Failed);
    }

    #[test]
    fn test_multiple_assertions_all_must_pass() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::NoOverlaps);
        spec.assertions.push(Assertion::CreatureCount { min: 5, max: 15 });
        spec.assertions.push(Assertion::TicksCompleted { count: 100 });
        director.start_trial(spec);

        // All assertions pass
        let snapshot = TrialSnapshot {
            ticks_run: 150,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: Vec::new(),
            tick_duration_us: 0,
        };

        director.complete_trial(&snapshot);

        assert_eq!(director.state(), TrialState::Completed);
        let result = director.result().unwrap();
        assert!(result.passed);
        assert_eq!(result.assertion_results.len(), 3);
        assert!(result.assertion_results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_one_failing_assertion_fails_trial() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::NoOverlaps);
        spec.assertions.push(Assertion::CreatureCount { min: 5, max: 15 });
        director.start_trial(spec);

        // One assertion fails
        let snapshot = TrialSnapshot {
            ticks_run: 100,
            creature_count: 2, // Below min - FAILS
            tagged_entities: TaggedEntities::default(),
            overlaps: Vec::new(), // No overlaps - PASSES
            tick_duration_us: 0,
        };

        director.complete_trial(&snapshot);

        assert_eq!(director.state(), TrialState::Failed);
        let result = director.result().unwrap();
        assert!(!result.passed);

        // First passes, second fails
        assert!(result.assertion_results[0].passed);
        assert!(!result.assertion_results[1].passed);
    }

    // ========================================================================
    // Result Tests
    // ========================================================================

    #[test]
    fn test_result_contains_spec_name() {
        let mut director = TrialDirector::new();
        director.start_trial(minimal_spec("My Test Spec"));

        let snapshot = empty_snapshot(100);
        director.complete_trial(&snapshot);

        let result = director.result().unwrap();
        assert_eq!(result.spec_name, "My Test Spec");
    }

    #[test]
    fn test_result_contains_ticks_run() {
        let mut director = TrialDirector::new();
        director.start_trial(minimal_spec("Test"));

        let snapshot = empty_snapshot(0);
        for _ in 0..42 {
            director.on_tick(&snapshot);
        }

        director.complete_trial(&snapshot);

        let result = director.result().unwrap();
        assert_eq!(result.ticks_run, 42);
    }

    // ========================================================================
    // New Overlap Assertion Tests
    // ========================================================================

    #[test]
    fn test_assertion_max_overlaps_passes() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::MaxOverlaps { count: 5 });
        director.start_trial(spec);

        // Simulate ticks with some overlaps (but within limit)
        let snapshot = TrialSnapshot {
            ticks_run: 1,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: vec![(0, 1, 0.5), (2, 3, 0.3), (4, 5, 0.2)], // 3 overlaps
            tick_duration_us: 0,
        };
        director.on_tick(&snapshot);

        let final_snapshot = empty_snapshot(100);
        director.complete_trial(&final_snapshot);

        assert_eq!(director.state(), TrialState::Completed);
        let result = director.result().unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_assertion_max_overlaps_fails() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::MaxOverlaps { count: 2 });
        director.start_trial(spec);

        // Simulate tick with too many overlaps
        let snapshot = TrialSnapshot {
            ticks_run: 1,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: vec![(0, 1, 0.5), (2, 3, 0.3), (4, 5, 0.2)], // 3 overlaps > limit of 2
            tick_duration_us: 0,
        };
        director.on_tick(&snapshot);

        let final_snapshot = empty_snapshot(100);
        director.complete_trial(&final_snapshot);

        assert_eq!(director.state(), TrialState::Failed);
        let result = director.result().unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_assertion_max_overlap_depth_passes() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::MaxOverlapDepth { depth: 1.0 });
        director.start_trial(spec);

        // Simulate tick with overlaps within depth limit
        let snapshot = TrialSnapshot {
            ticks_run: 1,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: vec![(0, 1, 0.5), (2, 3, 0.8)], // max depth 0.8 < 1.0
            tick_duration_us: 0,
        };
        director.on_tick(&snapshot);

        let final_snapshot = empty_snapshot(100);
        director.complete_trial(&final_snapshot);

        assert_eq!(director.state(), TrialState::Completed);
        let result = director.result().unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_assertion_max_overlap_depth_fails() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::MaxOverlapDepth { depth: 0.5 });
        director.start_trial(spec);

        // Simulate tick with overlap exceeding depth limit
        let snapshot = TrialSnapshot {
            ticks_run: 1,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: vec![(0, 1, 0.3), (2, 3, 0.9)], // max depth 0.9 > 0.5
            tick_duration_us: 0,
        };
        director.on_tick(&snapshot);

        let final_snapshot = empty_snapshot(100);
        director.complete_trial(&final_snapshot);

        assert_eq!(director.state(), TrialState::Failed);
        let result = director.result().unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_assertion_max_ticks_with_overlaps_passes() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::MaxTicksWithOverlaps { count: 10 });
        director.start_trial(spec);

        // Simulate 5 ticks with overlaps (within limit)
        let snapshot_with_overlaps = TrialSnapshot {
            ticks_run: 1,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: vec![(0, 1, 0.5)],
            tick_duration_us: 0,
        };
        for _ in 0..5 {
            director.on_tick(&snapshot_with_overlaps);
        }

        // Then 5 ticks without overlaps
        let snapshot_no_overlaps = empty_snapshot(1);
        for _ in 0..5 {
            director.on_tick(&snapshot_no_overlaps);
        }

        let final_snapshot = empty_snapshot(100);
        director.complete_trial(&final_snapshot);

        assert_eq!(director.state(), TrialState::Completed);
        let result = director.result().unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_assertion_max_ticks_with_overlaps_fails() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::MaxTicksWithOverlaps { count: 3 });
        director.start_trial(spec);

        // Simulate 5 ticks with overlaps (exceeds limit of 3)
        let snapshot_with_overlaps = TrialSnapshot {
            ticks_run: 1,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: vec![(0, 1, 0.5)],
            tick_duration_us: 0,
        };
        for _ in 0..5 {
            director.on_tick(&snapshot_with_overlaps);
        }

        let final_snapshot = empty_snapshot(100);
        director.complete_trial(&final_snapshot);

        assert_eq!(director.state(), TrialState::Failed);
        let result = director.result().unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_overlap_tracking_across_multiple_ticks() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::MaxOverlaps { count: 5 });
        spec.assertions.push(Assertion::MaxOverlapDepth { depth: 2.0 });
        spec.assertions.push(Assertion::MaxTicksWithOverlaps { count: 10 });
        director.start_trial(spec);

        // Tick 1: 2 overlaps, max depth 0.5
        let snapshot1 = TrialSnapshot {
            ticks_run: 1,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: vec![(0, 1, 0.3), (2, 3, 0.5)],
            tick_duration_us: 0,
        };
        director.on_tick(&snapshot1);

        // Tick 2: 4 overlaps, max depth 1.5
        let snapshot2 = TrialSnapshot {
            ticks_run: 2,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: vec![(0, 1, 0.2), (2, 3, 1.5), (4, 5, 0.1), (6, 7, 0.8)],
            tick_duration_us: 0,
        };
        director.on_tick(&snapshot2);

        // Tick 3: no overlaps
        let snapshot3 = empty_snapshot(3);
        director.on_tick(&snapshot3);

        let final_snapshot = empty_snapshot(100);
        director.complete_trial(&final_snapshot);

        assert_eq!(director.state(), TrialState::Completed);
        let result = director.result().unwrap();
        assert!(result.passed);
        // Verify: max overlaps = 4, max depth = 1.5, ticks with overlaps = 2
    }

    // ========================================================================
    // Performance Assertion Tests
    // ========================================================================

    #[test]
    fn test_assertion_max_avg_tick_latency_passes() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::MaxAvgTickLatency { microseconds: 1000 }); // 1ms limit
        director.start_trial(spec);

        // Run 100 warm-up ticks (excluded from latency calculation)
        for _ in 0..100 {
            director.on_tick(&empty_snapshot(0));
        }

        // Simulate 10 ticks after warm-up, each taking 500us (avg = 500us < 1000us limit)
        for _ in 0..10 {
            let snapshot = TrialSnapshot {
                ticks_run: 1,
                creature_count: 10,
                tagged_entities: TaggedEntities::default(),
                overlaps: Vec::new(),
                tick_duration_us: 500,
            };
            director.on_tick(&snapshot);
        }

        let final_snapshot = empty_snapshot(110);
        director.complete_trial(&final_snapshot);

        assert_eq!(director.state(), TrialState::Completed);
        let result = director.result().unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_assertion_max_avg_tick_latency_fails() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::MaxAvgTickLatency { microseconds: 500 }); // 500us limit
        director.start_trial(spec);

        // Run 100 warm-up ticks (excluded from latency calculation)
        for _ in 0..100 {
            director.on_tick(&empty_snapshot(0));
        }

        // Simulate 10 ticks after warm-up, each taking 1000us (avg = 1000us > 500us limit)
        for _ in 0..10 {
            let snapshot = TrialSnapshot {
                ticks_run: 1,
                creature_count: 10,
                tagged_entities: TaggedEntities::default(),
                overlaps: Vec::new(),
                tick_duration_us: 1000,
            };
            director.on_tick(&snapshot);
        }

        let final_snapshot = empty_snapshot(110);
        director.complete_trial(&final_snapshot);

        assert_eq!(director.state(), TrialState::Failed);
        let result = director.result().unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_avg_tick_latency_calculation() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        spec.assertions.push(Assertion::MaxAvgTickLatency { microseconds: 600 }); // avg should be 500us
        director.start_trial(spec);

        // Run 100 warm-up ticks (excluded from latency calculation)
        for _ in 0..100 {
            director.on_tick(&empty_snapshot(0));
        }

        // Tick 101: 200us
        director.on_tick(&TrialSnapshot {
            ticks_run: 101,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: Vec::new(),
            tick_duration_us: 200,
        });

        // Tick 102: 800us
        director.on_tick(&TrialSnapshot {
            ticks_run: 102,
            creature_count: 10,
            tagged_entities: TaggedEntities::default(),
            overlaps: Vec::new(),
            tick_duration_us: 800,
        });

        // Avg = (200 + 800) / 2 = 500us < 600us limit
        let final_snapshot = empty_snapshot(102);
        director.complete_trial(&final_snapshot);

        assert_eq!(director.state(), TrialState::Completed);
        let result = director.result().unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_latency_excludes_warmup_ticks() {
        let mut director = TrialDirector::new();

        let mut spec = minimal_spec("Test");
        // 1000us limit - should PASS if we exclude warm-up ticks
        spec.assertions.push(Assertion::MaxAvgTickLatency { microseconds: 1000 });
        director.start_trial(spec);

        // First 100 ticks: high latency (10000us each) - should be EXCLUDED from average
        for _ in 0..100 {
            director.on_tick(&TrialSnapshot {
                ticks_run: 1,
                creature_count: 10,
                tagged_entities: TaggedEntities::default(),
                overlaps: Vec::new(),
                tick_duration_us: 10000, // 10ms - way over limit
            });
        }

        // Next 50 ticks: low latency (500us each) - should be INCLUDED in average
        for _ in 0..50 {
            director.on_tick(&TrialSnapshot {
                ticks_run: 1,
                creature_count: 10,
                tagged_entities: TaggedEntities::default(),
                overlaps: Vec::new(),
                tick_duration_us: 500, // 0.5ms - under limit
            });
        }

        let final_snapshot = empty_snapshot(150);
        director.complete_trial(&final_snapshot);

        // With warm-up excluded: avg = 500us < 1000us limit → PASS
        // Without exclusion: avg = (100*10000 + 50*500) / 150 = 6833us → FAIL
        assert_eq!(director.state(), TrialState::Completed);
        let result = director.result().unwrap();
        assert!(result.passed, "Latency should exclude first 100 warm-up ticks");
    }
}

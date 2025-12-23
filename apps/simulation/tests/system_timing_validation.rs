#[cfg(feature = "dev-tools")]
use std::fs;

#[cfg(feature = "dev-tools")]
#[test]
fn test_all_registered_systems_have_timing() {
    let sim_source = fs::read_to_string("src/simulation/core/simulation.rs")
        .expect("Failed to read simulation.rs");

    let registered_systems_count = count_registered_systems(&sim_source);

    let timing_calls_count = count_timing_calls();

    println!("Registered systems: {}", registered_systems_count);
    println!("Systems with time_system!() calls: {}", timing_calls_count);

    if timing_calls_count < registered_systems_count {
        panic!(
            "\n❌ SYSTEM TIMING VALIDATION FAILED!\n\n\
             Found {} registered systems but only {} time_system!() calls.\n\
             {} system(s) are missing timing instrumentation!\n\n\
             To fix:\n\
             1. Find which system function is missing time_system!()\n\
             2. Add parameter: timings: Res<SystemTimings>\n\
             3. Add macro call: time_system!(timings, \"system_name\");\n\n\
             See apps/simulation/CLAUDE.md for complete instructions.\n",
            registered_systems_count,
            timing_calls_count,
            registered_systems_count - timing_calls_count
        );
    }

    println!(
        "✅ All {} systems have timing instrumentation",
        registered_systems_count
    );
}

#[cfg(feature = "dev-tools")]
fn count_registered_systems(source: &str) -> usize {
    use regex::Regex;

    let re = Regex::new(r"schedule\.add_systems\(\(\s*([^)]+)\s*\)\)").unwrap();

    if let Some(captures) = re.captures(source) {
        let systems_block = &captures[1];
        let system_re = Regex::new(r"(\w+_system)").unwrap();

        system_re
            .captures_iter(systems_block)
            .filter(|cap| {
                let name = &cap[1];
                name != "command_executor_system"
            })
            .count()
    } else {
        0
    }
}

#[cfg(feature = "dev-tools")]
fn count_timing_calls() -> usize {
    use regex::Regex;

    let timing_re = Regex::new(r#"time_system!\s*\("#).unwrap();
    let mut count = 0;

    if let Ok(entries) = fs::read_dir("src") {
        for entry in entries.filter_map(|e| e.ok()) {
            count += count_timing_in_dir(&entry.path(), &timing_re);
        }
    }

    count
}

#[cfg(feature = "dev-tools")]
fn count_timing_in_dir(path: &std::path::Path, timing_re: &regex::Regex) -> usize {
    let mut count = 0;

    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
        if let Ok(source) = fs::read_to_string(path) {
            count += timing_re.find_iter(&source).count();
        }
    } else if path.is_dir() {
        let dir_name = path.file_name().and_then(|s| s.to_str());
        if dir_name != Some("target") && dir_name != Some(".git") {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    count += count_timing_in_dir(&entry.path(), timing_re);
                }
            }
        }
    }

    count
}

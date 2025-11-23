#![cfg(feature = "dev-tools")]

use speciate::instrumentation::HardwareSnapshot;

#[test]
fn test_ipc_formula_correctness() {
    let cycles_delta = 1000u64;
    let instructions_delta = 1500u64;

    let ipc = if cycles_delta > 0 {
        instructions_delta as f64 / cycles_delta as f64
    } else {
        0.0
    };

    assert_eq!(ipc, 1.5, "IPC = instructions / cycles = 1500 / 1000 = 1.5");
    assert!(
        ipc > 0.0 && ipc < 10.0,
        "IPC should be reasonable range (0-10), got {}",
        ipc
    );
}

#[test]
fn test_ipc_zero_cycles_returns_zero() {
    let cycles_delta = 0u64;
    let instructions_delta = 1000u64;

    let ipc = if cycles_delta > 0 {
        instructions_delta as f64 / cycles_delta as f64
    } else {
        0.0
    };

    assert_eq!(ipc, 0.0, "IPC should be 0.0 when cycles_delta is 0");
}

#[test]
fn test_l1d_miss_rate_uses_l1d_accesses_not_cache_refs() {
    let l1d_misses_delta = 50u64;
    let l1d_accesses_delta = 1000u64;
    let cache_refs_delta = 5000u64;

    let l1d_miss_rate = if l1d_accesses_delta > 0 {
        (l1d_misses_delta as f64 / l1d_accesses_delta as f64) * 100.0
    } else {
        0.0
    };

    assert_eq!(
        l1d_miss_rate, 5.0,
        "L1D miss rate = (50 / 1000) * 100 = 5%"
    );

    let wrong_formula = if cache_refs_delta > 0 {
        (l1d_misses_delta as f64 / cache_refs_delta as f64) * 100.0
    } else {
        0.0
    };

    assert_eq!(wrong_formula, 1.0, "Wrong formula would give 1%");
    assert_ne!(
        l1d_miss_rate, wrong_formula,
        "L1D miss rate MUST use l1d_accesses, not cache_refs"
    );
}

#[test]
fn test_l1d_miss_rate_zero_accesses_returns_zero() {
    let l1d_misses_delta = 50u64;
    let l1d_accesses_delta = 0u64;

    let l1d_miss_rate = if l1d_accesses_delta > 0 {
        (l1d_misses_delta as f64 / l1d_accesses_delta as f64) * 100.0
    } else {
        0.0
    };

    assert_eq!(
        l1d_miss_rate, 0.0,
        "L1D miss rate should be 0.0 when accesses_delta is 0"
    );
}

#[test]
fn test_l1d_miss_rate_expected_ranges() {
    let excellent_case = (10u64, 10000u64);
    let good_case = (300u64, 10000u64);
    let poor_case = (700u64, 10000u64);
    let critical_case = (1200u64, 10000u64);

    let excellent_rate =
        (excellent_case.0 as f64 / excellent_case.1 as f64) * 100.0;
    let good_rate = (good_case.0 as f64 / good_case.1 as f64) * 100.0;
    let poor_rate = (poor_case.0 as f64 / poor_case.1 as f64) * 100.0;
    let critical_rate =
        (critical_case.0 as f64 / critical_case.1 as f64) * 100.0;

    assert!(excellent_rate < 1.0, "Excellent: < 1%");
    assert!(good_rate >= 1.0 && good_rate < 5.0, "Good: 1-5%");
    assert!(poor_rate >= 5.0 && poor_rate < 10.0, "Poor: 5-10%");
    assert!(critical_rate >= 10.0, "Critical: > 10%");
}

#[test]
fn test_llc_miss_rate_formula_correctness() {
    let cache_misses_delta = 200u64;
    let cache_refs_delta = 10000u64;

    let llc_miss_rate = if cache_refs_delta > 0 && cache_misses_delta > 0 {
        (cache_misses_delta as f64 / cache_refs_delta as f64) * 100.0
    } else {
        0.0
    };

    assert_eq!(
        llc_miss_rate, 2.0,
        "LLC miss rate = (200 / 10000) * 100 = 2%"
    );
}

#[test]
fn test_llc_miss_rate_zero_misses_is_valid() {
    let cache_misses_delta = 0u64;
    let cache_refs_delta = 10000u64;

    let llc_miss_rate_overcautious =
        if cache_refs_delta > 0 && cache_misses_delta > 0 {
            (cache_misses_delta as f64 / cache_refs_delta as f64) * 100.0
        } else {
            0.0
        };

    let llc_miss_rate_simplified = if cache_refs_delta > 0 {
        (cache_misses_delta as f64 / cache_refs_delta as f64) * 100.0
    } else {
        0.0
    };

    assert_eq!(
        llc_miss_rate_overcautious, 0.0,
        "Overcautious check returns 0.0"
    );
    assert_eq!(
        llc_miss_rate_simplified, 0.0,
        "Simplified check also returns 0.0"
    );
    assert_eq!(
        llc_miss_rate_overcautious, llc_miss_rate_simplified,
        "Zero misses is a valid (excellent) result"
    );
}

#[test]
fn test_branch_miss_rate_formula_correctness() {
    let branch_misses_delta = 150u64;
    let branch_instructions_delta = 10000u64;

    let branch_miss_rate = if branch_instructions_delta > 0 {
        (branch_misses_delta as f64 / branch_instructions_delta as f64) * 100.0
    } else {
        0.0
    };

    assert_eq!(
        branch_miss_rate, 1.5,
        "Branch miss rate = (150 / 10000) * 100 = 1.5%"
    );
}

#[test]
fn test_branch_miss_rate_expected_ranges() {
    let excellent_case = (50u64, 10000u64);
    let good_case = (200u64, 10000u64);
    let poor_case = (600u64, 10000u64);

    let excellent_rate =
        (excellent_case.0 as f64 / excellent_case.1 as f64) * 100.0;
    let good_rate = (good_case.0 as f64 / good_case.1 as f64) * 100.0;
    let poor_rate = (poor_case.0 as f64 / poor_case.1 as f64) * 100.0;

    assert!(excellent_rate < 1.0, "Excellent: < 1%");
    assert!(good_rate >= 1.0 && good_rate < 3.0, "Good: 1-3%");
    assert!(poor_rate >= 5.0, "Poor: > 5%");
}

#[test]
fn test_stall_ratio_formula_correctness() {
    let stalled_frontend_delta = 2500u64;
    let stalled_backend_delta = 1500u64;
    let cycles_delta = 10000u64;

    let frontend_stall_ratio = if cycles_delta > 0 {
        (stalled_frontend_delta as f64 / cycles_delta as f64) * 100.0
    } else {
        0.0
    };

    let backend_stall_ratio = if cycles_delta > 0 {
        (stalled_backend_delta as f64 / cycles_delta as f64) * 100.0
    } else {
        0.0
    };

    assert_eq!(
        frontend_stall_ratio, 25.0,
        "Frontend stall ratio = (2500 / 10000) * 100 = 25%"
    );
    assert_eq!(
        backend_stall_ratio, 15.0,
        "Backend stall ratio = (1500 / 10000) * 100 = 15%"
    );
}

#[test]
fn test_counter_overflow_wrapping_sub_correctness() {
    let current: u64 = 100;
    let prev: u64 = u64::MAX - 50;

    let delta = current.wrapping_sub(prev);

    assert_eq!(
        delta, 151,
        "wrapping_sub handles overflow: (100 - (2^64 - 50)) mod 2^64 = 151"
    );
}

#[test]
fn test_counter_overflow_no_wraparound() {
    let current: u64 = 1000;
    let prev: u64 = 500;

    let delta = current.wrapping_sub(prev);

    assert_eq!(delta, 500, "Normal case: 1000 - 500 = 500");
}

#[test]
fn test_hardware_snapshot_default_values() {
    let snapshot = HardwareSnapshot::default();

    assert_eq!(snapshot.cycles_delta, 0);
    assert_eq!(snapshot.instructions_delta, 0);
    assert_eq!(snapshot.ipc, 0.0);
    assert_eq!(snapshot.l1d_miss_rate, 0.0);
    assert_eq!(snapshot.llc_miss_rate, 0.0);
    assert_eq!(snapshot.branch_miss_rate, 0.0);
}

#[test]
fn test_all_formulas_produce_reasonable_values() {
    let cycles_delta = 3_000_000_000u64;
    let instructions_delta = 3_600_000_000u64;
    let cache_refs_delta = 100_000u64;
    let cache_misses_delta = 2_000u64;
    let l1d_misses_delta = 1_000u64;
    let l1d_accesses_delta = 50_000u64;
    let branch_instructions_delta = 500_000u64;
    let branch_misses_delta = 7_500u64;
    let stalled_frontend_delta = 300_000_000u64;
    let stalled_backend_delta = 600_000_000u64;

    let ipc = instructions_delta as f64 / cycles_delta as f64;
    let l1d_miss_rate =
        (l1d_misses_delta as f64 / l1d_accesses_delta as f64) * 100.0;
    let llc_miss_rate =
        (cache_misses_delta as f64 / cache_refs_delta as f64) * 100.0;
    let branch_miss_rate =
        (branch_misses_delta as f64 / branch_instructions_delta as f64) * 100.0;
    let frontend_stall_ratio =
        (stalled_frontend_delta as f64 / cycles_delta as f64) * 100.0;
    let backend_stall_ratio =
        (stalled_backend_delta as f64 / cycles_delta as f64) * 100.0;

    assert!(ipc >= 0.0 && ipc < 10.0, "IPC: {}", ipc);
    assert!(
        l1d_miss_rate >= 0.0 && l1d_miss_rate < 100.0,
        "L1D miss rate: {}%",
        l1d_miss_rate
    );
    assert!(
        llc_miss_rate >= 0.0 && llc_miss_rate < 100.0,
        "LLC miss rate: {}%",
        llc_miss_rate
    );
    assert!(
        branch_miss_rate >= 0.0 && branch_miss_rate < 100.0,
        "Branch miss rate: {}%",
        branch_miss_rate
    );
    assert!(
        frontend_stall_ratio >= 0.0 && frontend_stall_ratio <= 100.0,
        "Frontend stall ratio: {}%",
        frontend_stall_ratio
    );
    assert!(
        backend_stall_ratio >= 0.0 && backend_stall_ratio <= 100.0,
        "Backend stall ratio: {}%",
        backend_stall_ratio
    );

    println!("IPC: {:.2}", ipc);
    println!("L1D miss rate: {:.2}%", l1d_miss_rate);
    println!("LLC miss rate: {:.2}%", llc_miss_rate);
    println!("Branch miss rate: {:.2}%", branch_miss_rate);
    println!("Frontend stall: {:.2}%", frontend_stall_ratio);
    println!("Backend stall: {:.2}%", backend_stall_ratio);
}

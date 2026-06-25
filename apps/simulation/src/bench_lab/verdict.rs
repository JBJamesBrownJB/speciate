//! Keep/Defer/Ditch classifier for a perf experiment.
//!
//! WHY: the budget is whole-tick p99, but whole-tick noise (the sum of every
//! phase's variance) can drown a real win in the one phase a change actually
//! touches. So detection is judged against the *targeted phase's* own noise
//! floor, while banking still requires whole-tick evidence. A real per-phase
//! win that the tick noise hides is parked (Defer) for later stacking, never
//! discarded as if it were noise. See `docs/scale/optimization-checklist.md`.
//!
//! Detection & banking deltas are MEDIANS, not p99: run-to-run, the per-seed
//! median is ~3× quieter than p99 (whose noise lives in 1–2 tail samples), so a
//! real shift in typical cost clears a noise floor that p99 buries. p99 stays as
//! the *regression* guard (`worst_phase_regression_us`) — improvements are judged
//! on the quiet statistic, tail blow-ups on the strict one. See
//! `docs/scale/perf-hunt/noise-characterization-2026-06-25.md`.

/// A change must beat this multiple of the relevant noise floor to count as a
/// real signal rather than seed-to-seed luck.
pub const NOISE_MULTIPLIER: f64 = 2.0;

/// Any single phase regressing by more than this (µs) ditches the change
/// outright, regardless of net win — it has shifted cost, not removed it.
pub const PHASE_REGRESSION_LIMIT_US: f64 = 2_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Real at the phase level and visibly moves the whole-tick budget. Bank it.
    Keep,
    /// Real at the phase level but too small to see in the tick. Park it; stack
    /// with other deferred wins and re-measure the union before discarding.
    Defer,
    /// Indistinguishable from noise, or it regressed the tick / a phase. Drop it.
    Ditch,
}

/// Evidence for one experiment, expressed as deltas (after − before, µs) and the
/// noise floors (std of the paired per-seed differences, µs) the deltas are judged
/// against. Negative delta = improvement.
#[derive(Debug, Clone, Copy)]
pub struct ChangeEvidence {
    /// Δmedian of the phase the change targets (the detection statistic).
    pub phase_delta_us: f64,
    /// Noise floor of that phase (std of its paired per-seed median differences).
    pub phase_noise_floor_us: f64,
    /// Δmedian of the whole-tick wall clock (the banking statistic).
    pub tick_delta_us: f64,
    /// Noise floor of the whole-tick wall clock (paired median differences).
    pub tick_noise_floor_us: f64,
    /// Worst (most positive) per-phase **p99** regression across all phases — the
    /// strict tail guard: a change may not bank a median win by trading away the tail.
    pub worst_phase_regression_us: f64,
}

pub fn classify(ev: &ChangeEvidence) -> Verdict {
    // A regression in any phase that exceeds the limit is disqualifying: the
    // change moved cost around rather than removing it.
    if ev.worst_phase_regression_us > PHASE_REGRESSION_LIMIT_US {
        return Verdict::Ditch;
    }

    // Detect against the targeted phase's own noise, not the fatter tick noise.
    let phase_improvement = -ev.phase_delta_us;
    let detectable = phase_improvement > NOISE_MULTIPLIER * ev.phase_noise_floor_us;
    if !detectable {
        return Verdict::Ditch;
    }

    // Real at the phase level. Bank only on whole-tick evidence; a confident
    // tick regression means a hidden cost landed elsewhere; within tick noise
    // means a genuine win too small to see alone → park it for stacking.
    let tick_improvement = -ev.tick_delta_us;
    let tick_band = NOISE_MULTIPLIER * ev.tick_noise_floor_us;
    if tick_improvement > tick_band {
        Verdict::Keep
    } else if tick_improvement < -tick_band {
        Verdict::Ditch
    } else {
        Verdict::Defer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Baseline evidence: a clean, large win with negligible noise. Individual
    /// tests perturb one field to isolate each rule.
    fn clean_win() -> ChangeEvidence {
        ChangeEvidence {
            phase_delta_us: -3_000.0,
            phase_noise_floor_us: 200.0,
            tick_delta_us: -3_000.0,
            tick_noise_floor_us: 200.0,
            worst_phase_regression_us: 0.0,
        }
    }

    #[test]
    fn detectable_phase_win_that_moves_tick_is_kept() {
        assert_eq!(classify(&clean_win()), Verdict::Keep);
    }

    #[test]
    fn phase_change_within_its_own_noise_is_ditched() {
        // 300µs "win" against a 200µs phase noise floor: under 2× → not real.
        let ev = ChangeEvidence {
            phase_delta_us: -300.0,
            phase_noise_floor_us: 200.0,
            ..clean_win()
        };
        assert_eq!(classify(&ev), Verdict::Ditch);
    }

    #[test]
    fn real_phase_win_invisible_in_tick_noise_is_deferred() {
        // The crux: a 3ms phase win is obvious against a 200µs phase floor, but
        // the tick only moved 1ms against a 2ms tick floor — drowned. Park it.
        let ev = ChangeEvidence {
            phase_delta_us: -3_000.0,
            phase_noise_floor_us: 200.0,
            tick_delta_us: -1_000.0,
            tick_noise_floor_us: 2_000.0,
            worst_phase_regression_us: 0.0,
        };
        assert_eq!(classify(&ev), Verdict::Defer);
    }

    #[test]
    fn phase_win_with_confident_tick_regression_is_ditched() {
        // Phase improved, but the tick got confidently worse — hidden cost
        // landed elsewhere. Not a win to bank or park.
        let ev = ChangeEvidence {
            phase_delta_us: -3_000.0,
            phase_noise_floor_us: 200.0,
            tick_delta_us: 3_000.0,
            tick_noise_floor_us: 200.0,
            worst_phase_regression_us: 0.0,
        };
        assert_eq!(classify(&ev), Verdict::Ditch);
    }

    #[test]
    fn any_phase_regression_over_limit_is_ditched() {
        // Even with a real targeted win and a net tick gain, a >2ms regression
        // in some other phase ditches it.
        let ev = ChangeEvidence {
            worst_phase_regression_us: 2_500.0,
            ..clean_win()
        };
        assert_eq!(classify(&ev), Verdict::Ditch);
    }

    #[test]
    fn phase_regression_is_not_credited_as_a_win() {
        // A positive phase delta is a regression, never "detectable improvement".
        let ev = ChangeEvidence {
            phase_delta_us: 3_000.0,
            phase_noise_floor_us: 200.0,
            ..clean_win()
        };
        assert_eq!(classify(&ev), Verdict::Ditch);
    }
}

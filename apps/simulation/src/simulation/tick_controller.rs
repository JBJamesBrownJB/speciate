use std::time::Instant;

/// Controls game loop timing using the accumulator pattern.
///
/// This is the industry-standard approach used by Unity, Unreal, and Godot.
/// It provides:
/// - Fixed timestep physics (deterministic, stable)
/// - Catch-up after lag spikes (runs multiple ticks to recover)
/// - Death spiral prevention (caps catch-up to avoid freezing)
/// - Time scaling (fast-forward, slow-motion)
pub struct TickController {
    accumulator: f32,
    previous_time: Instant,
    time_scale: f32,
    total_ticks: u64,
}

/// Metrics from a single frame's tick processing
#[derive(Debug, Clone, Copy)]
pub struct TickMetrics {
    /// Number of simulation ticks run this frame
    pub ticks_this_frame: u32,
    /// Real time that was dropped to prevent death spiral (seconds)
    pub time_dropped: f32,
    /// Leftover accumulator time carried to next frame (seconds)
    pub accumulator_remainder: f32,
    /// Total ticks since controller was created
    pub total_ticks: u64,
}

impl TickController {
    /// Fixed delta time per tick (20Hz = 50ms per tick)
    pub const FIXED_DT: f32 = 1.0 / 20.0;

    /// Maximum real time to accumulate per frame (prevents death spiral)
    /// 250ms = 5 ticks max catch-up
    pub const MAX_FRAME_TIME: f32 = 0.25;

    /// Maximum ticks to run in a single frame
    pub const MAX_TICKS_PER_FRAME: u32 = 5;

    pub fn new() -> Self {
        Self {
            accumulator: 0.0,
            previous_time: Instant::now(),
            time_scale: 1.0,
            total_ticks: 0,
        }
    }

    /// Set time scale (1.0 = normal, 2.0 = 2x speed, 0.5 = half speed)
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.clamp(0.0, 1000.0);
    }

    /// Get current time scale
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Process one frame, running as many ticks as needed to catch up.
    ///
    /// The update_fn is called once per tick with the fixed delta time.
    /// Returns metrics about how many ticks were run.
    pub fn tick<F>(&mut self, mut update_fn: F) -> TickMetrics
    where
        F: FnMut(f32),
    {
        let current_time = Instant::now();
        let mut frame_time = current_time
            .duration_since(self.previous_time)
            .as_secs_f32();
        self.previous_time = current_time;

        // Cap frame time to prevent death spiral
        let time_dropped = if frame_time > Self::MAX_FRAME_TIME {
            let dropped = frame_time - Self::MAX_FRAME_TIME;
            frame_time = Self::MAX_FRAME_TIME;
            dropped
        } else {
            0.0
        };

        // Apply time scale
        self.accumulator += frame_time * self.time_scale;

        // Run ticks to catch up (with cap to prevent too many in one frame)
        let mut ticks_this_frame = 0u32;

        while self.accumulator >= Self::FIXED_DT && ticks_this_frame < Self::MAX_TICKS_PER_FRAME {
            update_fn(Self::FIXED_DT);
            self.accumulator -= Self::FIXED_DT;
            ticks_this_frame += 1;
            self.total_ticks += 1;
        }

        // If still behind after max ticks, drop excess to prevent permanent lag
        let additional_dropped = if self.accumulator > Self::FIXED_DT * 2.0 {
            let excess = self.accumulator - Self::FIXED_DT;
            self.accumulator = Self::FIXED_DT;
            excess
        } else {
            0.0
        };

        TickMetrics {
            ticks_this_frame,
            time_dropped: time_dropped + additional_dropped,
            accumulator_remainder: self.accumulator,
            total_ticks: self.total_ticks,
        }
    }

    /// Reset the controller (e.g., after unpausing)
    pub fn reset(&mut self) {
        self.accumulator = 0.0;
        self.previous_time = Instant::now();
    }
}

impl Default for TickController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_new_controller() {
        let controller = TickController::new();
        assert_eq!(controller.time_scale(), 1.0);
        assert_eq!(controller.accumulator, 0.0);
    }

    #[test]
    fn test_time_scale_clamping() {
        let mut controller = TickController::new();

        controller.set_time_scale(5.0);
        assert_eq!(controller.time_scale(), 5.0);

        controller.set_time_scale(-1.0);
        assert_eq!(controller.time_scale(), 0.0);

        controller.set_time_scale(1001.0);
        assert_eq!(controller.time_scale(), 1000.0);
    }

    #[test]
    fn test_single_tick_after_50ms() {
        let mut controller = TickController::new();

        // Wait ~60ms to ensure we have enough time for 1 tick
        thread::sleep(Duration::from_millis(60));

        let mut tick_count = 0;
        let metrics = controller.tick(|_dt| {
            tick_count += 1;
        });

        assert!(
            metrics.ticks_this_frame >= 1,
            "Should run at least 1 tick after 60ms"
        );
        assert_eq!(tick_count, metrics.ticks_this_frame as usize);
    }

    #[test]
    fn test_multiple_ticks_catch_up() {
        let mut controller = TickController::new();

        // Wait 150ms = 3 ticks worth
        thread::sleep(Duration::from_millis(150));

        let mut tick_count = 0;
        let metrics = controller.tick(|_dt| {
            tick_count += 1;
        });

        // Should run 2-3 ticks to catch up
        assert!(
            metrics.ticks_this_frame >= 2 && metrics.ticks_this_frame <= 4,
            "Expected 2-4 ticks, got {}",
            metrics.ticks_this_frame
        );
    }

    #[test]
    fn test_death_spiral_prevention() {
        let mut controller = TickController::new();

        // Wait 500ms = would be 10 ticks, but capped at 5
        thread::sleep(Duration::from_millis(500));

        let mut tick_count = 0;
        let metrics = controller.tick(|_dt| {
            tick_count += 1;
        });

        assert!(
            metrics.ticks_this_frame <= TickController::MAX_TICKS_PER_FRAME,
            "Should cap at {} ticks, got {}",
            TickController::MAX_TICKS_PER_FRAME,
            metrics.ticks_this_frame
        );
        assert!(metrics.time_dropped > 0.0, "Should have dropped some time");
    }

    #[test]
    fn test_time_scale_double_speed() {
        let mut controller = TickController::new();
        controller.set_time_scale(2.0);

        // Wait 60ms, with 2x scale should accumulate 120ms = 2+ ticks
        thread::sleep(Duration::from_millis(60));

        let mut tick_count = 0;
        let metrics = controller.tick(|_dt| {
            tick_count += 1;
        });

        assert!(
            metrics.ticks_this_frame >= 2,
            "With 2x scale after 60ms, expected 2+ ticks, got {}",
            metrics.ticks_this_frame
        );
    }

    #[test]
    fn test_time_scale_zero_pauses() {
        let mut controller = TickController::new();
        controller.set_time_scale(0.0);

        thread::sleep(Duration::from_millis(100));

        let mut tick_count = 0;
        let metrics = controller.tick(|_dt| {
            tick_count += 1;
        });

        assert_eq!(
            metrics.ticks_this_frame, 0,
            "With 0x scale, no ticks should run"
        );
    }

    #[test]
    fn test_reset_clears_accumulator() {
        let mut controller = TickController::new();

        thread::sleep(Duration::from_millis(100));
        controller.reset();

        // Immediately after reset, no time has passed
        let metrics = controller.tick(|_dt| {});

        assert_eq!(
            metrics.ticks_this_frame, 0,
            "After reset, no ticks should run immediately"
        );
    }

    #[test]
    fn test_total_ticks_accumulates() {
        let mut controller = TickController::new();

        thread::sleep(Duration::from_millis(60));
        let metrics1 = controller.tick(|_dt| {});

        thread::sleep(Duration::from_millis(60));
        let metrics2 = controller.tick(|_dt| {});

        assert!(
            metrics2.total_ticks > metrics1.total_ticks,
            "Total ticks should accumulate"
        );
    }
}

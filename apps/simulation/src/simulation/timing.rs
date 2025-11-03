use std::collections::VecDeque;
use std::time::Duration;

/// Tracks tick timing statistics with a rolling window
#[derive(Debug)]
pub struct TickTimer {
    recent_durations: VecDeque<Duration>,
    window_size: usize,
    tick_count: u64,
    report_interval: u64,
}

impl TickTimer {
    /// Create a new TickTimer with specified window size and report interval
    pub fn new(window_size: usize, report_interval: u64) -> Self {
        Self {
            recent_durations: VecDeque::with_capacity(window_size),
            window_size,
            tick_count: 0,
            report_interval,
        }
    }

    /// Record a tick duration and return whether to report
    pub fn record_tick(&mut self, duration: Duration) -> bool {
        self.recent_durations.push_back(duration);
        if self.recent_durations.len() > self.window_size {
            self.recent_durations.pop_front();
        }

        self.tick_count += 1;
        self.tick_count.is_multiple_of(self.report_interval)
    }

    /// Get the current tick count
    /// Calculate average duration over the rolling window
    pub fn average_duration(&self) -> Option<Duration> {
        if self.recent_durations.is_empty() {
            return None;
        }

        let total: Duration = self.recent_durations.iter().sum();
        Some(total / self.recent_durations.len() as u32)
    }

    /// Get the most recent tick duration
    pub fn current_duration(&self) -> Option<Duration> {
        self.recent_durations.back().copied()
    }

    /// Format timing statistics for logging
    pub fn format_stats(&self) -> String {
        let avg = self.average_duration()
            .map(|d| format!("{:.2}ms", d.as_secs_f64() * 1000.0))
            .unwrap_or_else(|| "N/A".to_string());

        let current = self.current_duration()
            .map(|d| format!("{:.2}ms", d.as_secs_f64() * 1000.0))
            .unwrap_or_else(|| "N/A".to_string());

        format!("[Tick {}] Avg: {}, Current: {}", self.tick_count, avg, current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_new_tick_timer() {
        let timer = TickTimer::new(100, 60);
        assert_eq!(timer.window_size, 100);
        assert_eq!(timer.report_interval, 60);
        assert!(timer.average_duration().is_none());
        assert!(timer.current_duration().is_none());
    }

    #[test]
    fn test_record_single_tick() {
        let mut timer = TickTimer::new(100, 60);
        let duration = Duration::from_millis(10);

        let should_report = timer.record_tick(duration);

        assert!(!should_report); // First tick shouldn't trigger report
        assert_eq!(timer.current_duration(), Some(duration));
        assert_eq!(timer.average_duration(), Some(duration));
    }

    #[test]
    fn test_report_interval() {
        let mut timer = TickTimer::new(100, 5);

        for i in 1..=5 {
            let should_report = timer.record_tick(Duration::from_millis(10));
            if i < 5 {
                assert!(!should_report, "Tick {} should not report", i);
            } else {
                assert!(should_report, "Tick {} should report", i);
            }
        }

        // Next report should be at tick 10
        for _ in 6..10 {
            let should_report = timer.record_tick(Duration::from_millis(10));
            assert!(!should_report);
        }

        let should_report = timer.record_tick(Duration::from_millis(10));
        assert!(should_report);
    }

    #[test]
    fn test_rolling_window() {
        let mut timer = TickTimer::new(3, 60); // Small window for testing

        timer.record_tick(Duration::from_millis(10));
        timer.record_tick(Duration::from_millis(20));
        timer.record_tick(Duration::from_millis(30));

        // Average should be (10 + 20 + 30) / 3 = 20ms
        let avg = timer.average_duration().unwrap();
        assert_eq!(avg.as_millis(), 20);

        // Add fourth tick - should evict first (10ms)
        timer.record_tick(Duration::from_millis(40));

        // Average should be (20 + 30 + 40) / 3 = 30ms
        let avg = timer.average_duration().unwrap();
        assert_eq!(avg.as_millis(), 30);
    }

    #[test]
    fn test_average_calculation() {
        let mut timer = TickTimer::new(100, 60);

        timer.record_tick(Duration::from_millis(5));
        timer.record_tick(Duration::from_millis(10));
        timer.record_tick(Duration::from_millis(15));

        let avg = timer.average_duration().unwrap();
        assert_eq!(avg.as_millis(), 10);
    }

    #[test]
    fn test_current_duration() {
        let mut timer = TickTimer::new(100, 60);

        timer.record_tick(Duration::from_millis(5));
        timer.record_tick(Duration::from_millis(10));
        timer.record_tick(Duration::from_millis(15));

        assert_eq!(timer.current_duration(), Some(Duration::from_millis(15)));
    }

    #[test]
    fn test_format_stats() {
        let mut timer = TickTimer::new(100, 60);

        timer.record_tick(Duration::from_millis(8));
        timer.record_tick(Duration::from_millis(9));
        timer.record_tick(Duration::from_millis(10));

        let stats = timer.format_stats();
        assert!(stats.contains("[Tick 3]"));
        assert!(stats.contains("Avg:"));
        assert!(stats.contains("Current:"));
        assert!(stats.contains("ms"));
    }

    #[test]
    fn test_empty_timer_format() {
        let timer = TickTimer::new(100, 60);
        let stats = timer.format_stats();

        assert!(stats.contains("[Tick 0]"));
        assert!(stats.contains("N/A"));
    }
}

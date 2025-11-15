use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug)]
pub struct TickTimer {
    recent_durations: VecDeque<Duration>,
    window_size: usize,
    tick_count: u64,
    report_interval: u64,
}

impl TickTimer {
    pub fn new(window_size: usize, report_interval: u64) -> Self {
        Self {
            recent_durations: VecDeque::with_capacity(window_size),
            window_size,
            tick_count: 0,
            report_interval,
        }
    }

    pub fn record_tick(&mut self, duration: Duration) -> bool {
        self.recent_durations.push_back(duration);
        if self.recent_durations.len() > self.window_size {
            self.recent_durations.pop_front();
        }

        self.tick_count += 1;
        self.tick_count.is_multiple_of(self.report_interval)
    }

    pub fn average_duration(&self) -> Option<Duration> {
        if self.recent_durations.is_empty() {
            return None;
        }

        let total: Duration = self.recent_durations.iter().sum();
        Some(total / self.recent_durations.len() as u32)
    }

    pub fn current_duration(&self) -> Option<Duration> {
        self.recent_durations.back().copied()
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

        assert!(!should_report);
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

        for _ in 6..10 {
            let should_report = timer.record_tick(Duration::from_millis(10));
            assert!(!should_report);
        }

        let should_report = timer.record_tick(Duration::from_millis(10));
        assert!(should_report);
    }

    #[test]
    fn test_rolling_window() {
        let mut timer = TickTimer::new(3, 60);

        timer.record_tick(Duration::from_millis(10));
        timer.record_tick(Duration::from_millis(20));
        timer.record_tick(Duration::from_millis(30));

        let avg = timer.average_duration().unwrap();
        assert_eq!(avg.as_millis(), 20);

        timer.record_tick(Duration::from_millis(40));

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
}

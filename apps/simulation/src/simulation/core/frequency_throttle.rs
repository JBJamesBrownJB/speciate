/// Frequency throttle state for entity-ID bucketing.
///
/// Uses power-of-2 divisors for bitwise AND optimization:
/// `entity.index() & (divisor-1)` is 1 CPU cycle vs 30 cycles for modulo.
pub struct FrequencyThrottle {
    bucket_mask: usize,
    current_bucket: usize,
}

impl FrequencyThrottle {
    /// Create throttle from divisor and current tick.
    /// Divisor MUST be power-of-2 (2, 4, or 8) - use FreqConfig::clamp_power_of_2().
    #[inline(always)]
    pub fn new(divisor: u8, tick: u64) -> Self {
        let divisor = divisor as usize;
        Self {
            bucket_mask: divisor - 1,
            current_bucket: (tick as usize) & (divisor - 1),
        }
    }

    /// Returns true if this entity should be processed this tick.
    /// Distributes entities evenly across ticks based on entity index.
    #[inline(always)]
    pub fn should_process(&self, entity_index: u32) -> bool {
        (entity_index as usize) & self.bucket_mask == self.current_bucket
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_throttle_divisor_2_distribution() {
        // Divisor 2: entities split into 2 buckets (even/odd indices)
        let throttle_tick0 = FrequencyThrottle::new(2, 0);
        let throttle_tick1 = FrequencyThrottle::new(2, 1);

        // Tick 0: processes entities 0, 2, 4, 6...
        assert!(throttle_tick0.should_process(0));
        assert!(!throttle_tick0.should_process(1));
        assert!(throttle_tick0.should_process(2));
        assert!(!throttle_tick0.should_process(3));

        // Tick 1: processes entities 1, 3, 5, 7...
        assert!(!throttle_tick1.should_process(0));
        assert!(throttle_tick1.should_process(1));
        assert!(!throttle_tick1.should_process(2));
        assert!(throttle_tick1.should_process(3));
    }

    #[test]
    fn test_throttle_divisor_4_distribution() {
        // Divisor 4: entities split into 4 buckets
        let throttle = FrequencyThrottle::new(4, 0);

        // Tick 0: processes entities 0, 4, 8, 12...
        assert!(throttle.should_process(0));
        assert!(!throttle.should_process(1));
        assert!(!throttle.should_process(2));
        assert!(!throttle.should_process(3));
        assert!(throttle.should_process(4));
        assert!(throttle.should_process(8));
    }

    #[test]
    fn test_throttle_divisor_8_distribution() {
        // Divisor 8: entities split into 8 buckets
        let throttle = FrequencyThrottle::new(8, 3);

        // Tick 3: processes entities 3, 11, 19...
        assert!(!throttle.should_process(0));
        assert!(!throttle.should_process(1));
        assert!(!throttle.should_process(2));
        assert!(throttle.should_process(3));
        assert!(!throttle.should_process(4));
        assert!(throttle.should_process(11));
        assert!(throttle.should_process(19));
    }

    #[test]
    fn test_throttle_wrapping_ticks() {
        // Tick wraps around based on divisor
        let throttle_t0 = FrequencyThrottle::new(4, 0);
        let throttle_t4 = FrequencyThrottle::new(4, 4);
        let throttle_t8 = FrequencyThrottle::new(4, 8);

        // Ticks 0, 4, 8 should all process same entities (bucket 0)
        assert!(throttle_t0.should_process(0));
        assert!(throttle_t4.should_process(0));
        assert!(throttle_t8.should_process(0));

        assert!(throttle_t0.should_process(4));
        assert!(throttle_t4.should_process(4));
        assert!(throttle_t8.should_process(4));
    }

    #[test]
    fn test_all_entities_processed_over_cycle() {
        // Over N ticks (where N = divisor), all entities get processed exactly once
        let divisor = 4u8;
        let entity_count = 16u32;

        let mut processed_count = vec![0usize; entity_count as usize];

        for tick in 0..divisor as u64 {
            let throttle = FrequencyThrottle::new(divisor, tick);
            for entity_idx in 0..entity_count {
                if throttle.should_process(entity_idx) {
                    processed_count[entity_idx as usize] += 1;
                }
            }
        }

        // Each entity should be processed exactly once per cycle
        for (idx, count) in processed_count.iter().enumerate() {
            assert_eq!(
                *count, 1,
                "Entity {} processed {} times (expected 1)",
                idx, count
            );
        }
    }
}

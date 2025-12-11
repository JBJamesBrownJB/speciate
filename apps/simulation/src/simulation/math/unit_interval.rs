/// A float guaranteed to be in the range [0.0, 1.0].
///
/// Uses compile-time validation via const fn to catch invalid values at build time.
/// This is used for force multipliers to ensure they stay within the valid range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitInterval(f32);

impl UnitInterval {
    // 1.0 in f32 bits is 0x3f800000
    const ONE_BITS: u32 = 0x3f800000;

    /// Creates a new UnitInterval.
    ///
    /// This function is `const`, so invalid values are caught at compile time
    /// when used to initialize const variables.
    ///
    /// # Panics
    /// Panics if `val` is not in [0.0, 1.0].
    pub const fn new(val: f32) -> Self {
        let bits = val.to_bits();

        // In IEEE 754, negative numbers have the high bit set.
        // When interpreted as u32, negative floats have values > 0x80000000.
        // Positive floats <= 1.0 have bits <= ONE_BITS.
        // This single check ensures:
        // 1. The number is not negative (high bit would make it huge)
        // 2. The number is <= 1.0
        if bits > Self::ONE_BITS {
            panic!("UnitInterval value must be between 0.0 and 1.0");
        }

        Self(val)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f32 {
        self.0
    }
}

// Compile-time validation examples:

// ✅ Valid values - these compile successfully
pub const VALID_ZERO: UnitInterval = UnitInterval::new(0.0);
pub const VALID_HALF: UnitInterval = UnitInterval::new(0.5);
pub const VALID_ONE: UnitInterval = UnitInterval::new(1.0);

// ❌ Invalid values - uncomment to see compile-time errors:
// pub const INVALID_NEGATIVE: UnitInterval = UnitInterval::new(-0.1);
// pub const INVALID_OVER_ONE: UnitInterval = UnitInterval::new(1.5);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_values() {
        assert_eq!(UnitInterval::new(0.0).get(), 0.0);
        assert_eq!(UnitInterval::new(0.5).get(), 0.5);
        assert_eq!(UnitInterval::new(1.0).get(), 1.0);
        assert_eq!(UnitInterval::new(0.001).get(), 0.001);
        assert_eq!(UnitInterval::new(0.999).get(), 0.999);
    }

    #[test]
    fn test_const_values_are_accessible() {
        assert_eq!(VALID_ZERO.get(), 0.0);
        assert_eq!(VALID_HALF.get(), 0.5);
        assert_eq!(VALID_ONE.get(), 1.0);
    }

    #[test]
    #[should_panic(expected = "UnitInterval value must be between 0.0 and 1.0")]
    fn test_rejects_negative() {
        let _ = UnitInterval::new(-0.1);
    }

    #[test]
    #[should_panic(expected = "UnitInterval value must be between 0.0 and 1.0")]
    fn test_rejects_over_one() {
        let _ = UnitInterval::new(1.1);
    }

    #[test]
    #[should_panic(expected = "UnitInterval value must be between 0.0 and 1.0")]
    fn test_rejects_large_value() {
        let _ = UnitInterval::new(100.0);
    }
}

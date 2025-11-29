// Flee behavior constants

pub const FLEE_FORCE: f32 = 20.0; // Threat response force (Newtons)

// TODO: Flee behavior not yet implemented

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flee_force_positive() {
        assert!(FLEE_FORCE > 0.0);
    }
}

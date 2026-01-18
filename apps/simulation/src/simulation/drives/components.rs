use bevy_ecs::prelude::*;

/// Sensory channel that generated a drive contribution.
/// Extensible enum for future sensory modalities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum DriveSource {
    #[default]
    Vision = 0,
    Sound = 1,
    Scent = 2,
    Seismic = 3,
    Habitat = 4,
}

/// Priority tier for drive processing.
/// Emergency tier has priority override; Motivated tier uses weighted sum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DriveTier {
    Emergency,
    #[default]
    Motivated,
}

/// A single drive contribution from a sensory source.
/// Direction is normalized; magnitude is 0.0-1.0.
#[derive(Clone, Copy, Debug, Default)]
pub struct DriveContribution {
    pub direction: (f32, f32),
    pub magnitude: f32,
}

/// Maximum contributions per category (flee/approach/disperse).
/// 4 slots handles typical scenarios; overflow is silently dropped.
pub const MAX_DRIVE_CONTRIBUTIONS: usize = 4;

/// WARM PATH: Drive contributions from all sources before combination.
/// Written by VisionDriveSystem, read by DriveCombineSystem.
/// Fixed arrays prevent heap allocation at 500K creature scale.
#[derive(Component, Clone, Copy, Default)]
pub struct DriveContributions {
    pub flee: [DriveContribution; MAX_DRIVE_CONTRIBUTIONS],
    pub flee_count: u8,
    pub approach: [DriveContribution; MAX_DRIVE_CONTRIBUTIONS],
    pub approach_count: u8,
    pub disperse: [DriveContribution; MAX_DRIVE_CONTRIBUTIONS],
    pub disperse_count: u8,
}

impl DriveContributions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.flee_count = 0;
        self.approach_count = 0;
        self.disperse_count = 0;
    }

    /// Push flee contribution. Silently ignores if array full (Phase B behavior).
    pub fn push_flee(&mut self, direction: (f32, f32), magnitude: f32) {
        let idx = self.flee_count as usize;
        if idx < MAX_DRIVE_CONTRIBUTIONS {
            self.flee[idx] = DriveContribution {
                direction,
                magnitude,
            };
            self.flee_count += 1;
        }
    }

    /// Push approach contribution. Silently ignores if array full.
    pub fn push_approach(&mut self, direction: (f32, f32), magnitude: f32) {
        let idx = self.approach_count as usize;
        if idx < MAX_DRIVE_CONTRIBUTIONS {
            self.approach[idx] = DriveContribution {
                direction,
                magnitude,
            };
            self.approach_count += 1;
        }
    }

    /// Push disperse contribution. Silently ignores if array full.
    pub fn push_disperse(&mut self, direction: (f32, f32), magnitude: f32) {
        let idx = self.disperse_count as usize;
        if idx < MAX_DRIVE_CONTRIBUTIONS {
            self.disperse[idx] = DriveContribution {
                direction,
                magnitude,
            };
            self.disperse_count += 1;
        }
    }

    /// Iterate flee contributions.
    pub fn iter_flee(&self) -> impl Iterator<Item = &DriveContribution> {
        self.flee[..self.flee_count as usize].iter()
    }

    /// Iterate approach contributions.
    pub fn iter_approach(&self) -> impl Iterator<Item = &DriveContribution> {
        self.approach[..self.approach_count as usize].iter()
    }

    /// Iterate disperse contributions.
    pub fn iter_disperse(&self) -> impl Iterator<Item = &DriveContribution> {
        self.disperse[..self.disperse_count as usize].iter()
    }

    /// Check if any flee contributions exist.
    pub fn has_flee(&self) -> bool {
        self.flee_count > 0
    }

    /// Check if any approach contributions exist.
    pub fn has_approach(&self) -> bool {
        self.approach_count > 0
    }

    /// Check if any disperse contributions exist.
    pub fn has_disperse(&self) -> bool {
        self.disperse_count > 0
    }

    /// Check if any contributions exist.
    pub fn is_empty(&self) -> bool {
        self.flee_count == 0 && self.approach_count == 0 && self.disperse_count == 0
    }
}

/// HOT PATH: Final combined drive output for steering.
/// 8 bytes - read every tick by steering system.
/// At 500K creatures: 4MB cache footprint (vs 114MB if combined with DriveContributions).
#[derive(Component, Clone, Copy, Default)]
pub struct DriveOutput {
    pub combined: (f32, f32),
}

impl DriveOutput {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if drive output is effectively zero (creature at rest).
    pub fn is_zero(&self) -> bool {
        let (x, y) = self.combined;
        x.abs() < 0.001 && y.abs() < 0.001
    }

    /// Get magnitude of combined drive.
    pub fn magnitude(&self) -> f32 {
        let (x, y) = self.combined;
        (x * x + y * y).sqrt()
    }
}

/// DEV-TOOLS ONLY: Simplex triangle visualization data.
/// Shows per-category drive vectors for debug HUD.
#[cfg(feature = "dev-tools")]
#[derive(Component, Clone, Copy, Default)]
pub struct DriveSimplex {
    pub flee: (f32, f32),
    pub approach: (f32, f32),
    pub disperse: (f32, f32),
}

/// Freeze state tracking for desperate escape behavior.
/// When a creature is stuck (drives ≈ 0), tracks duration.
/// After threshold, triggers random escape direction.
#[derive(Component, Clone, Copy, Default)]
pub struct FreezeState {
    pub ticks_frozen: u16,
    pub escape_direction: (f32, f32),
}

impl FreezeState {
    /// Threshold in ticks before desperate escape triggers.
    /// ~4.5 seconds at 22Hz tick rate.
    pub const DESPERATE_THRESHOLD: u16 = 100;

    pub fn new() -> Self {
        Self::default()
    }

    /// Check if creature has been frozen long enough to trigger desperate escape.
    pub fn is_desperate(&self) -> bool {
        self.ticks_frozen >= Self::DESPERATE_THRESHOLD
    }

    /// Increment freeze counter. Sets random escape direction at threshold.
    pub fn tick(&mut self) {
        self.ticks_frozen = self.ticks_frozen.saturating_add(1);
        if self.ticks_frozen == Self::DESPERATE_THRESHOLD {
            use rand::Rng;
            let angle = rand::thread_rng().gen_range(0.0..std::f32::consts::TAU);
            self.escape_direction = (angle.cos(), angle.sin());
        }
    }

    /// Reset freeze state when creature starts moving again.
    pub fn reset(&mut self) {
        self.ticks_frozen = 0;
        self.escape_direction = (0.0, 0.0);
    }
}

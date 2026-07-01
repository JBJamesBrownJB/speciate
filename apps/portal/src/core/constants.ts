export const RENDERING_CONFIG = {
  TARGET_FPS: 90,
  // Viewport is X% of window size (synced to CSS via --viewport-size custom property)
  VIEWPORT_SIZE_RATIO: 0.75,
  VELOCITY_DAMPING: 0.1, // Exponential decay rate for extrapolation (tune: 0.05-0.2)
} as const;

// Derived timing value
export const getTickIntervalMs = (tickRateHz: number): number => 1000 / tickRateHz;

export const WORLD_BOUNDS = {
  MIN_X: -5000,
  MAX_X: 5000,
  MIN_Y: -5000,
  MAX_Y: 5000,
} as const;

export const CAMERA_CONFIG = {
  MIN_ZOOM: 2.0,
  MAX_ZOOM: 200,
  ZOOM_SENSITIVITY: 0.001,
  MAX_ZOOM_SPEED: 2.0,
  PAN_SPEED_BASE: 500,
} as const;

export const SCALE_BAR_CONFIG = {
  TARGET_PIXEL_WIDTH: 120,
  NICE_INTERVALS: [
    1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 20000, 50000,
    100000, 200000, 500000, 1000000,
  ] as readonly number[],
} as const;

export const VIEWPORT_CULLING_CONFIG = {
  MARGIN: 40.0, // World units buffer to prevent edge flickering
} as const;

// Capacity tiering for the creature pipeline. Two deliberate tiers:
// - SEAM_MAX: the hot-buffer cap on the Rust↔Electron seam. MUST mirror
//   electron/bufferLayout.cjs MAX_CREATURES (pinned by bufferLayout.test.ts).
// - EXPECTED_VISIBLE: renderer pre-allocation. Deliveries are viewport-culled,
//   so the renderer sizes GPU/frame buffers for this and grows geometrically
//   if a frame ever exceeds it (no truncation, just a one-off realloc).
export const CREATURE_CAPACITY = {
  SEAM_MAX: 1_000_000,
  EXPECTED_VISIBLE: 200_000,
} as const;

// Display defaults for the two-level spatial grid, matching the engine's
// authoritative values (apps/simulation/src/simulation/spatial/constants.rs:
// CELL_SIZE = 20, L1 = 3x). Telemetry overrides these at runtime; the defaults
// only matter for the frames before the first telemetry arrives.
export const SPATIAL_GRID_CONFIG = {
  L0_CELL_SIZE: 20,
  L1_CELL_SIZE: 60,
} as const;

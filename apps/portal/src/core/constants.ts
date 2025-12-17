export const RENDERING_CONFIG = {
  TARGET_FPS: 90,
  // Viewport is X% of window size (synced to CSS via --viewport-size custom property)
  VIEWPORT_SIZE_RATIO: 0.75,
  VELOCITY_DAMPING: 0.1, // Exponential decay rate for extrapolation (tune: 0.05-0.2)
} as const;

// Derived timing value
export const getTickIntervalMs = (tickRateHz: number): number => 1000 / tickRateHz;

export const WORLD_CONFIG = {
  SIZE: 10000, // World is 2000km × 2000km (-1,000,000 to +1,000,000 meters)
} as const;

export const CAMERA_CONFIG = {
  MIN_ZOOM: 2.0,
  MAX_ZOOM: 200,
  ZOOM_SENSITIVITY: 0.001, // Mouse wheel zoom sensitivity
  MAX_ZOOM_SPEED: 2.0, // Max zoom change per second (logarithmic scale)
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

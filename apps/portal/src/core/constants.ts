export const RENDERING_CONFIG = {
  TARGET_FPS: 90,
  VIEWPORT_SIZE_RATIO: 0.75, // Viewport is 75% of window size
  VELOCITY_DAMPING: 0.1, // Exponential decay rate for extrapolation (tune: 0.05-0.2)
} as const;

export const WORLD_CONFIG = {
  SIZE: 2000000, // World is 2000km × 2000km (-1,000,000 to +1,000,000 meters)
} as const;

export const GRID_CONFIG = {
  SPACING: 1, // Fixed 1m grid spacing
  COLOR: 0x727472, // Gray
  ALPHA: 1.0, // 100% opacity (fully opaque)
  LINE_WIDTH: 0.8, // 0.8 pixel line width
  MIN_ZOOM_FOR_GRID: 20, // Only show grid when zoom >= 20 px/m
} as const;

export const CAMERA_CONFIG = {
  MIN_ZOOM: 0.0005, // Minimum zoom (px/m) - view full 2000km world
  MAX_ZOOM: 400, // Maximum zoom (px/m) - 1 meter = 200 pixels
} as const;

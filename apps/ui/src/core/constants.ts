export const NETWORK_CONFIG = {
  UPDATE_RATE_HZ: 20,
  RECONNECT_DELAY_MS: 3000,
  MAX_RECONNECT_ATTEMPTS: 5,
} as const;

export const RENDERING_CONFIG = {
  TARGET_FPS: 60,
  INTERPOLATION_BUFFER_MS: 100,
  VIEWPORT_PADDING: 100,
} as const;

export const ENTITY_CONFIG = {
  DEFAULT_RADIUS: 10,
  MIN_RADIUS: 5,
  MAX_RADIUS: 50,
  DEFAULT_COLOR: 0x00ff00,
} as const;

export const DIAGNOSTIC_CONFIG = {
  TEXT_STYLE: {
    fontFamily: 'monospace',
    fontSize: 12,
    fill: 0xffffff,
  },
  PADDING: 10,
} as const;

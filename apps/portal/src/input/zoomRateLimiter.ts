/**
 * Wheel-zoom rate limiting in log-zoom space: each event's requested delta is
 * clamped to `maxSpeed` log-units/second of elapsed time since the last
 * applied zoom, so a fast wheel flick can't teleport the zoom level. Returns
 * the multiplicative zoom factor to apply, or null for a no-op event.
 */
export function createZoomRateLimiter(config: {
  sensitivity: number;
  maxSpeed: number;
}): (deltaY: number, nowMs: number) => number | null {
  let lastZoomTime = -Infinity; // first event gets the full budget

  return (deltaY: number, nowMs: number): number | null => {
    const elapsedMs = nowMs - lastZoomTime;
    const maxDelta = (config.maxSpeed * elapsedMs) / 1000;

    let zoomDelta = -deltaY * config.sensitivity;
    const sign = Math.sign(zoomDelta);
    zoomDelta = sign * Math.min(Math.abs(zoomDelta), maxDelta);

    if (zoomDelta === 0) return null;
    lastZoomTime = nowMs;
    return Math.exp(zoomDelta);
  };
}

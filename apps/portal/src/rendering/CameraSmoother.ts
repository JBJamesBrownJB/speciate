/**
 * CameraSmoother — frame-rate-independent exponential easing of the *rendered* camera
 * toward the logic camera.
 *
 * Why: at high population the sim's per-tick CPU burst occasionally delays a render frame.
 * Driving the world transform from an eased pose lets such a frame glide to the correct
 * position rather than lurching the whole world — the same reason interpolated entities stay
 * smooth through timing jitter. The ease is the exact continuous solution sampled at `dt`
 * (`x += (target - x)·(1 - e^(-dt/τ))`), so the result is identical regardless of how the
 * elapsed time is subdivided across frames.
 *
 * Pure (no PixiJS/DOM) — unit-tested.
 */
export class CameraSmoother {
  private _x: number;
  private _y: number;
  private _zoom: number;

  /** @param tau time constant in seconds (larger = softer/slower follow). */
  constructor(x: number, y: number, zoom: number, private readonly tau = 0.045) {
    this._x = x;
    this._y = y;
    this._zoom = zoom;
  }

  /** Ease the rendered pose toward the target over `dt` seconds. `dt <= 0` is a no-op. */
  follow(targetX: number, targetY: number, targetZoom: number, dt: number): void {
    if (dt <= 0) return;
    const k = 1 - Math.exp(-dt / this.tau);
    this._x += (targetX - this._x) * k;
    this._y += (targetY - this._y) * k;
    this._zoom += (targetZoom - this._zoom) * k;
  }

  /** Jump instantly to a pose with no easing (e.g. teleport / initial sync). */
  snap(x: number, y: number, zoom: number): void {
    this._x = x;
    this._y = y;
    this._zoom = zoom;
  }

  get x(): number {
    return this._x;
  }

  get y(): number {
    return this._y;
  }

  get zoom(): number {
    return this._zoom;
  }
}

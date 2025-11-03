import type { Position, InterpolatedState } from '../types/entity';

export class InterpolationCalculator {
  calculatePosition(
    state: InterpolatedState,
    currentTime: number,
    bufferMs: number
  ): Position {
    const timeSinceUpdate = currentTime - state.lastUpdateTime;
    const alpha = Math.min(timeSinceUpdate / bufferMs, 1);

    return {
      x: this.lerp(state.previousPosition.x, state.position.x, alpha),
      y: this.lerp(state.previousPosition.y, state.position.y, alpha),
    };
  }

  calculateOrientation(
    state: InterpolatedState,
    currentTime: number,
    bufferMs: number
  ): number {
    const timeSinceUpdate = currentTime - state.lastUpdateTime;
    const alpha = Math.min(timeSinceUpdate / bufferMs, 1);

    return this.lerpAngle(state.previousOrientation, state.orientation, alpha);
  }

  private lerp(start: number, end: number, alpha: number): number {
    return start + (end - start) * alpha;
  }

  private lerpAngle(start: number, end: number, alpha: number): number {
    let delta = end - start;

    if (delta > Math.PI) delta -= 2 * Math.PI;
    if (delta < -Math.PI) delta += 2 * Math.PI;

    return start + delta * alpha;
  }
}

import type { GameState, CreatureFrameView } from "@/types/GameState";

/**
 * The simulation → renderer routing logic, extracted from main() so its
 * contract is unit-testable (first-frame initialize vs steady-state tick,
 * duplicate suppression, selection tracking). main() only supplies the wiring.
 */
export interface StateUpdateDeps {
  renderer: {
    setTickRate(hz: number): void;
    initializeSoA(buffer: Float32Array, count: number): void;
    onSimulationTickSoA(buffer: Float32Array, count: number): void;
    getLatestSlot(): CreatureFrameView | null;
  };
  selectionManager: {
    updateSelectedFromFrame(frame: CreatureFrameView | null): void;
  };
  changeDetector: {
    shouldUpdate(tick: number, buffer: Float32Array, count: number): boolean;
  };
  /** Called with the visible-creature count on every delivery (HUD). */
  onCreatureCount(count: number): void;
  /** Optional dev-only probe, called with the change verdict per delivery. */
  onDelivery?(stateChanged: boolean): void;
}

export function createStateUpdateHandler(deps: StateUpdateDeps): (state: GameState) => void {
  let isFirstFrame = true;

  return (state: GameState): void => {
    const { buffer, count } = state.soa;
    deps.onCreatureCount(count);

    if (state.tickRateHz && !isNaN(state.tickRateHz)) {
      deps.renderer.setTickRate(state.tickRateHz);
    }

    // New data? Tick identity in push mode; exact compare in the poll fallback.
    const stateChanged = deps.changeDetector.shouldUpdate(state.tick, buffer, count);
    deps.onDelivery?.(stateChanged);

    if (stateChanged) {
      if (isFirstFrame && count > 0) {
        deps.renderer.initializeSoA(buffer, count);
        isFirstFrame = false;
      } else {
        deps.renderer.onSimulationTickSoA(buffer, count);
      }
    }

    // Track the selection into the newest frame (moved or died) — O(1) via
    // the frame's id index.
    deps.selectionManager.updateSelectedFromFrame(deps.renderer.getLatestSlot());
  };
}

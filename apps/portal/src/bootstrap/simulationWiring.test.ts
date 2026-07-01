import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createStateUpdateHandler } from './simulationWiring';
import type { GameState } from '@/types/GameState';

function makeState(overrides?: Partial<GameState>): GameState {
  const soa = overrides?.soa ?? { buffer: new Float32Array(5), count: 1 };
  return {
    protocolVersion: 2,
    tick: 1,
    tickRateHz: 20,
    soa,
    creatures: [],
    ...overrides,
  } as GameState;
}

function makeDeps() {
  const latestSlot = { count: 1 };
  return {
    renderer: {
      setTickRate: vi.fn(),
      initializeSoA: vi.fn(),
      onSimulationTickSoA: vi.fn(),
      getLatestSlot: vi.fn(() => latestSlot as never),
    },
    selectionManager: { updateSelectedFromFrame: vi.fn() },
    changeDetector: { shouldUpdate: vi.fn(() => true) },
    onCreatureCount: vi.fn(),
    onDelivery: vi.fn(),
  };
}

describe('createStateUpdateHandler', () => {
  let deps: ReturnType<typeof makeDeps>;
  let handle: (state: GameState) => void;

  beforeEach(() => {
    deps = makeDeps();
    handle = createStateUpdateHandler(deps);
  });

  it('routes the first non-empty changed frame to initializeSoA, later ones to onSimulationTickSoA', () => {
    handle(makeState({ tick: 1 }));
    expect(deps.renderer.initializeSoA).toHaveBeenCalledTimes(1);
    expect(deps.renderer.onSimulationTickSoA).not.toHaveBeenCalled();

    handle(makeState({ tick: 2 }));
    expect(deps.renderer.initializeSoA).toHaveBeenCalledTimes(1);
    expect(deps.renderer.onSimulationTickSoA).toHaveBeenCalledTimes(1);
  });

  it('an empty first frame does NOT consume the initialize (warm-up waits for creatures)', () => {
    handle(makeState({ soa: { buffer: new Float32Array(0), count: 0 } }));
    expect(deps.renderer.initializeSoA).not.toHaveBeenCalled();

    handle(makeState({ tick: 2 }));
    expect(deps.renderer.initializeSoA).toHaveBeenCalledTimes(1);
  });

  it('skips the renderer push when the change detector says duplicate — but still tracks selection', () => {
    deps.changeDetector.shouldUpdate.mockReturnValue(false);
    handle(makeState());

    expect(deps.renderer.initializeSoA).not.toHaveBeenCalled();
    expect(deps.renderer.onSimulationTickSoA).not.toHaveBeenCalled();
    expect(deps.selectionManager.updateSelectedFromFrame).toHaveBeenCalledTimes(1);
  });

  it('feeds the change detector the tick and the SoA payload', () => {
    const soa = { buffer: new Float32Array(10), count: 2 };
    handle(makeState({ tick: 77, soa }));
    expect(deps.changeDetector.shouldUpdate).toHaveBeenCalledWith(77, soa.buffer, 2);
  });

  it('reports the creature count on every delivery', () => {
    handle(makeState({ soa: { buffer: new Float32Array(15), count: 3 } }));
    expect(deps.onCreatureCount).toHaveBeenCalledWith(3);
  });

  it('forwards a valid tick rate to the renderer, but never NaN/0', () => {
    handle(makeState({ tickRateHz: 20 }));
    expect(deps.renderer.setTickRate).toHaveBeenCalledWith(20);

    deps.renderer.setTickRate.mockClear();
    handle(makeState({ tickRateHz: NaN }));
    handle(makeState({ tickRateHz: 0 }));
    expect(deps.renderer.setTickRate).not.toHaveBeenCalled();
  });

  it('invokes the diagnostics hook with the change verdict', () => {
    handle(makeState());
    expect(deps.onDelivery).toHaveBeenCalledWith(true);

    deps.changeDetector.shouldUpdate.mockReturnValue(false);
    handle(makeState());
    expect(deps.onDelivery).toHaveBeenLastCalledWith(false);
  });
});

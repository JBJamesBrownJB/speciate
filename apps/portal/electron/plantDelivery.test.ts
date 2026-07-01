import { describe, it, expect, vi } from 'vitest';
// @ts-ignore - plain CJS module under test; types not needed for this spec
import { createPlantDelivery } from './plantDelivery.cjs';

function makeHarness(opts: { shuttingDown?: boolean; destroyed?: boolean; engine?: unknown } = {}) {
  const sent: Array<{ channel: string; data: unknown }> = [];
  const plantBuf = new Float32Array([1, 2, 2, 0.75, 3]);
  const engine = { getPlantBuffer: vi.fn(() => plantBuf) };
  const win = {
    isDestroyed: () => opts.destroyed ?? false,
    webContents: { send: (channel: string, data: unknown) => sent.push({ channel, data }) },
  };
  const deliverPlants = createPlantDelivery({
    getEngine: () => ('engine' in opts ? opts.engine : engine),
    getMainWindow: () => win,
    isShuttingDown: () => opts.shuttingDown ?? false,
  });
  return { deliverPlants, sent, engine, plantBuf };
}

describe('createPlantDelivery', () => {
  it('sends the plant buffer values on plant-buffer-update', () => {
    const { deliverPlants, sent, plantBuf } = makeHarness();
    deliverPlants();

    expect(sent).toHaveLength(1);
    expect(sent[0].channel).toBe('plant-buffer-update');
    // Data integrity: values, not just shape (density 0.75, type 3).
    expect(Array.from(sent[0].data as Float32Array)).toEqual(Array.from(plantBuf));
  });

  it('bails without touching the engine when shutting down', () => {
    const { deliverPlants, sent, engine } = makeHarness({ shuttingDown: true });
    deliverPlants();
    expect(engine.getPlantBuffer).not.toHaveBeenCalled();
    expect(sent).toHaveLength(0);
  });

  it('bails when the engine is gone', () => {
    const { deliverPlants, sent } = makeHarness({ engine: null });
    deliverPlants();
    expect(sent).toHaveLength(0);
  });

  it('does not send to a destroyed window', () => {
    const { deliverPlants, sent } = makeHarness({ destroyed: true });
    deliverPlants();
    expect(sent).toHaveLength(0);
  });

  it('catches engine errors instead of throwing into the timer', () => {
    const engine = { getPlantBuffer: vi.fn(() => { throw new Error('boom'); }) };
    const { deliverPlants } = makeHarness({ engine });
    const errors = vi.spyOn(console, 'error').mockImplementation(() => {});
    expect(() => deliverPlants()).not.toThrow();
    expect(errors).toHaveBeenCalled();
    errors.mockRestore();
  });
});

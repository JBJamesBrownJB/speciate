import { describe, it, expect, vi } from 'vitest';
// @ts-ignore - plain CJS module under test; types not needed for this spec
import { createFrameDelivery } from './frameDelivery.cjs';

interface HarnessOpts {
  count?: number;
  shuttingDown?: boolean;
  destroyed?: boolean;
  disableBufferCalls?: boolean;
  noPerception?: boolean;
  engine?: unknown; // pass null to simulate a torn-down engine
}

function makeHarness(opts: HarnessOpts = {}) {
  const sent: Array<{ channel: string; data: any }> = [];
  const win = {
    isDestroyed: () => opts.destroyed ?? false,
    webContents: { send: (channel: string, data: any) => sent.push({ channel, data }) },
  };
  const engine = {
    fillBuffer: vi.fn(() => opts.count ?? 3),
    fillPerceptionDebug: opts.noPerception ? undefined : vi.fn(),
  };
  const deliverFrame = createFrameDelivery({
    getEngine: () => ('engine' in opts ? opts.engine : engine),
    getMainWindow: () => win,
    isShuttingDown: () => opts.shuttingDown ?? false,
    creatureBuffer: new Float32Array(100),
    perceptionBuffer: new Float32Array(10),
    floatsPerCreature: 5,
    disableBufferCalls: opts.disableBufferCalls ?? false,
  });
  return { deliverFrame, sent, engine, win };
}

describe('deliverFrame (push-on-swap)', () => {
  it('bails without touching the engine when shutting down (teardown-safe)', () => {
    const { deliverFrame, engine, sent } = makeHarness({ shuttingDown: true });
    deliverFrame(42);
    expect(engine.fillBuffer).not.toHaveBeenCalled();
    expect(sent).toHaveLength(0);
  });

  it('bails when the engine has been nulled', () => {
    const { deliverFrame, sent } = makeHarness({ engine: null });
    deliverFrame(42);
    expect(sent).toHaveLength(0);
  });

  it('sends napi-buffer-update carrying the tick and the count-sliced buffer', () => {
    const { deliverFrame, sent, engine } = makeHarness({ count: 3 });
    deliverFrame(42);
    expect(engine.fillBuffer).toHaveBeenCalledOnce();
    const msg = sent.find((s) => s.channel === 'napi-buffer-update');
    expect(msg).toBeDefined();
    expect(msg!.data.tick).toBe(42);
    expect(msg!.data.creatureCount).toBe(3);
    // Sliced to count*floats (3*5=15), NOT the full 100-element backing buffer.
    expect(msg!.data.buffer).toHaveLength(15);
  });

  it('sends perception-debug-update when the method exists', () => {
    const { deliverFrame, sent } = makeHarness();
    deliverFrame(1);
    expect(sent.some((s) => s.channel === 'perception-debug-update')).toBe(true);
  });

  it('skips all sends when buffer calls are disabled', () => {
    const { deliverFrame, sent, engine } = makeHarness({ disableBufferCalls: true });
    deliverFrame(1);
    expect(engine.fillBuffer).not.toHaveBeenCalled();
    expect(sent).toHaveLength(0);
  });

  it('does not send to a destroyed window', () => {
    const { deliverFrame, sent } = makeHarness({ destroyed: true });
    deliverFrame(1);
    expect(sent).toHaveLength(0);
  });
});

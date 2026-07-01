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
  /** Writes the perception buffer on each fill (defaults to has_data = 1). */
  fillPerception?: (buf: Float32Array) => void;
}

function makeHarness(opts: HarnessOpts = {}) {
  const sent: Array<{ channel: string; data: any }> = [];
  const win = {
    isDestroyed: () => opts.destroyed ?? false,
    webContents: {
      send: (channel: string, data: any) =>
        // Mirror Electron's serialize-on-send: snapshot typed arrays so later
        // mutations of the reused perception buffer don't rewrite history.
        sent.push({ channel, data: data instanceof Float32Array ? data.slice() : data }),
    },
  };
  const engine = {
    fillBuffer: vi.fn(() => opts.count ?? 3),
    fillPerceptionDebug: opts.noPerception
      ? undefined
      : vi.fn((buf: Float32Array) => (opts.fillPerception ?? ((b: Float32Array) => { b[0] = 1; }))(buf)),
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
    // slice() not subarray(): the sent view must own a right-sized ArrayBuffer,
    // not share the full backing one (the whole-buffer structured-clone trap).
    expect(msg!.data.buffer.buffer.byteLength).toBe(15 * 4);
  });

  it('sends perception-debug-update every frame while a creature is selected', () => {
    const { deliverFrame, sent } = makeHarness();
    deliverFrame(1);
    deliverFrame(2);
    expect(sent.filter((s) => s.channel === 'perception-debug-update')).toHaveLength(2);
  });

  it('on deselect sends ONE trailing empty buffer then goes quiet until reselected', () => {
    const hasData = { value: 1 };
    const { deliverFrame, sent } = makeHarness({
      fillPerception: (buf) => { buf[0] = hasData.value; },
    });

    deliverFrame(1); // selected → send
    hasData.value = 0;
    deliverFrame(2); // deselect transition → one clearing send
    deliverFrame(3); // steady-state empty → silent
    deliverFrame(4); // steady-state empty → silent
    hasData.value = 1;
    deliverFrame(5); // reselected → send resumes

    const perception = sent.filter((s) => s.channel === 'perception-debug-update');
    expect(perception).toHaveLength(3);
    expect(perception[1].data[0]).toBe(0); // the clearing send
    expect(perception[2].data[0]).toBe(1);
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

import { describe, it, expect, vi } from 'vitest';
import { initRendererWithFallback } from './rendererFallback';

type InitFn = (options: Record<string, unknown>) => Promise<void>;

interface FakeApp {
  init: InitFn & { mock: { calls: unknown[][] } };
}

function makeFactory(behaviours: Array<'ok' | 'fail'>) {
  const apps: FakeApp[] = [];
  const createApp = () => {
    const behaviour = behaviours[apps.length] ?? 'ok';
    const init = vi.fn(() =>
      behaviour === 'ok' ? Promise.resolve() : Promise.reject(new Error(`${behaviour} init`))
    ) as unknown as FakeApp['init'];
    const app: FakeApp = { init };
    apps.push(app);
    return app;
  };
  return { createApp, apps };
}

const BASE = { width: 800, height: 600 };

describe('initRendererWithFallback', () => {
  it('uses WebGL on the happy path and creates exactly one app', async () => {
    const { createApp, apps } = makeFactory(['ok']);

    const result = await initRendererWithFallback(createApp, BASE);

    expect(apps).toHaveLength(1);
    expect(result.app).toBe(apps[0]);
    expect(result.renderer).toBe('webgl');
    expect(apps[0].init).toHaveBeenCalledWith(
      expect.objectContaining({ ...BASE, preference: 'webgl' })
    );
  });

  it('falls back to WebGPU on a FRESH Application when WebGL init throws', async () => {
    const { createApp, apps } = makeFactory(['fail', 'ok']);
    vi.spyOn(console, 'error').mockImplementation(() => {});
    vi.spyOn(console, 'warn').mockImplementation(() => {});

    const result = await initRendererWithFallback(createApp, BASE);

    // Never re-init() the half-initialized first instance.
    expect(apps).toHaveLength(2);
    expect(result.app).toBe(apps[1]);
    expect(result.renderer).toBe('webgpu');
    expect(apps[1].init).toHaveBeenCalledWith(
      expect.objectContaining({ ...BASE, preference: 'webgpu' })
    );
    vi.restoreAllMocks();
  });

  it('logs the fallback accurately (WebGPU, not the old bogus Canvas2D message)', async () => {
    const { createApp } = makeFactory(['fail', 'ok']);
    const errors = vi.spyOn(console, 'error').mockImplementation(() => {});
    const warns = vi.spyOn(console, 'warn').mockImplementation(() => {});

    await initRendererWithFallback(createApp, BASE);

    const logged = [...errors.mock.calls, ...warns.mock.calls].flat().join(' ');
    expect(logged).toContain('WebGPU');
    expect(logged).not.toContain('Canvas2D');
    vi.restoreAllMocks();
  });

  it('rejects with the fallback error when both renderers fail', async () => {
    const { createApp } = makeFactory(['fail', 'fail']);
    vi.spyOn(console, 'error').mockImplementation(() => {});

    await expect(initRendererWithFallback(createApp, BASE)).rejects.toThrow('fail init');
    vi.restoreAllMocks();
  });
});

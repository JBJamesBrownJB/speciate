/** The subset of PixiJS Application this policy needs — injectable for tests. */
export interface InitializableApp {
  init(options: Record<string, unknown>): Promise<void>;
}

/**
 * Renderer init policy: WebGL first (the proven path for the instanced-mesh
 * pipeline), falling back to WebGPU on a FRESH Application if WebGL init
 * throws. Re-init()ing a half-initialized Application is undefined behaviour,
 * so the fallback always constructs a new one. If both fail, the error
 * propagates to the caller's failure page.
 */
export async function initRendererWithFallback<T extends InitializableApp>(
  createApp: () => T,
  baseOptions: Record<string, unknown>
): Promise<{ app: T; renderer: 'webgl' | 'webgpu' }> {
  const primary = createApp();
  try {
    await primary.init({
      ...baseOptions,
      preference: 'webgl',
      powerPreference: 'low-power',
      failIfMajorPerformanceCaveat: false,
    });
    return { app: primary, renderer: 'webgl' };
  } catch (error) {
    console.error('[PixiJS] WebGL initialization failed, retrying with WebGPU:', error);
    const fallback = createApp();
    await fallback.init({
      ...baseOptions,
      preference: 'webgpu',
    });
    console.warn('[PixiJS] ⚠️ Running on the WebGPU fallback renderer');
    return { app: fallback, renderer: 'webgpu' };
  }
}

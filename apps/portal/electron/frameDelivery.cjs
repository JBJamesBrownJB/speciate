/**
 * Frame delivery for push-on-swap — extracted from napi-main.cjs so its contract is
 * unit-testable (napi-main is otherwise a monolith of Electron globals).
 *
 * `createFrameDelivery(deps)` returns a `deliverFrame(tick)` that ships ONE buffer
 * swap's worth of creature positions (+ perception) to the renderer. All environment
 * is injected, so tests drive it with mocks. Invariants it must hold:
 *  - bail without touching the engine when shutting down / engine gone (teardown-safe)
 *  - send 'napi-buffer-update' carrying the sim `tick` + the count-sliced buffer
 *  - use slice() not subarray() (subarray serializes the whole 10MB ArrayBuffer)
 *  - reuse the injected persistent buffers (no per-frame allocation)
 */
function createFrameDelivery(deps) {
  const {
    getEngine, // () => simulationEngine | null
    getMainWindow, // () => BrowserWindow | null
    isShuttingDown, // () => boolean
    creatureBuffer,
    perceptionBuffer,
    floatsPerCreature,
    disableBufferCalls = false,
    onMemorySample, // optional: called once per 200 frames (diagnostics)
  } = deps;

  let frameCount = 0;

  return function deliverFrame(tick) {
    const engine = getEngine();
    if (!engine || isShuttingDown()) return;

    frameCount++;
    if (onMemorySample && frameCount % 200 === 0) onMemorySample();

    try {
      if (disableBufferCalls) return;

      // fillBuffer() copies positions into our JS-owned buffer (zero-alloc); returns count.
      const creatureCount = engine.fillBuffer(creatureBuffer);

      // slice() (NOT subarray) — subarray would serialize the whole backing ArrayBuffer.
      const usedSize = creatureCount * floatsPerCreature;
      const buffer = creatureBuffer.slice(0, usedSize);

      const win = getMainWindow();
      const alive = !!win && !win.isDestroyed();
      if (alive) {
        win.webContents.send('napi-buffer-update', { buffer, creatureCount, tick });
      }

      // Perception debug buffer (dev-tools only; frontend tracks selection state).
      if (engine.fillPerceptionDebug) {
        engine.fillPerceptionDebug(perceptionBuffer);
        if (alive) {
          win.webContents.send('perception-debug-update', perceptionBuffer);
        }
      }
    } catch (error) {
      console.error('[Electron NAPI] deliverFrame error:', error);
    }
  };
}

module.exports = { createFrameDelivery };

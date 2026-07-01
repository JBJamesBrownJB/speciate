/**
 * Plant-buffer delivery — one injectable function shared by the 2s snapshot
 * interval and the spawn-plant immediate push (previously two divergent
 * copies; the spawn-plant one existed only because the original was scoped
 * inside startSimulation). Same DI pattern as frameDelivery.cjs.
 */
function createPlantDelivery(deps) {
  const { getEngine, getMainWindow, isShuttingDown } = deps;

  return function deliverPlants() {
    const engine = getEngine();
    if (!engine || isShuttingDown()) return;
    try {
      const plantBuf = engine.getPlantBuffer();
      const win = getMainWindow();
      if (win && !win.isDestroyed()) {
        win.webContents.send('plant-buffer-update', plantBuf);
      }
    } catch (error) {
      console.error('[Electron NAPI] Plant buffer error:', error);
    }
  };
}

module.exports = { createPlantDelivery };

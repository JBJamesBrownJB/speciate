import { Application, Container } from "pixi.js";
import { SpriteProvider } from "@/rendering/SpriteProvider";
import { Camera } from "@/domain/Camera";
import { Viewport } from "@/domain/Viewport";
import { RENDERING_CONFIG, CAMERA_CONFIG } from "@/core/constants";
import { createIPCClient, type IPCClient } from "@/infrastructure/ipc";
import { PerformanceMetrics } from "@/core/PerformanceMetrics";
import { FPSSparkline } from "@/ui/FPSSparkline";
import { ScaleBarManager } from "@/ui/ScaleBarManager";
import { HUDManager } from "@/ui/HUDManager";
import { InterpolatedCreatureRenderer } from "@/rendering/InterpolatedCreatureRenderer";
import { ChangeDetector } from "@/core/ChangeDetection";
import { SelectionManager } from "@/systems/SelectionManager";
import {
  OverlayManager,
  SelectionHighlight,
  PerceptionOverlay,
  SpatialGridOverlay,
  ForceOverlay,
} from "@/rendering/overlays";
import { CreatureInfoPanel } from "@/ui/CreatureInfoPanel";
import type { CreatureData } from "@/types/GameState";

function updateContainerSize(
  container: HTMLElement,
  width: number,
  height: number
): void {
  container.style.width = `${width}px`;
  container.style.height = `${height}px`;
}

async function main(): Promise<void> {
  try {
    const viewportSizePercent = (RENDERING_CONFIG.VIEWPORT_SIZE_RATIO * 100).toString();
    document.documentElement.style.setProperty('--viewport-size', viewportSizePercent);

    const viewportWidth = Math.floor(
      window.innerWidth * RENDERING_CONFIG.VIEWPORT_SIZE_RATIO
    );
    const viewportHeight = Math.floor(
      window.innerHeight * RENDERING_CONFIG.VIEWPORT_SIZE_RATIO
    );

    const app = new Application();

    try {
      await app.init({
        width: viewportWidth,
        height: viewportHeight,
        backgroundColor: 0x000000,
        resolution: window.devicePixelRatio || 1,
        autoDensity: true,
        preference: 'webgl',
        powerPreference: 'low-power',
        failIfMajorPerformanceCaveat: false,
        antialias: false,
      });
    } catch (error) {
      console.error('[PixiJS] WebGL initialization failed:', error);

      await app.init({
        width: viewportWidth,
        height: viewportHeight,
        backgroundColor: 0x000000,
        resolution: window.devicePixelRatio || 1,
        autoDensity: true,
        preference: 'webgpu',
        antialias: false,
      });

      console.warn('[PixiJS] ⚠️ Running in Canvas2D mode (software rendering, expect 30-60 FPS)');
    }

    app.ticker.maxFPS = 0;

    let rafSamples: number[] = [];
    let rafLastTime = performance.now();
    let rafCount = 0;

    const measureRefreshRate = () => {
      const now = performance.now();
      const delta = now - rafLastTime;
      if (delta > 0) {
        rafSamples.push(1000 / delta);
      }
      rafLastTime = now;
      rafCount++;

      if (rafCount < 20) {
        requestAnimationFrame(measureRefreshRate);
      }
    };
    requestAnimationFrame(measureRefreshRate);

    const container = document.getElementById("canvas-container");
    if (!container) throw new Error("canvas-container not found");
    container.classList.add('glow-active');
    container.appendChild(app.canvas);

    updateContainerSize(container, viewportWidth, viewportHeight);

    const camera = new Camera(0, 0, 10);
    const viewport = new Viewport(viewportWidth, viewportHeight);

    const spriteProvider = new SpriteProvider();
    await spriteProvider.init();

    const texture = spriteProvider.getCreatureTexture();

    const worldContainer = new Container();
    app.stage.addChild(worldContainer);

    camera.applyTransform(worldContainer, viewportWidth, viewportHeight);

    const scaleBarManager = new ScaleBarManager("scale-line", "scale-label");
    scaleBarManager.update(camera.zoom);

    // Selection system
    const selectionManager = new SelectionManager();
    const creatureInfoPanel = new CreatureInfoPanel(document.body);
    let latestCreatures: CreatureData[] = [];

    // Overlay system
    const overlayManager = new OverlayManager();
    const selectionHighlight = new SelectionHighlight(worldContainer);
    const perceptionOverlay = new PerceptionOverlay(worldContainer);
    const spatialGridOverlay = new SpatialGridOverlay(worldContainer);
    const forceOverlay = new ForceOverlay(worldContainer);

    overlayManager.register(selectionHighlight);
    overlayManager.register(perceptionOverlay);
    overlayManager.register(spatialGridOverlay);
    overlayManager.register(forceOverlay);
    overlayManager.enableKeyboardShortcuts();

    // Wire up selection events
    selectionManager.on('creature-selected', (creature) => {
      if (creature) {
        selectionHighlight.showAt(creature.x, creature.y, creature.size / 2);
        creatureInfoPanel.show(creature);
      }
    });

    selectionManager.on('creature-deselected', () => {
      selectionHighlight.hide();
      perceptionOverlay.clear();
      creatureInfoPanel.hide();
    });

    // Click handler for creature selection
    const CLICK_RADIUS = 15; // World units for click detection

    container.addEventListener('click', (event: MouseEvent) => {
      const rect = container.getBoundingClientRect();
      const screenX = event.clientX - rect.left;
      const screenY = event.clientY - rect.top;

      // Convert screen to world coordinates
      // Account for camera being centered in viewport
      const worldPos = camera.screenToWorld(
        screenX - viewportWidth / 2,
        screenY - viewportHeight / 2
      );

      // Find nearest creature
      const nearest = selectionManager.findNearestCreature(
        latestCreatures,
        worldPos.x,
        worldPos.y,
        CLICK_RADIUS
      );

      if (nearest) {
        selectionManager.selectCreature(nearest);
        if (ipcClient) {
          ipcClient.selectCreatureDebug(nearest.id);
        }
      } else {
        selectionManager.deselect();
        if (ipcClient) {
          ipcClient.selectCreatureDebug(null);
        }
      }
    });

    const creatureRenderer = new InterpolatedCreatureRenderer(texture, 200000);
    worldContainer.addChild(creatureRenderer.getMesh());

    const fpsSparkline = new FPSSparkline("fps-sparkline");
    const hudManager = new HUDManager(
      {
        fpsValue: "fps-value",
        tickRateValue: "tick-rate-value",
        creatureCount: "creature-count",
        zoomValue: "zoom-value",
      },
      fpsSparkline
    );

    let lastFrameTime = performance.now();
    let currentCreatureCount = 0;

    const perfMetrics = new PerformanceMetrics(RENDERING_CONFIG.TARGET_FPS);

    let isFirstFrame = true;
    const changeDetector = new ChangeDetector();

    const ipcClient: IPCClient | null = createIPCClient();

    if (ipcClient) {
      await ipcClient.connect();

      // Handle simulation tick updates via IPC callback (fires when new data arrives)
      ipcClient.onStateUpdate((state) => {
        const creatures = state.creatures;
        currentCreatureCount = creatures.length;
        latestCreatures = creatures;

        // Update tick rate when telemetry provides it
        if (state.tickRateHz && !isNaN(state.tickRateHz)) {
          creatureRenderer.setTickRate(state.tickRateHz);
        }

        // Update selection tracking (creature may have moved or died)
        selectionManager.updateSelectedFromBuffer(creatures);

        // Detect if state changed (count or positions changed)
        const stateChanged = changeDetector.shouldUpdate(creatures);

        if (stateChanged) {

          const spriteUpdateStart = performance.now();
          if (creatures.length > 0) {
            if (isFirstFrame) {
              creatureRenderer.initialize(creatures);
              isFirstFrame = false;
            } else {
              creatureRenderer.onSimulationTick(creatures);
            }
          } else {
            creatureRenderer.onSimulationTick([]);
          }
          const spriteUpdateEnd = performance.now();
          perfMetrics.recordSpriteUpdateTime(spriteUpdateEnd - spriteUpdateStart);
        }
      });

      // Handle perception debug buffer updates (every tick - smooth visualization)
      ipcClient.onPerceptionDebugUpdate((debugData) => {
        // Only update overlay if a creature is selected (prevents stale data race)
        if (debugData && selectionManager.hasSelection()) {
          perceptionOverlay.update(debugData);
          creatureInfoPanel.updateDebugData(debugData);
          // Update spatial grid overlay with queried + checked cells
          if (debugData.queriedCells && debugData.checkedCells && debugData.creatureCell) {
            spatialGridOverlay.updateQueriedCells(
              debugData.queriedCells,
              debugData.checkedCells,
              debugData.creatureCell
            );
          }
          // Update force overlay with acceleration from perception debug data
          const selected = selectionManager.getSelected();
          if (selected) {
            forceOverlay.update({
              x: debugData.x,
              y: debugData.y,
              radius: selected.size / 2,
              ax: debugData.ax,
              ay: debugData.ay,
            });
          }
        } else {
          perceptionOverlay.clear();
          creatureInfoPanel.updateDebugData(null);
          spatialGridOverlay.clearQueriedCells();
          forceOverlay.update(undefined);
        }
      });

      // Handle telemetry updates (cell size and bounds for grid overlay)
      ipcClient.onTelemetryUpdate((telemetry) => {
        if (telemetry.spatialGridCellSize) {
          spatialGridOverlay.setCellSize(telemetry.spatialGridCellSize);
        }
        // Update grid bounds from actual spatial grid data
        if (
          telemetry.spatialGridMinX !== undefined &&
          telemetry.spatialGridMaxX !== undefined &&
          telemetry.spatialGridMinY !== undefined &&
          telemetry.spatialGridMaxY !== undefined
        ) {
          spatialGridOverlay.setBounds(
            telemetry.spatialGridMinX,
            telemetry.spatialGridMaxX,
            telemetry.spatialGridMinY,
            telemetry.spatialGridMaxY
          );
        }
      });

      window.addEventListener("beforeunload", async () => {
        await ipcClient.disconnect();
      });
    }

    // Render loop - only handles rendering, not simulation updates
    app.ticker.add(() => {
      const frameStart = performance.now();
      const deltaMs = frameStart - lastFrameTime;
      const fps = Math.round(1000 / deltaMs);

      perfMetrics.recordFrameTime(deltaMs);

      // Update HUD with cached values
      const state = ipcClient?.getLatestState();
      if (state) {
        hudManager.updateTickRate(state.tickRateHz || 0);
        hudManager.updateCreatureCount(currentCreatureCount);
        hudManager.updateZoom(camera.zoom);
      }

      // Render with interpolation every frame
      creatureRenderer.render(
        deltaMs,
        camera.x,
        camera.y,
        camera.zoom,
        viewportWidth,
        viewportHeight
      );

      // Update spatial grid overlay (only redraws when visible)
      spatialGridOverlay.update(
        camera.x,
        camera.y,
        camera.zoom,
        viewportWidth,
        viewportHeight
      );

      // Update selection highlight animation and position
      const selected = selectionManager.getSelected();
      if (selected) {
        selectionHighlight.updatePosition(selected.x, selected.y);
        selectionHighlight.update(deltaMs);
        creatureInfoPanel.update(selected);
        // Force overlay is updated in onPerceptionDebugUpdate callback
      }

      hudManager.updateFPS(fps);

      lastFrameTime = frameStart;
    });

    document.title = "✅ Simulation Viewer - Live";

    window.addEventListener("resize", () => {
      const newWidth = Math.floor(
        window.innerWidth * RENDERING_CONFIG.VIEWPORT_SIZE_RATIO
      );
      const newHeight = Math.floor(
        window.innerHeight * RENDERING_CONFIG.VIEWPORT_SIZE_RATIO
      );

      updateContainerSize(container, newWidth, newHeight);
      app.renderer.resize(newWidth, newHeight);
      viewport.resize(newWidth, newHeight);
      camera.applyTransform(worldContainer, newWidth, newHeight);
      scaleBarManager.update(camera.zoom);
    });

    window.addEventListener(
      "wheel",
      (event: WheelEvent) => {
        event.preventDefault();

        const zoomFactor = 1 - event.deltaY * CAMERA_CONFIG.ZOOM_SENSITIVITY;

        camera.adjustZoom(zoomFactor);
        camera.applyTransform(worldContainer, viewport.width, viewport.height);
        scaleBarManager.update(camera.zoom);
      },
      { passive: false }
    );
  } catch (error) {
    console.error("[Portal] ❌ Failed to initialize:", error);
    document.title = "❌ Failed";
    document.body.innerHTML = `<div style="color: white; padding: 20px; font-family: monospace;">
      <h1>Failed to load</h1>
      <pre>${error}</pre>
    </div>`;
  }
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => main());
} else {
  main();
}

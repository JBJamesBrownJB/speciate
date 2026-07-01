// Side-effect import: swaps Pixi's shader/uniform generation to eval-free polyfills.
// Required so rendering works under a no-`unsafe-eval` environment — i.e. the packaged
// (file://) build and any strict CSP. MUST run before `new Application().init()`.
import "pixi.js/unsafe-eval";
import { Application, Container } from "pixi.js";
import { SpriteProvider } from "@/rendering/SpriteProvider";
import { Camera } from "@/domain/Camera";
import { CameraController } from "@/domain/CameraController";
import { createWorldBounds } from "@/domain/WorldBounds";
import { conspicuousness } from "@/domain/conspicuousness";
import { InputManager } from "@/input";
import {
  RENDERING_CONFIG,
  CAMERA_CONFIG,
  VIEWPORT_CULLING_CONFIG,
  WORLD_BOUNDS,
  CREATURE_CAPACITY,
} from "@/core/constants";
import { createIPCClient, type IPCClient } from "@/infrastructure/ipc";
import { FPSSparkline } from "@/ui/FPSSparkline";
import { ScaleBarManager } from "@/ui/ScaleBarManager";
import { HUDManager } from "@/ui/HUDManager";
import { InterpolatedCreatureRenderer } from "@/rendering/InterpolatedCreatureRenderer";
import { initRendererWithFallback } from "@/rendering/rendererFallback";
import { PlantRenderer } from "@/rendering/PlantRenderer";
import { interpDiag } from "@/rendering/InterpolationDiagnostics";
import { CameraSmoother } from "@/rendering/CameraSmoother";
import { ChangeDetector } from "@/core/ChangeDetection";
import { SelectionManager } from "@/systems/SelectionManager";
import {
  OverlayManager,
  SelectionHighlight,
  PerceptionOverlay,
  SpatialGridOverlay,
  ForceOverlay,
} from "@/rendering/overlays";
import { Minimap } from "@/rendering/minimap";
import { CreatureInfoPanel } from "@/ui/CreatureInfoPanel";
import { PauseControl } from "@/ui/PauseControl";
import { TimeScaleControl } from "@/ui/TimeScaleControl";
import { ToolsPanel } from "@/ui/ToolsPanel";
import { InteractionManager } from "@/interaction";
import { GridMode, P0_CELL_SIZE } from "@/rendering/overlays/SpatialGridOverlay";

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

    let viewportWidth = Math.floor(
      window.innerWidth * RENDERING_CONFIG.VIEWPORT_SIZE_RATIO
    );
    let viewportHeight = Math.floor(
      window.innerHeight * RENDERING_CONFIG.VIEWPORT_SIZE_RATIO
    );

    const { app } = await initRendererWithFallback(() => new Application(), {
      width: viewportWidth,
      height: viewportHeight,
      backgroundColor: 0x000000,
      resolution: window.devicePixelRatio || 1,
      autoDensity: true,
      antialias: false,
    });

    // Render at the display's native refresh (0 = uncapped). A maxFPS below the monitor's
    // refresh skips frames unevenly, producing beat-pattern stutter; vsync already bounds us.
    app.ticker.maxFPS = 0;

    const container = document.getElementById("canvas-container");
    if (!container) throw new Error("canvas-container not found");
    container.classList.add('glow-active');
    container.appendChild(app.canvas);

    updateContainerSize(container, viewportWidth, viewportHeight);

    const camera = new Camera(0, 0, 10);
    camera.setWorldBounds(createWorldBounds(
      WORLD_BOUNDS.MIN_X,
      WORLD_BOUNDS.MAX_X,
      WORLD_BOUNDS.MIN_Y,
      WORLD_BOUNDS.MAX_Y
    ));
    camera.setViewportSize(viewportWidth, viewportHeight);
    const inputManager = new InputManager();
    const cameraController = new CameraController(camera, inputManager);

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
    const creatureInfoPanel = new CreatureInfoPanel(document.body, {
      showDebugInfo: import.meta.env.DEV,
    });

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
    // Debug-overlay shortcuts (G/P/F) only exist in dev builds; the minimap's
    // 'm' (a game overlay) is always available.
    overlayManager.enableKeyboardShortcuts({ includeDevToolsOverlays: import.meta.env.DEV });

    // Minimap
    const minimap = new Minimap(
      app.stage,
      createWorldBounds(
        WORLD_BOUNDS.MIN_X,
        WORLD_BOUNDS.MAX_X,
        WORLD_BOUNDS.MIN_Y,
        WORLD_BOUNDS.MAX_Y
      ),
      180
    );
    overlayManager.register(minimap);

    minimap.onMinimapClick = (worldX, worldY) => {
      camera.centerOn(worldX, worldY);
    };

    const updateMinimapPosition = () => {
      minimap.setPosition(viewportWidth - 180 - 20, viewportHeight - 180 - 20);
    };
    updateMinimapPosition();

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

    // IPC client (created early for InteractionManager)
    const ipcClient: IPCClient | null = createIPCClient();

    // Interaction manager (handles clicks and hover using PixiJS events).
    // Click hit-testing scans the renderer's newest SoA frame directly.
    const interactionManager = new InteractionManager({
      worldContainer,
      gridOverlay: spatialGridOverlay,
      selectionManager,
      ipcClient,
      getFrame: () => creatureRenderer.getLatestSlot(),
    });

    // Tools panel — manages active tool state and auto-switches grid mode
    const toolsPanel = new ToolsPanel();
    let savedGridMode: GridMode = GridMode.Off;
    toolsPanel.onToolChange = (tool) => {
      if (tool === 'plant') {
        savedGridMode = spatialGridOverlay.getMode();
        spatialGridOverlay.setMode(GridMode.P0);
      } else {
        spatialGridOverlay.setMode(savedGridMode);
      }
      interactionManager.setActiveTool(tool);
    };

    // Plant renderer — added to worldContainer BEFORE creatures so it renders underneath.
    const plantRenderer = new PlantRenderer(worldContainer);

    // Subscribe to plant snapshot updates from Electron main (push every ~2s).
    window.electron?.onPlantBufferUpdate?.((buf: Float32Array) => {
      plantRenderer.updateFromBuffer(buf);
      spatialGridOverlay.updateP0Cells(buf);
    });

    const creatureRenderer = new InterpolatedCreatureRenderer(
      texture,
      CREATURE_CAPACITY.EXPECTED_VISIBLE
    );
    worldContainer.addChild(creatureRenderer.getMesh());

    const fpsSparkline = new FPSSparkline("fps-sparkline");
    const hudManager = new HUDManager(
      {
        fpsValue: "fps-value",
        tickRateValue: "tick-rate-value",
        creatureWorldCount: "creature-world-count",
        creatureScreenCount: "creature-screen-count",
        plantWorldCount: "plant-world-count",
        plantScreenCount: "plant-screen-count",
        zoomValue: "zoom-value",
      },
      fpsSparkline
    );

    let lastFrameTime = performance.now();
    let currentCreatureCount = 0;

    let isFirstFrame = true;
    const changeDetector = new ChangeDetector();

    // Pause control
    const pauseControl = new PauseControl({
      buttonId: 'pause-button',
      onPauseChange: (paused) => {
        window.electron?.setPaused?.(paused);
      },
    });
    pauseControl.enableKeyboardShortcut();

    // Time scale control
    new TimeScaleControl({
      containerId: 'time-scale-controls',
      onTimeScaleChange: (scale) => {
        window.electron?.setTimeScale?.(scale);
      },
    });

    if (ipcClient) {
      await ipcClient.connect();

      // Handle simulation tick updates via IPC callback (fires when new data
      // arrives). Hot path: the SoA buffer flows straight to the renderer —
      // state.creatures (the lazy object view) is never touched here.
      ipcClient.onStateUpdate((state) => {
        const { buffer, count } = state.soa;
        currentCreatureCount = count;

        // Update tick rate when telemetry provides it
        if (state.tickRateHz && !isNaN(state.tickRateHz)) {
          creatureRenderer.setTickRate(state.tickRateHz);
        }

        // Detect if this delivery carries new data (tick identity in push mode,
        // exact compare in the poll fallback where tick is always 0)
        const stateChanged = changeDetector.shouldUpdate(state.tick, buffer, count);

        // DEV-only interpolation pipeline probe (stripped from prod builds).
        if (import.meta.env.DEV) {
          const now = performance.now();
          interpDiag.recordDelivery(now);
          interpDiag.recordSnapshot(now, stateChanged);
          const renderMetrics = interpDiag.maybeReport(now);
          if (renderMetrics) window.electron?.sendRenderMetrics?.(renderMetrics);
        }

        if (stateChanged) {
          if (isFirstFrame && count > 0) {
            creatureRenderer.initializeSoA(buffer, count);
            isFirstFrame = false;
          } else {
            creatureRenderer.onSimulationTickSoA(buffer, count);
          }
        }

        // Track the selection into the newest frame (moved or died) — O(1)
        // via the frame's id index.
        selectionManager.updateSelectedFromFrame(creatureRenderer.getLatestSlot());
      });

      // Handle perception debug buffer updates (every tick - smooth visualization)
      ipcClient.onPerceptionDebugUpdate((debugData) => {
        // Only update overlay if a creature is selected (prevents stale data race)
        if (debugData && selectionManager.hasSelection()) {
          const selected = selectionManager.getSelected();
          // Amber ring = how far away THIS creature can be detected by others; driven
          // by its body size (conspicuousness), distinct from the cyan FOV "what it sees".
          const conspicuousnessRadius = selected ? conspicuousness(selected.size) : 0;
          perceptionOverlay.update(debugData, conspicuousnessRadius);
          creatureInfoPanel.updateDebugData(debugData);
          // Update spatial grid overlay with queried + checked cells
          if (debugData.queriedCells && debugData.checkedCells && debugData.creatureCell) {
            spatialGridOverlay.updateQueriedCells(
              debugData.queriedCells,
              debugData.checkedCells,
              debugData.creatureCell
            );
          }
          // Update L1 vision cells for L1 grid highlighting
          spatialGridOverlay.updateL1VisionCells(debugData.l1Vision);
          // Update force overlay with acceleration from perception debug data
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
          spatialGridOverlay.clearL1VisionCells();
          forceOverlay.update(undefined);
        }
      });

      // Handle telemetry updates (cell size and bounds for grid overlay)
      ipcClient.onTelemetryUpdate((telemetry) => {
        if (telemetry.spatialGridCellSize) {
          spatialGridOverlay.setCellSize(telemetry.spatialGridCellSize);
        }
        if (telemetry.l1CellSize) {
          spatialGridOverlay.setL1CellSize(telemetry.l1CellSize);
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
        hudManager.updatePlantWorldCount(telemetry.plantCount ?? 0);
        hudManager.updateCreatureWorldCount(telemetry.creatureCount ?? 0);
      });

      window.addEventListener("beforeunload", async () => {
        await ipcClient.disconnect();
      });
    }

    // Zoom rate limiting
    let lastZoomTime = 0;

    // Viewport culling: send bounds to backend to filter creatures.
    // One reused object — this runs every frame; only IPC allocates (on change).
    const viewportBounds = {
      minX: Infinity, // forces the first send
      minY: 0,
      maxX: 0,
      maxY: 0,
      margin: VIEWPORT_CULLING_CONFIG.MARGIN,
    };

    const sendViewportBounds = () => {
      const halfWidth = (viewportWidth / 2) / camera.zoom;
      const halfHeight = (viewportHeight / 2) / camera.zoom;
      const minX = camera.x - halfWidth;
      const minY = camera.y - halfHeight;
      const maxX = camera.x + halfWidth;
      const maxY = camera.y + halfHeight;

      // Only send if bounds changed significantly (>1 unit)
      if (
        Math.abs(minX - viewportBounds.minX) > 1 ||
        Math.abs(minY - viewportBounds.minY) > 1 ||
        Math.abs(maxX - viewportBounds.maxX) > 1 ||
        Math.abs(maxY - viewportBounds.maxY) > 1
      ) {
        viewportBounds.minX = minX;
        viewportBounds.minY = minY;
        viewportBounds.maxX = maxX;
        viewportBounds.maxY = maxY;
        window.electron?.setViewportBounds?.(viewportBounds);
      }
    };

    // Render the world through a time-eased camera so an occasional late frame glides
    // instead of lurching the whole world (see CameraSmoother). Seeded at the current pose.
    const cameraSmoother = new CameraSmoother(camera.x, camera.y, camera.zoom);

    // Render loop - only handles rendering, not simulation updates
    app.ticker.add(() => {
      const frameStart = performance.now();
      const deltaMs = frameStart - lastFrameTime;
      const deltaTime = deltaMs / 1000;
      const fps = Math.round(1000 / deltaMs);

      // Update camera panning from keyboard/mouse input
      cameraController.update(deltaTime);

      // Ease the rendered camera toward the logic camera, then drive the world transform
      // from the smoothed pose (worldContainer children — plants/overlays — ride this).
      cameraSmoother.follow(camera.x, camera.y, camera.zoom, deltaTime);
      worldContainer.scale.set(cameraSmoother.zoom);
      worldContainer.position.set(
        viewportWidth / 2 - cameraSmoother.x * cameraSmoother.zoom,
        viewportHeight / 2 - cameraSmoother.y * cameraSmoother.zoom
      );
      scaleBarManager.update(camera.zoom);

      // Cull plants to visible world area (+1 cell margin so edge cells never vanish)
      const halfW = viewportWidth / 2 / camera.zoom;
      const halfH = viewportHeight / 2 / camera.zoom;
      plantRenderer.setViewportBounds(
        camera.x - halfW - P0_CELL_SIZE, camera.x + halfW + P0_CELL_SIZE,
        camera.y - halfH - P0_CELL_SIZE, camera.y + halfH + P0_CELL_SIZE
      );

      // Update viewport bounds for backend culling
      sendViewportBounds();

      // Update HUD with cached values
      const state = ipcClient?.getLatestState();
      if (state) {
        hudManager.updateTickRate(state.tickRateHz || 0);
        hudManager.updateCreatureScreenCount(currentCreatureCount);
        hudManager.updatePlantScreenCount(plantRenderer.visibleCount);
        hudManager.updateZoom(camera.zoom);
      }

      // Render with interpolation every frame (smoothed camera pose)
      creatureRenderer.render(
        deltaMs,
        cameraSmoother.x,
        cameraSmoother.y,
        cameraSmoother.zoom,
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

      // Update interaction manager hit area
      interactionManager.updateViewport(
        viewportWidth,
        viewportHeight,
        camera.x,
        camera.y,
        camera.zoom
      );

      // Update minimap viewport rectangle
      minimap.update(camera, viewportWidth, viewportHeight);

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
      viewportWidth = Math.floor(
        window.innerWidth * RENDERING_CONFIG.VIEWPORT_SIZE_RATIO
      );
      viewportHeight = Math.floor(
        window.innerHeight * RENDERING_CONFIG.VIEWPORT_SIZE_RATIO
      );

      updateContainerSize(container, viewportWidth, viewportHeight);
      app.renderer.resize(viewportWidth, viewportHeight);
      camera.setViewportSize(viewportWidth, viewportHeight);
      camera.applyTransform(worldContainer, viewportWidth, viewportHeight);
      scaleBarManager.update(camera.zoom);
      updateMinimapPosition();
    });

    // Keyboard input for camera panning
    window.addEventListener("keydown", (event: KeyboardEvent) => {
      inputManager.handleKeyDown(event.key);
    });

    window.addEventListener("keyup", (event: KeyboardEvent) => {
      inputManager.handleKeyUp(event.key);
    });

    window.addEventListener("blur", () => {
      inputManager.clearAllKeys();
    });

    // Mouse drag for camera panning (right-click)
    app.canvas.addEventListener("pointerdown", (event: PointerEvent) => {
      inputManager.handlePointerDown(event.clientX, event.clientY, event.button);
    });

    app.canvas.addEventListener("pointermove", (event: PointerEvent) => {
      inputManager.handlePointerMove(event.clientX, event.clientY);
    });

    app.canvas.addEventListener("pointerup", () => {
      inputManager.handlePointerUp();
    });

    app.canvas.addEventListener("contextmenu", (event: Event) => {
      event.preventDefault();
    });

    window.addEventListener(
      "wheel",
      (event: WheelEvent) => {
        event.preventDefault();

        const now = performance.now();
        const timeSinceLastZoom = now - lastZoomTime;

        // Calculate max allowed zoom delta based on time elapsed
        const maxDelta = (CAMERA_CONFIG.MAX_ZOOM_SPEED * timeSinceLastZoom) / 1000;

        // Calculate requested zoom delta (in log space)
        let zoomDelta = -event.deltaY * CAMERA_CONFIG.ZOOM_SENSITIVITY;

        // Clamp to max speed (discard excess)
        const sign = Math.sign(zoomDelta);
        zoomDelta = sign * Math.min(Math.abs(zoomDelta), maxDelta);

        if (zoomDelta !== 0) {
          // Only adjust the logic camera — the ticker drives the world transform
          // through CameraSmoother. Applying the raw pose here caused a
          // one-frame pop that the next smoothed frame snapped back from.
          camera.adjustZoom(Math.exp(zoomDelta));
          scaleBarManager.update(camera.zoom);
          lastZoomTime = now;
        }
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

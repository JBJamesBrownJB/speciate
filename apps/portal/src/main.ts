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

    let latestCreatureData: CreatureData[] = [];
    let lastSimulationTick = -1;
    let isFirstFrame = true;

    const ipcClient: IPCClient | null = createIPCClient();

    if (ipcClient) {
      await ipcClient.connect();

      window.addEventListener("beforeunload", async () => {
        await ipcClient.disconnect();
      });
    }

    app.ticker.add(() => {
      const frameStart = performance.now();
      const deltaMs = frameStart - lastFrameTime;
      const fps = Math.round(1000 / deltaMs);

      perfMetrics.recordFrameTime(deltaMs);

      const state = ipcClient?.getLatestState();

      if (state && state.creatures) {
        currentCreatureCount = state.creatures.length;
        latestCreatureData = state.creatures;

        hudManager.updateTickRate(state.tickRateHz || 0);
        hudManager.updateCreatureCount(currentCreatureCount);
        hudManager.updateZoom(camera.zoom);

        // Handle simulation tick updates
        if (state.tick !== lastSimulationTick) {
          lastSimulationTick = state.tick;

          const spriteUpdateStart = performance.now();
          if (isFirstFrame) {
            creatureRenderer.initialize(latestCreatureData);
            isFirstFrame = false;
          } else {
            creatureRenderer.onSimulationTick(latestCreatureData);
          }
          const spriteUpdateEnd = performance.now();
          perfMetrics.recordSpriteUpdateTime(spriteUpdateEnd - spriteUpdateStart);
        }
      }

      // Render with interpolation every frame (60 FPS)
      // Pass camera and viewport parameters for coordinate transform
      creatureRenderer.render(
        deltaMs,
        camera.x,
        camera.y,
        camera.zoom,
        viewportWidth,
        viewportHeight
      );

      hudManager.updateFPS(fps);

      lastFrameTime = frameStart;
    });

    document.title = "✅ Simulation Viewer - Live";

    document.addEventListener('visibilitychange', () => {
    });

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

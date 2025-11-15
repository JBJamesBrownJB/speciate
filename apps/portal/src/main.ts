import { Application, Container, Graphics, type FederatedPointerEvent } from "pixi.js";
import { SpriteProvider } from "@/rendering/SpriteProvider";
import { Camera } from "@/domain/Camera";
import { Viewport } from "@/domain/Viewport";
import { Creature } from "@/domain/Creature";
import { RENDERING_CONFIG } from "@/core/constants";
import { SpritePool } from "@/infrastructure/SpritePool";
import { createIPCClient, type IPCClient } from "@/infrastructure/ipc";
import { PerformanceMetrics } from "@/core/PerformanceMetrics";
import type { CreatureData } from "@/types/GameState";

function updateContainerSize(
  container: HTMLElement,
  width: number,
  height: number
): void {
  container.style.width = `${width}px`;
  container.style.height = `${height}px`;
}

class FPSSparkline {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private fpsHistory: number[] = [];
  private maxHistory = RENDERING_CONFIG.TARGET_FPS;

  constructor(canvasId: string) {
    const canvas = document.getElementById(canvasId) as HTMLCanvasElement;
    if (!canvas) throw new Error(`Canvas ${canvasId} not found`);

    this.canvas = canvas;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("Could not get 2d context");
    this.ctx = ctx;

    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    this.ctx.scale(dpr, dpr);
  }

  update(fps: number): void {
    this.fpsHistory.push(fps);
    if (this.fpsHistory.length > this.maxHistory) {
      this.fpsHistory.shift();
    }
    this.render();
  }

  private render(): void {
    const width = this.canvas.width / (window.devicePixelRatio || 1);
    const height = this.canvas.height / (window.devicePixelRatio || 1);

    this.ctx.clearRect(0, 0, width, height);

    if (this.fpsHistory.length < 2) return;

    const maxFPS = RENDERING_CONFIG.TARGET_FPS;
    const minFPS = 0;

    this.ctx.beginPath();
    this.ctx.strokeStyle = "#6fb83f";
    this.ctx.lineWidth = 1.5;

    const xStep = width / (this.maxHistory - 1);

    this.fpsHistory.forEach((fps, i) => {
      const x = i * xStep;
      const normalizedFPS = Math.max(minFPS, Math.min(maxFPS, fps));
      const y = height - (normalizedFPS / maxFPS) * height;

      if (i === 0) {
        this.ctx.moveTo(x, y);
      } else {
        this.ctx.lineTo(x, y);
      }
    });

    this.ctx.stroke();

    this.ctx.beginPath();
    this.ctx.strokeStyle = "rgba(111, 184, 63, 0.3)";
    this.ctx.lineWidth = 1;
    this.ctx.setLineDash([2, 2]);
    this.ctx.moveTo(0, 0);
    this.ctx.lineTo(width, 0);
    this.ctx.stroke();
    this.ctx.setLineDash([]);
  }
}

function updateInspectionPanel(creatureData: CreatureData): void {
  const idElement = document.getElementById("inspect-id");
  if (idElement) idElement.textContent = `#${creatureData.id}`;

  const positionElement = document.getElementById("inspect-position");
  if (positionElement) {
    positionElement.textContent = `${creatureData.x.toFixed(
      1
    )}m, ${creatureData.y.toFixed(1)}m`;
  }

  const rotationElement = document.getElementById("inspect-rotation");
  if (rotationElement) {
    const degrees = ((creatureData.rotation * 180) / Math.PI).toFixed(1);
    rotationElement.textContent = `${degrees}°`;
  }

  const sizeElement = document.getElementById("inspect-size");
  if (sizeElement) {
    sizeElement.textContent = `${creatureData.width.toFixed(
      2
    )}m × ${creatureData.height.toFixed(2)}m`;
  }

  const velocityElement = document.getElementById("inspect-velocity");
  if (
    velocityElement &&
    creatureData.vx !== undefined &&
    creatureData.vy !== undefined
  ) {
    velocityElement.textContent = `${creatureData.vx.toFixed(
      2
    )}m/s, ${creatureData.vy.toFixed(2)}m/s`;
  } else if (velocityElement) {
    velocityElement.textContent = "N/A";
  }

  const speedElement = document.getElementById("inspect-speed");
  if (
    speedElement &&
    creatureData.vx !== undefined &&
    creatureData.vy !== undefined
  ) {
    const speed = Math.sqrt(creatureData.vx ** 2 + creatureData.vy ** 2);
    speedElement.textContent = `${speed.toFixed(2)}m/s`;
  } else if (speedElement) {
    speedElement.textContent = "N/A";
  }

  const energyElement = document.getElementById("inspect-energy");
  if (energyElement && creatureData.energy !== undefined) {
    energyElement.textContent = `${creatureData.energy.toFixed(0)}`;
  } else if (energyElement) {
    energyElement.textContent = "N/A";
  }

  const ageElement = document.getElementById("inspect-age");
  if (ageElement && creatureData.age !== undefined) {
    ageElement.textContent = `${creatureData.age} ticks`;
  } else if (ageElement) {
    ageElement.textContent = "N/A";
  }
}

function showInspectionPanel(): void {
  const panel = document.getElementById("inspection-panel");
  if (panel) {
    panel.classList.add("visible");
  }
}

function hideInspectionPanel(): void {
  const panel = document.getElementById("inspection-panel");
  if (panel) {
    panel.classList.remove("visible");
  }
}

function selectScaleDistance(zoom: number): {
  distance: number;
  label: string;
} {
  const targetPixelWidth = 120;
  const niceNumbers = [
    1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 20000, 50000,
    100000, 200000, 500000, 1000000,
  ];

  const idealDistance = targetPixelWidth / zoom;

  let bestDistance = niceNumbers[0];
  let bestDiff = Math.abs(idealDistance - bestDistance);

  for (const num of niceNumbers) {
    const diff = Math.abs(idealDistance - num);
    if (diff < bestDiff) {
      bestDistance = num;
      bestDiff = diff;
    }
  }

  let label: string;
  if (bestDistance >= 1000) {
    label = `${bestDistance / 1000}km`;
  } else {
    label = `${bestDistance}m`;
  }

  return { distance: bestDistance, label };
}

function updateScale(zoom: number): void {
  const { distance, label } = selectScaleDistance(zoom);

  const pixelWidth = distance * zoom;
  const scaleLine = document.getElementById("scale-line");
  const scaleLabel = document.getElementById("scale-label");

  if (scaleLine) {
    scaleLine.style.width = `${pixelWidth}px`;
  }

  if (scaleLabel) {
    scaleLabel.textContent = label;
  }
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

    const worldContainer = new Container();
    worldContainer.eventMode = 'static';
    app.stage.addChild(worldContainer);

    app.stage.eventMode = "static";
    app.stage.hitArea = app.screen;

    camera.applyTransform(worldContainer, viewportWidth, viewportHeight);

    updateScale(camera.zoom);

    const spritePool = new SpritePool();
    const texture = spriteProvider.getCreatureTexture();

    const hudElements = {
      fpsValue: document.getElementById("fps-value"),
      tickRateValue: document.getElementById("tick-rate-value"),
      creatureCount: document.getElementById("creature-count"),
      zoomValue: document.getElementById("zoom-value"),
    };

    const fpsSparkline = new FPSSparkline("fps-sparkline");

    let lastFrameTime = performance.now();
    let currentCreatureCount = 0;

    let selectedCreatureId: number | null = null;
    let selectionIndicator: Graphics | null = null;
    let creatureDataMap = new Map<number, any>();

    const perfMetrics = new PerformanceMetrics(RENDERING_CONFIG.TARGET_FPS);

    let latestCreatures: Creature[] = [];
    const ipcClient: IPCClient | null = createIPCClient();

    if (ipcClient) {
      await ipcClient.connect();

      window.addEventListener("beforeunload", async () => {
        await ipcClient.disconnect();
      });
    }

    const closeInspectorBtn = document.getElementById("close-inspector");
    if (closeInspectorBtn) {
      closeInspectorBtn.addEventListener("click", () => {
        if (selectionIndicator && selectionIndicator.parent) {
          selectionIndicator.parent.removeChild(selectionIndicator);
          selectionIndicator.destroy();
          selectionIndicator = null;
        }

        selectedCreatureId = null;

        hideInspectionPanel();
      });
    }

    let frameCount = 0;
    app.ticker.add(() => {
      const frameStart = performance.now();
      const deltaMs = frameStart - lastFrameTime;
      const fps = Math.round(1000 / deltaMs);

      perfMetrics.recordFrameTime(deltaMs);

      frameCount++;

      const state = ipcClient?.getLatestState();

      if (state && state.creatures) {
        currentCreatureCount = state.creatures.length;

        if (hudElements.tickRateValue) {
          const tickRateHz = state.tickRateHz || 0;
          const tickRateDisplay = tickRateHz < 0 ? "..." : `${tickRateHz.toFixed(1)} Hz`;
          hudElements.tickRateValue.textContent = tickRateDisplay;
        }

        if (hudElements.creatureCount) {
          hudElements.creatureCount.textContent = currentCreatureCount.toString();
        }

        if (hudElements.zoomValue) {
          hudElements.zoomValue.textContent = `${camera.zoom.toFixed(2)}x`;
        }

        creatureDataMap.clear();
        for (const creature of state.creatures) {
          creatureDataMap.set(creature.id, creature);
        }

        latestCreatures = state.creatures.map((c: CreatureData) => new Creature(
          c.id,
          c.x,
          c.y,
          c.rotation,
          c.width,
          c.height
        ));
      }

      const spriteUpdateStart = performance.now();
      const creatures = latestCreatures;

      for (const creature of creatures) {
        const sprite = spritePool.acquire(creature.id, texture);

        sprite.position.set(creature.x, creature.y);
        sprite.rotation = creature.rotation;

        const creatureData = creatureDataMap.get(creature.id);
        if (creatureData) {
          const worldScale = Math.min(
            creatureData.width / texture.width,
            creatureData.height / texture.height
          );
          sprite.scale.set(worldScale);
        }

        if (!sprite.parent) {
          worldContainer.addChild(sprite);

          sprite.eventMode = 'static';
          sprite.cursor = 'pointer';

          sprite.on('click', (event: FederatedPointerEvent) => {
            event.stopPropagation();
            const clickedId = (sprite as any).creatureId;

            if (selectionIndicator && selectionIndicator.parent) {
              selectionIndicator.parent.removeChild(selectionIndicator);
              selectionIndicator.destroy();
              selectionIndicator = null;
            }

            selectedCreatureId = clickedId;

            const data = creatureDataMap.get(clickedId);
            if (data) {
              selectionIndicator = new Graphics();
              selectionIndicator.circle(0, 0, Math.max(data.width, data.height) * 0.7);
              selectionIndicator.stroke({ width: 2, color: 0x6fb83f, alpha: 0.8 });
              selectionIndicator.circle(0, 0, Math.max(data.width, data.height) * 0.75);
              selectionIndicator.stroke({ width: 1, color: 0x6fb83f, alpha: 0.4 });

              selectionIndicator.position.set((sprite as any).__lastX, (sprite as any).__lastY);
              worldContainer.addChild(selectionIndicator);

              updateInspectionPanel(data);
              showInspectionPanel();
            }
          });
        }

        (sprite as any).creatureId = creature.id;
        (sprite as any).__lastX = creature.x;
        (sprite as any).__lastY = creature.y;

        if (creature.id === selectedCreatureId && selectionIndicator) {
          selectionIndicator.position.set(creature.x, creature.y);
        }
      }

      const currentCreatureIds = new Set(creatures.map(c => c.id));
      const pooledIds = spritePool.getActiveIds();
      for (const id of pooledIds) {
        if (!currentCreatureIds.has(id)) {
          spritePool.release(id);
        }
      }

      const spriteUpdateEnd = performance.now();
      perfMetrics.recordSpriteUpdateTime(spriteUpdateEnd - spriteUpdateStart);

      if (hudElements.fpsValue) {
        hudElements.fpsValue.textContent = fps.toString();
      }
      fpsSparkline.update(fps);

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
      updateScale(camera.zoom);
    });

    window.addEventListener(
      "wheel",
      (event: WheelEvent) => {
        event.preventDefault();

        const zoomFactor = 1 - event.deltaY * 0.001;

        camera.adjustZoom(zoomFactor);
        camera.applyTransform(worldContainer, viewport.width, viewport.height);
        updateScale(camera.zoom);
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

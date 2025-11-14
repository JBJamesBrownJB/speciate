import { Application, Container, Graphics } from "pixi.js";
import { SpriteProvider } from "@/rendering/SpriteProvider";
import { GridRenderer } from "@/rendering/GridRenderer";
import { Camera } from "@/domain/Camera";
import { Viewport } from "@/domain/Viewport";
import { Creature } from "@/domain/Creature";
import { RENDERING_CONFIG, GRID_CONFIG } from "@/core/constants";
import { SpritePool } from "@/infrastructure/SpritePool";
import { createIPCClient, type IPCClient } from "@/infrastructure/ipc";
import { PerformanceMetrics } from "@/core/PerformanceMetrics";

/**
 * Helper to update the canvas container size to match viewport dimensions
 */
function updateContainerSize(
  container: HTMLElement,
  width: number,
  height: number
): void {
  container.style.width = `${width}px`;
  container.style.height = `${height}px`;
}

/**
 * FPS Sparkline Chart
 * Displays a mini performance graph showing FPS over the last TARGET_FPS frames
 */
class FPSSparkline {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private fpsHistory: number[] = [];
  private maxHistory = RENDERING_CONFIG.TARGET_FPS; // Show last TARGET_FPS frames (~1 second)

  constructor(canvasId: string) {
    const canvas = document.getElementById(canvasId) as HTMLCanvasElement;
    if (!canvas) throw new Error(`Canvas ${canvasId} not found`);

    this.canvas = canvas;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("Could not get 2d context");
    this.ctx = ctx;

    // Set canvas resolution (2x for retina displays)
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

    // Clear canvas
    this.ctx.clearRect(0, 0, width, height);

    if (this.fpsHistory.length < 2) return;

    // Find max FPS for scaling (cap at TARGET_FPS for consistent scale)
    const maxFPS = RENDERING_CONFIG.TARGET_FPS;
    const minFPS = 0;

    // Draw sparkline
    this.ctx.beginPath();
    this.ctx.strokeStyle = "#6fb83f"; // Life green
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

    // Draw 60fps reference line
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

/**
 * Updates the inspection panel with creature data
 */
function updateInspectionPanel(creatureData: any): void {
  // Identity
  const idElement = document.getElementById("inspect-id");
  if (idElement) idElement.textContent = `#${creatureData.id}`;

  // Physical
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

  // Movement
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

  // State
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

/**
 * Shows the inspection panel
 */
function showInspectionPanel(): void {
  const panel = document.getElementById("inspection-panel");
  if (panel) {
    panel.classList.add("visible");
  }
}

/**
 * Hides the inspection panel
 */
function hideInspectionPanel(): void {
  const panel = document.getElementById("inspection-panel");
  if (panel) {
    panel.classList.remove("visible");
  }
}

/**
 * Selects an appropriate "nice" distance for the scale bar
 * based on current zoom level. Targets ~120px bar width.
 */
function selectScaleDistance(zoom: number): {
  distance: number;
  label: string;
} {
  const targetPixelWidth = 120; // Target bar width in pixels
  // Expanded nice numbers to support 2000km × 2000km world (up to 1,000km scale)
  const niceNumbers = [
    1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 20000, 50000,
    100000, 200000, 500000, 1000000,
  ];

  // Calculate what world distance would give us target width
  const idealDistance = targetPixelWidth / zoom;

  // Find closest nice number
  let bestDistance = niceNumbers[0];
  let bestDiff = Math.abs(idealDistance - bestDistance);

  for (const num of niceNumbers) {
    const diff = Math.abs(idealDistance - num);
    if (diff < bestDiff) {
      bestDistance = num;
      bestDiff = diff;
    }
  }

  // Format label with appropriate unit
  let label: string;
  if (bestDistance >= 1000) {
    label = `${bestDistance / 1000}km`;
  } else {
    label = `${bestDistance}m`;
  }

  return { distance: bestDistance, label };
}

/**
 * Updates scale bar and grid based on current zoom level.
 * Scale bar uses adaptive distances for readability across all zoom levels.
 * Grid uses fixed 1m spacing for consistent spatial reference.
 * Grid is only rendered when zoom >= MIN_ZOOM_FOR_GRID for performance.
 */
function updateScaleAndGrid(
  zoom: number,
  gridRenderer: GridRenderer,
  camera: Camera,
  viewport: Viewport
): void {
  const { distance, label } = selectScaleDistance(zoom);

  // Update scale bar
  const pixelWidth = distance * zoom;
  const scaleLine = document.getElementById("scale-line");
  const scaleLabel = document.getElementById("scale-label");

  if (scaleLine) {
    scaleLine.style.width = `${pixelWidth}px`;
  }

  if (scaleLabel) {
    scaleLabel.textContent = label;
  }

  // Only render grid when zoomed in enough (>= MIN_ZOOM_FOR_GRID)
  if (zoom >= GRID_CONFIG.MIN_ZOOM_FOR_GRID) {
    // Fixed 1m grid spacing for consistent spatial reference
    const gridSpacing = 1;
    gridRenderer.update(zoom, gridSpacing, camera, viewport);
  } else {
    // Clear grid when zoomed out
    gridRenderer.clear();
  }
}

async function main(): Promise<void> {
  try {
    // Calculate viewport dimensions from config
    const viewportWidth = Math.floor(
      window.innerWidth * RENDERING_CONFIG.VIEWPORT_SIZE_RATIO
    );
    const viewportHeight = Math.floor(
      window.innerHeight * RENDERING_CONFIG.VIEWPORT_SIZE_RATIO
    );

    // Create Pixi application with WebGL fallback
    const app = new Application();

    try {
      // Try WebGL first (preferred for performance)
      await app.init({
        width: viewportWidth,
        height: viewportHeight,
        backgroundColor: 0x000000,
        resolution: window.devicePixelRatio || 1,
        autoDensity: true,
        preference: 'webgl', // Prefer WebGL
        powerPreference: 'low-power', // Use integrated GPU if available
        failIfMajorPerformanceCaveat: false, // Allow slow WebGL
        antialias: false, // Disable AA to reduce GPU load during init
      });
    } catch (error) {
      console.error('[PixiJS] WebGL initialization failed:', error);

      // Fallback to Canvas2D renderer
      await app.init({
        width: viewportWidth,
        height: viewportHeight,
        backgroundColor: 0x000000,
        resolution: window.devicePixelRatio || 1,
        autoDensity: true,
        preference: 'webgpu', // Force Canvas2D (webgpu is canvas in PixiJS 8)
        antialias: false,
      });

      console.warn('[PixiJS] ⚠️ Running in Canvas2D mode (software rendering, expect 30-60 FPS)');
    }

    // Explicitly disable PixiJS throttle - 0 means use native RAF rate
    app.ticker.maxFPS = 0;

    // Detect actual browser refresh rate via RAF sampling
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
    container.appendChild(app.canvas);

    // Update container size to match viewport (overrides CSS)
    updateContainerSize(container, viewportWidth, viewportHeight);

    // Initialize domain objects
    const camera = new Camera(0, 0, 10); // Start at origin, 10 pixels per meter
    const viewport = new Viewport(viewportWidth, viewportHeight);

    // Load sprite
    const spriteProvider = new SpriteProvider();
    await spriteProvider.init();

    // Create world container (holds all game objects at world coordinates)
    const worldContainer = new Container();
    worldContainer.eventMode = 'static'; // Enable event handling on container
    app.stage.addChild(worldContainer);

    // Enable interactive events on the stage
    app.stage.eventMode = "static";
    app.stage.hitArea = app.screen;

    // Add reference grid for zoom/scale visualization
    const gridRenderer = new GridRenderer(
      worldContainer,
      GRID_CONFIG.SPACING,
      GRID_CONFIG.COLOR,
      GRID_CONFIG.ALPHA,
      GRID_CONFIG.LINE_WIDTH,
      camera.zoom // Pass initial zoom
    );

    // Apply camera transform to world container (after grid is added)
    camera.applyTransform(worldContainer, viewportWidth, viewportHeight);

    // Initialize scale bar and grid with synchronized spacing
    updateScaleAndGrid(camera.zoom, gridRenderer, camera, viewport);

    // Initialize sprite pool for efficient entity management
    const spritePool = new SpritePool();
    const texture = spriteProvider.getCreatureTexture();

    // Cache HUD elements for efficient updates (no DOM queries in render loop)
    const hudElements = {
      fpsValue: document.getElementById("fps-value"),
      tickRateValue: document.getElementById("tick-rate-value"),
      positionValue: document.getElementById("position-value"),
      zoomValue: document.getElementById("zoom-value"),
      creatureCount: document.getElementById("creature-count"),
      emptyStateWarning: document.getElementById("empty-state-warning"),
      statusValue: document.getElementById("status-value"),
      ipcLatencyValue: document.getElementById("ipc-latency-value"),
      decodeTimeValue: document.getElementById("decode-time-value"),
      payloadSizeValue: document.getElementById("payload-size-value"),
      spriteUpdateValue: document.getElementById("sprite-update-value"),
      domUpdateValue: document.getElementById("dom-update-value"),
      frameBudgetValue: document.getElementById("frame-budget-value"),
      tickFreshnessValue: document.getElementById("tick-freshness-value"),
    };

    // Initialize FPS sparkline
    const fpsSparkline = new FPSSparkline("fps-sparkline");

    let lastFrameTime = performance.now();
    let currentTick = 0;
    let currentCreatureCount = 0;

    // Track selected creature for inspection
    let selectedCreatureId: number | null = null;
    let selectionIndicator: Graphics | null = null;
    let creatureDataMap = new Map<number, any>(); // Store full creature data

    // Initialize performance metrics tracker
    const perfMetrics = new PerformanceMetrics(RENDERING_CONFIG.TARGET_FPS);

    // Initialize IPC client (auto-detects Electron/browser)
    let latestCreatures: Creature[] = [];
    const ipcClient: IPCClient | null = createIPCClient();

    if (ipcClient) {
      // Connect to simulation backend (event-driven)
      await ipcClient.connect();

      // Cleanup on page unload
      window.addEventListener("beforeunload", async () => {
        await ipcClient.disconnect();
      });
    }

    // Handle inspection panel close button
    const closeInspectorBtn = document.getElementById("close-inspector");
    if (closeInspectorBtn) {
      closeInspectorBtn.addEventListener("click", () => {
        // Remove selection indicator
        if (selectionIndicator && selectionIndicator.parent) {
          selectionIndicator.parent.removeChild(selectionIndicator);
          selectionIndicator.destroy();
          selectionIndicator = null;
        }

        // Deselect creature
        selectedCreatureId = null;

        // Hide panel
        hideInspectionPanel();
      });
    }

    // FPS counter and HUD update loop (synced to display refresh rate)
    // SYNCHRONOUS: No await! Uses cached state from background polling
    let frameCount = 0;
    app.ticker.add(() => {
      const frameStart = performance.now();
      const deltaMs = frameStart - lastFrameTime;
      const fps = Math.round(1000 / deltaMs);

      // Record frame time
      perfMetrics.recordFrameTime(deltaMs);

      frameCount++;

      // Read latest state from IPC client (synchronous, <1ms!)
      const state = ipcClient?.getLatestState();

      // Defensive: Check both state AND state.creatures
      // This prevents crashes if malformed state somehow gets through validation
      if (state && state.creatures) {
        currentTick = state.tick;
        currentCreatureCount = state.creatures.length;

        // Update status to Connected (use cached element)
        if (hudElements.statusValue && hudElements.statusValue.textContent !== "Connected") {
          hudElements.statusValue.textContent = "Connected";
          hudElements.statusValue.className = "value connected";
        }

        // Update tick rate display (use cached element)
        if (hudElements.tickRateValue) {
          const tickRateHz = state.tickRateHz || 0;
          // Display "..." until first measurement (backend uses -1.0 as sentinel)
          const tickRateDisplay = tickRateHz < 0 ? "..." : `${tickRateHz.toFixed(1)} Hz`;
          hudElements.tickRateValue.textContent = tickRateDisplay;
        }

        // Update creature count (use cached element)
        if (hudElements.creatureCount) {
          hudElements.creatureCount.textContent = currentCreatureCount.toString();
        }

        // Show/hide empty state warning (use cached element)
        if (hudElements.emptyStateWarning) {
          if (currentCreatureCount === 0 && currentTick > 5) {
            hudElements.emptyStateWarning.style.display = 'block';
          } else {
            hudElements.emptyStateWarning.style.display = 'none';
          }
        }

        // Store full creature data for inspection panel
        creatureDataMap.clear();
        for (const creature of state.creatures) {
          creatureDataMap.set(creature.id, creature);
        }

        // Convert to Creature domain objects for rendering
        latestCreatures = state.creatures.map((c: any) => new Creature(
          c.id,
          c.x,
          c.y,
          c.rotation,
          c.width,
          c.height
        ));
      } else {
        // Update status based on mode
        if (hudElements.statusValue) {
          if (!ipcClient) {
            // Browser mode - no backend
            hudElements.statusValue.textContent = "Browser Mode";
            hudElements.statusValue.className = "value";
          } else if (currentTick === 0) {
            // Electron mode but still connecting
            hudElements.statusValue.textContent = "Connecting...";
            hudElements.statusValue.className = "value reconnecting";
          }
        }
      }

      // RENDER SPRITES (updated from latest state above)
      const spriteUpdateStart = performance.now();
      const creatures = latestCreatures;

      for (const creature of creatures) {
        const sprite = spritePool.acquire(creature.id, texture);

        // Update sprite transform
        sprite.position.set(creature.x, creature.y);
        sprite.rotation = creature.rotation;

        // Get creature data for sizing
        const creatureData = creatureDataMap.get(creature.id);
        if (creatureData) {
          const worldScale = Math.min(
            creatureData.width / texture.width,
            creatureData.height / texture.height
          );
          sprite.scale.set(worldScale);
        }

        // Add to world container and configure interactivity (only once per sprite)
        if (!sprite.parent) {
          worldContainer.addChild(sprite);

          // Make sprite interactive for click detection (configure once)
          sprite.eventMode = 'static';
          sprite.cursor = 'pointer';

          // Add click handler ONCE when sprite is first added
          sprite.on('click', (event: any) => {
            event.stopPropagation(); // Prevent event bubbling
            const clickedId = (sprite as any).creatureId;

            // Remove old selection indicator
            if (selectionIndicator && selectionIndicator.parent) {
              selectionIndicator.parent.removeChild(selectionIndicator);
              selectionIndicator.destroy();
              selectionIndicator = null;
            }

            // Select new creature
            selectedCreatureId = clickedId;

            // Get creature data and show panel
            const data = creatureDataMap.get(clickedId);
            if (data) {
              // Create selection indicator
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

        // Store creature ID on sprite for click handling (update every frame)
        (sprite as any).creatureId = creature.id;
        (sprite as any).__lastX = creature.x;
        (sprite as any).__lastY = creature.y;

        // Update selection indicator position if this is the selected creature
        if (creature.id === selectedCreatureId && selectionIndicator) {
          selectionIndicator.position.set(creature.x, creature.y);
        }
      }

      // Release sprites for creatures that no longer exist
      const currentCreatureIds = new Set(creatures.map(c => c.id));
      const pooledIds = spritePool.getActiveIds();
      for (const id of pooledIds) {
        if (!currentCreatureIds.has(id)) {
          spritePool.release(id);
        }
      }

      const spriteUpdateEnd = performance.now();
      perfMetrics.recordSpriteUpdateTime(spriteUpdateEnd - spriteUpdateStart);

      // DOM UPDATES
      const domUpdateStart = performance.now();

      // Update FPS display and sparkline (use cached element)
      if (hudElements.fpsValue) {
        hudElements.fpsValue.textContent = fps.toString();
      }
      fpsSparkline.update(fps);

      // Update camera position (use cached element)
      if (hudElements.positionValue) {
        const x = Math.round(camera.x);
        const y = Math.round(camera.y);
        hudElements.positionValue.textContent = `${x}m, ${y}m`;
      }

      // Update zoom level (use cached element)
      if (hudElements.zoomValue) {
        const zoom = camera.zoom.toFixed(2);
        hudElements.zoomValue.textContent = `${zoom}px/m`;
      }

      const domUpdateEnd = performance.now();
      perfMetrics.recordDomUpdateTime(domUpdateEnd - domUpdateStart);

      // Calculate total render time (sprite + DOM)
      const totalRenderTime = (spriteUpdateEnd - spriteUpdateStart) + (domUpdateEnd - domUpdateStart);
      perfMetrics.recordTotalRenderTime(totalRenderTime);

      // Update performance metrics in HUD
      const perfSnapshot = perfMetrics.getSnapshot();

      if (hudElements.ipcLatencyValue) {
        hudElements.ipcLatencyValue.textContent = `${perfSnapshot.ipcLatencyAvg.toFixed(2)}ms`;
      }

      if (hudElements.decodeTimeValue) {
        hudElements.decodeTimeValue.textContent = `${perfSnapshot.decodeTimeAvg.toFixed(2)}ms`;
      }

      if (hudElements.payloadSizeValue) {
        hudElements.payloadSizeValue.textContent = `${(perfSnapshot.payloadSize / 1024).toFixed(2)}KB`;
      }

      if (hudElements.spriteUpdateValue) {
        hudElements.spriteUpdateValue.textContent = `${perfSnapshot.spriteUpdateTime.toFixed(2)}ms`;
      }

      if (hudElements.domUpdateValue) {
        hudElements.domUpdateValue.textContent = `${perfSnapshot.domUpdateTime.toFixed(2)}ms`;
      }

      if (hudElements.frameBudgetValue) {
        const overhead = perfSnapshot.frameOverhead;
        const color = overhead > 0 ? '#d94848' : '#6fb83f'; // Red if over budget, green otherwise
        hudElements.frameBudgetValue.textContent = `${overhead >= 0 ? '+' : ''}${overhead.toFixed(2)}ms`;
        hudElements.frameBudgetValue.style.color = color;
      }

      if (hudElements.tickFreshnessValue) {
        const tickAge = perfSnapshot.tickAge;
        const color = tickAge > 5 ? '#d94848' : (tickAge > 2 ? '#f0a830' : '#6fb83f');
        hudElements.tickFreshnessValue.textContent = `${tickAge} frames`;
        hudElements.tickFreshnessValue.style.color = color;
      }

      // Update lastFrameTime for next frame
      lastFrameTime = frameStart;
    });

    document.title = "✅ Simulation Viewer - Live";

    // Monitor for browser throttling of background tabs
    document.addEventListener('visibilitychange', () => {
      // Tab visibility changed - could update UI status here if needed
    });

    // Handle resize
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
      updateScaleAndGrid(camera.zoom, gridRenderer, camera, viewport);
    });

    // Handle mouse wheel zoom
    window.addEventListener(
      "wheel",
      (event: WheelEvent) => {
        event.preventDefault();

        // Zoom sensitivity: 0.001 = 0.1% per wheel delta unit
        // Standard mouse wheel generates ~100 deltaY per notch, so ~10% zoom per notch
        const zoomFactor = 1 - event.deltaY * 0.001;

        camera.adjustZoom(zoomFactor);
        camera.applyTransform(worldContainer, viewport.width, viewport.height);
        updateScaleAndGrid(camera.zoom, gridRenderer, camera, viewport);
      },
      { passive: false }
    ); // passive: false required for preventDefault()
  } catch (error) {
    console.error("[Portal] ❌ Failed to initialize:", error);
    document.title = "❌ Failed";
    document.body.innerHTML = `<div style="color: white; padding: 20px; font-family: monospace;">
      <h1>Failed to load</h1>
      <pre>${error}</pre>
    </div>`;
  }
}

main();

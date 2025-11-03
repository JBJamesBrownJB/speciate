import { PixiApp } from '@/rendering/PixiApp';
import { EntityRenderer } from '@/rendering/EntityRenderer';
import { DiagnosticOverlay } from '@/rendering/DiagnosticOverlay';
import { WebSocketClient } from '@/core/WebSocketClient';
import { StateManager } from '@/core/StateManager';
import { GameLoop } from '@/core/GameLoop';
import { ConnectionState } from '@/types/entities';
import type { SimulationStateMessage } from '@/types/messages';

class HUDController {
  private fpsElement: HTMLElement;
  private tickElement: HTMLElement;
  private pingElement: HTMLElement;
  private statusElement: HTMLElement;
  private connectionOverlay: HTMLElement;

  constructor() {
    this.fpsElement = this.getElement('fps-value');
    this.tickElement = this.getElement('tick-value');
    this.pingElement = this.getElement('ping-value');
    this.statusElement = this.getElement('status-value');
    this.connectionOverlay = this.getElement('connection-overlay');
  }

  private getElement(id: string): HTMLElement {
    const element = document.getElementById(id);
    if (!element) throw new Error(`HUD element not found: ${id}`);
    return element;
  }

  public updateFPS(fps: number): void {
    this.fpsElement.textContent = fps.toString();
  }

  public updateTick(tick: number): void {
    this.tickElement.textContent = tick.toString();
  }

  public updatePing(ping: number): void {
    this.pingElement.textContent = `${ping}ms`;
  }

  public updateConnectionState(state: ConnectionState): void {
    this.statusElement.textContent = state;
    this.statusElement.className = 'value';
    switch (state) {
      case ConnectionState.Connected:
        this.statusElement.classList.add('connected');
        this.hideConnectionOverlay();
        break;
      case ConnectionState.Connecting:
      case ConnectionState.Reconnecting:
        this.statusElement.classList.add('reconnecting');
        if (state === ConnectionState.Reconnecting) {
          this.showConnectionOverlay();
        }
        break;
      case ConnectionState.Disconnected:
        this.statusElement.classList.add('disconnected');
        this.showConnectionOverlay();
        break;
    }
  }

  private showConnectionOverlay(): void {
    this.connectionOverlay.classList.add('visible');
  }

  private hideConnectionOverlay(): void {
    this.connectionOverlay.classList.remove('visible');
  }
}

class SimulationApp {
  private pixiApp: PixiApp;
  private entityRenderer: EntityRenderer | null = null;
  private diagnosticOverlay: DiagnosticOverlay | null = null;
  private wsClient: WebSocketClient;
  private stateManager: StateManager;
  private gameLoop: GameLoop;
  private hud: HUDController;

  constructor() {
    this.pixiApp = new PixiApp(
      document.getElementById('canvas-container') ?? document.body
    );
    this.wsClient = new WebSocketClient('ws://localhost:8080/ws');
    this.stateManager = new StateManager();
    this.gameLoop = new GameLoop();
    this.hud = new HUDController();
  }

  public async start(): Promise<void> {
    try {
      await this.pixiApp.init();
      const stage = this.pixiApp.getStage();
      const { width, height } = this.pixiApp.getDimensions();

      // Initialize renderer
      this.entityRenderer = new EntityRenderer(stage, width, height);

      // Initialize diagnostic overlay
      this.diagnosticOverlay = new DiagnosticOverlay(stage);

      // Draw world boundary and grid
      const bounds = this.entityRenderer.getWorldBoundsScreen();
      this.diagnosticOverlay.drawWorldBoundary(bounds.x, bounds.y, bounds.width, bounds.height);
      this.diagnosticOverlay.drawGrid(bounds.x, bounds.y, bounds.width, bounds.height, 50);

      // Setup keyboard shortcut to toggle diagnostics (D key)
      window.addEventListener('keydown', (e) => {
        if (e.key === 'd' || e.key === 'D') {
          this.diagnosticOverlay?.toggle();
        }
      });

      this.setupWebSocketHandlers();
      this.setupGameLoop();
      this.wsClient.connect();
      this.gameLoop.start();

      console.log('[SimulationApp] Started successfully');
      console.log('[SimulationApp] Press "D" to toggle diagnostic overlay');
    } catch (error) {
      console.error('[SimulationApp] Failed to start:', error);
      throw error;
    }
  }

  private setupWebSocketHandlers(): void {
    this.wsClient.onMessage((message: SimulationStateMessage) => {
      this.stateManager.updateFromServer(message);

      // Directly update renderer with creature data
      if (message.creatures && this.entityRenderer) {
        this.entityRenderer.updateCreatures(message.creatures);
      }
    });
    this.wsClient.onConnectionStateChange((state: ConnectionState) => {
      this.hud.updateConnectionState(state);
      if (state === ConnectionState.Disconnected || state === ConnectionState.Reconnecting) {
        this.stateManager.clear();
        this.entityRenderer?.clear();
      }
    });
  }

  private setupGameLoop(): void {
    this.gameLoop.onUpdate(() => {
      // Interpolated rendering every frame for smooth movement
      if (this.entityRenderer) {
        this.entityRenderer.render();
      }

      // Update HUD
      const fps = this.gameLoop.getFPS();
      const tick = this.stateManager.getCurrentTick();
      const ping = this.wsClient.getPing();

      this.hud.updateFPS(fps);
      this.hud.updateTick(tick);
      this.hud.updatePing(ping);

      // Update diagnostic overlay
      if (this.diagnosticOverlay) {
        this.diagnosticOverlay.update({
          fps,
          ping,
          tick,
          creatureCount: 0, // TODO: get from renderer
          worldBounds: { width: 180, height: 130 },
        });
      }
    });
  }

  public destroy(): void {
    this.gameLoop.stop();
    this.wsClient.disconnect();
    this.pixiApp.destroy();
  }
}

async function main(): Promise<void> {
  const app = new SimulationApp();
  try {
    await app.start();
  } catch (error) {
    console.error('Failed to start application:', error);
    const overlay = document.getElementById('connection-overlay');
    if (overlay) {
      overlay.innerHTML = `<h2>Initialization Failed</h2><p>Check console for details</p>`;
      overlay.classList.add('visible');
    }
  }
  window.addEventListener('beforeunload', () => app.destroy());
}

main().catch(console.error);

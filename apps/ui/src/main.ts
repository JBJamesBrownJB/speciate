import { PixiApp } from '@/rendering/PixiApp';
import { EntityRenderer } from '@/rendering/EntityRenderer';
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
      this.entityRenderer = new EntityRenderer(stage, width, height);
      this.setupWebSocketHandlers();
      this.setupGameLoop();
      this.wsClient.connect();
      this.gameLoop.start();
    } catch (error) {
      console.error('[SimulationApp] Failed to start:', error);
      throw error;
    }
  }

  private setupWebSocketHandlers(): void {
    this.wsClient.onMessage((message: SimulationStateMessage) => {
      this.stateManager.updateFromServer(message);
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
      const entityIds = this.stateManager.getEntityIds();
      for (const entityId of entityIds) {
        const position = this.stateManager.getInterpolatedPosition(entityId);
        if (position && this.entityRenderer) {
          this.entityRenderer.updateEntityPosition(entityId, position);
        }
      }
      this.hud.updateFPS(this.gameLoop.getFPS());
      this.hud.updateTick(this.stateManager.getCurrentTick());
      this.hud.updatePing(this.wsClient.getPing());
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

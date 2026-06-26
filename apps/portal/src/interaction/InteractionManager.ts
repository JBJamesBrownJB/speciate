import { Container, Graphics, FederatedPointerEvent } from 'pixi.js';
import type { SelectionManager } from '@/systems/SelectionManager';
import type { SpatialGridOverlay } from '@/rendering/overlays/SpatialGridOverlay';
import { GridMode, P0_CELL_SIZE } from '@/rendering/overlays/SpatialGridOverlay';
import type { CreatureData } from '@/types/GameState';
import type { IPCClient } from '@/infrastructure/ipc';

type ActiveTool = 'plant' | null;

const CLICK_RADIUS = 15;

interface InteractionManagerConfig {
  worldContainer: Container;
  gridOverlay: SpatialGridOverlay;
  selectionManager: SelectionManager;
  ipcClient: IPCClient | null;
  getCreatures: () => CreatureData[];
}

export class InteractionManager {
  private worldContainer: Container;
  private gridOverlay: SpatialGridOverlay;
  private selectionManager: SelectionManager;
  private ipcClient: IPCClient | null;
  private getCreatures: () => CreatureData[];

  private hitArea: Graphics;
  private activeTool: ActiveTool = null;
  private isPainting = false;
  private paintedThisStroke = new Set<string>();

  constructor(config: InteractionManagerConfig) {
    this.worldContainer = config.worldContainer;
    this.gridOverlay = config.gridOverlay;
    this.selectionManager = config.selectionManager;
    this.ipcClient = config.ipcClient;
    this.getCreatures = config.getCreatures;

    this.hitArea = new Graphics();
    this.hitArea.eventMode = 'static';
    this.hitArea.cursor = 'default';

    this.hitArea.on('pointerdown', this.handlePointerDown);
    this.hitArea.on('pointermove', this.handlePointerMove);
    this.hitArea.on('pointerout', this.handlePointerOut);
    this.hitArea.on('pointerup', () => this.endStroke());
    this.hitArea.on('pointerupoutside', () => this.endStroke());

    this.worldContainer.addChild(this.hitArea);
  }

  updateViewport(viewportWidth: number, viewportHeight: number, cameraX: number, cameraY: number, zoom: number): void {
    const halfWidth = viewportWidth / 2 / zoom;
    const halfHeight = viewportHeight / 2 / zoom;

    const worldLeft = cameraX - halfWidth;
    const worldTop = cameraY - halfHeight;
    const worldWidth = halfWidth * 2;
    const worldHeight = halfHeight * 2;

    this.hitArea.clear();
    this.hitArea.rect(worldLeft, worldTop, worldWidth, worldHeight);
    this.hitArea.fill({ color: 0x000000, alpha: 0.001 });

    const mode = this.gridOverlay.getMode();
    if (this.activeTool === 'plant' || mode === GridMode.L1 || mode === GridMode.P0) {
      this.hitArea.cursor = 'crosshair';
    } else {
      this.hitArea.cursor = 'default';
    }
  }

  private handlePointerDown = (event: FederatedPointerEvent): void => {
    const localPos = event.getLocalPosition(this.worldContainer);
    const worldX = localPos.x;
    const worldY = localPos.y;

    if (this.activeTool === 'plant') {
      this.isPainting = true;
      this.paintedThisStroke.clear();
      this.tryPaintAt(worldX, worldY);
      return;
    }

    if (this.gridOverlay.getMode() === GridMode.L1) {
      return;
    }

    this.handleCreatureClick(worldX, worldY);
  };

  private handlePointerMove = (event: FederatedPointerEvent): void => {
    const localPos = event.getLocalPosition(this.worldContainer);

    if (this.isPainting && this.activeTool === 'plant') {
      this.tryPaintAt(localPos.x, localPos.y);
      return;
    }

    const mode = this.gridOverlay.getMode();
    if (mode !== GridMode.L1 && mode !== GridMode.P0) {
      return;
    }
    this.gridOverlay.handleHover(localPos.x, localPos.y);
  };

  private handlePointerOut = (): void => {
    this.gridOverlay.clearHover();
  };

  setActiveTool(tool: ActiveTool): void {
    this.activeTool = tool;
    if (!tool) {
      this.isPainting = false;
      this.paintedThisStroke.clear();
    }
  }

  private tryPaintAt(worldX: number, worldY: number): void {
    const cx = Math.floor(worldX / P0_CELL_SIZE) * P0_CELL_SIZE;
    const cy = Math.floor(worldY / P0_CELL_SIZE) * P0_CELL_SIZE;
    const key = `${cx}:${cy}`;
    if (this.paintedThisStroke.has(key)) return;
    const centerX = cx + P0_CELL_SIZE / 2;
    const centerY = cy + P0_CELL_SIZE / 2;
    if (this.gridOverlay.hasPlantAt(centerX, centerY)) {
      this.paintedThisStroke.add(key);
      return;
    }
    this.paintedThisStroke.add(key);
    window.electron?.spawnPlant?.(centerX, centerY);
  }

  private endStroke(): void {
    this.isPainting = false;
    this.paintedThisStroke.clear();
  }

  private handleCreatureClick(worldX: number, worldY: number): void {
    const creatures = this.getCreatures();
    const nearest = this.selectionManager.findNearestCreature(
      creatures,
      worldX,
      worldY,
      CLICK_RADIUS
    );

    if (nearest) {
      this.selectionManager.selectCreature(nearest);
      if (this.ipcClient) {
        this.ipcClient.selectCreatureDebug(nearest.id);
      }
    } else {
      this.selectionManager.deselect();
      if (this.ipcClient) {
        this.ipcClient.selectCreatureDebug(null);
      }
    }
  }

  destroy(): void {
    this.hitArea.off('pointerdown', this.handlePointerDown);
    this.hitArea.off('pointermove', this.handlePointerMove);
    this.hitArea.off('pointerout', this.handlePointerOut);
    this.hitArea.removeAllListeners('pointerup');
    this.hitArea.removeAllListeners('pointerupoutside');
    this.hitArea.destroy();
  }
}

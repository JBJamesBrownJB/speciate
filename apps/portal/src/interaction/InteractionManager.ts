import { Container, Graphics, FederatedPointerEvent } from 'pixi.js';
import type { SelectionManager } from '@/systems/SelectionManager';
import type { SpatialGridOverlay } from '@/rendering/overlays/SpatialGridOverlay';
import { GridMode } from '@/rendering/overlays/SpatialGridOverlay';
import type { CreatureData } from '@/types/GameState';
import type { IPCClient } from '@/infrastructure/ipc';
import type { TerrainTool } from '@/ui/TerrainTool';

const CLICK_RADIUS = 15;

interface InteractionManagerConfig {
  worldContainer: Container;
  gridOverlay: SpatialGridOverlay;
  selectionManager: SelectionManager;
  ipcClient: IPCClient | null;
  getCreatures: () => CreatureData[];
  terrainTool?: TerrainTool;
}

export class InteractionManager {
  private worldContainer: Container;
  private gridOverlay: SpatialGridOverlay;
  private selectionManager: SelectionManager;
  private ipcClient: IPCClient | null;
  private getCreatures: () => CreatureData[];
  private terrainTool: TerrainTool | null;

  private hitArea: Graphics;

  constructor(config: InteractionManagerConfig) {
    this.worldContainer = config.worldContainer;
    this.gridOverlay = config.gridOverlay;
    this.selectionManager = config.selectionManager;
    this.ipcClient = config.ipcClient;
    this.getCreatures = config.getCreatures;
    this.terrainTool = config.terrainTool ?? null;

    this.hitArea = new Graphics();
    this.hitArea.eventMode = 'static';
    this.hitArea.cursor = 'default';

    this.hitArea.on('pointerdown', this.handlePointerDown);
    this.hitArea.on('pointermove', this.handlePointerMove);
    this.hitArea.on('pointerup', this.handlePointerUp);
    this.hitArea.on('pointerout', this.handlePointerOut);
    this.hitArea.on('pointerupoutside', this.handlePointerUp);

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

    if (this.terrainTool?.isActive()) {
      this.hitArea.cursor = this.terrainTool.getCursor();
    } else if (this.gridOverlay.getMode() === GridMode.L1) {
      this.hitArea.cursor = 'crosshair';
    } else {
      this.hitArea.cursor = 'default';
    }
  }

  private handlePointerDown = (event: FederatedPointerEvent): void => {
    const localPos = event.getLocalPosition(this.worldContainer);
    const worldX = localPos.x;
    const worldY = localPos.y;

    if (this.terrainTool?.handlePointerDown(worldX, worldY)) {
      return;
    }

    if (this.gridOverlay.getMode() === GridMode.L1) {
      return;
    }

    this.handleCreatureClick(worldX, worldY);
  };

  private handlePointerMove = (event: FederatedPointerEvent): void => {
    const localPos = event.getLocalPosition(this.worldContainer);
    const worldX = localPos.x;
    const worldY = localPos.y;

    if (this.terrainTool?.handlePointerMove(worldX, worldY)) {
      return;
    }

    if (this.gridOverlay.getMode() !== GridMode.L1) {
      return;
    }

    this.gridOverlay.handleHover(localPos.x, localPos.y);
  };

  private handlePointerUp = (): void => {
    this.terrainTool?.handlePointerUp();
  };

  private handlePointerOut = (): void => {
    this.terrainTool?.handlePointerUp();
    this.gridOverlay.clearHover();
  };

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
    this.hitArea.off('pointerup', this.handlePointerUp);
    this.hitArea.off('pointerout', this.handlePointerOut);
    this.hitArea.off('pointerupoutside', this.handlePointerUp);
    this.hitArea.destroy();
  }
}

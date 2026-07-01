import { Container, Graphics, Rectangle, FederatedPointerEvent } from 'pixi.js';
import type { SelectionManager } from '@/systems/SelectionManager';
import type { SpatialGridOverlay } from '@/rendering/overlays/SpatialGridOverlay';
import { GridMode, P0_CELL_SIZE } from '@/rendering/overlays/SpatialGridOverlay';
import type { CreatureFrameView } from '@/types/GameState';
import type { IPCClient } from '@/infrastructure/ipc';

type ActiveTool = 'plant' | null;

const CLICK_RADIUS = 15;

interface InteractionManagerConfig {
  worldContainer: Container;
  gridOverlay: SpatialGridOverlay;
  selectionManager: SelectionManager;
  ipcClient: IPCClient | null;
  /** Newest decoded SoA frame (null before the first delivery). */
  getFrame: () => CreatureFrameView | null;
}

export class InteractionManager {
  private worldContainer: Container;
  private gridOverlay: SpatialGridOverlay;
  private selectionManager: SelectionManager;
  private ipcClient: IPCClient | null;
  private getFrame: () => CreatureFrameView | null;

  private hitSurface: Graphics;
  // One Rectangle, mutated in place every frame. Setting `hitArea` makes hit
  // testing use it directly — no geometry, no per-frame clear/rect/fill draw.
  private readonly hitBounds = new Rectangle(0, 0, 0, 0);
  private activeTool: ActiveTool = null;
  private isPainting = false;
  private paintedThisStroke = new Set<string>();

  constructor(config: InteractionManagerConfig) {
    this.worldContainer = config.worldContainer;
    this.gridOverlay = config.gridOverlay;
    this.selectionManager = config.selectionManager;
    this.ipcClient = config.ipcClient;
    this.getFrame = config.getFrame;

    this.hitSurface = new Graphics();
    this.hitSurface.eventMode = 'static';
    this.hitSurface.cursor = 'default';
    this.hitSurface.hitArea = this.hitBounds;

    this.hitSurface.on('pointerdown', this.handlePointerDown);
    this.hitSurface.on('pointermove', this.handlePointerMove);
    this.hitSurface.on('pointerout', this.handlePointerOut);
    this.hitSurface.on('pointerup', () => this.endStroke());
    this.hitSurface.on('pointerupoutside', () => this.endStroke());

    this.worldContainer.addChild(this.hitSurface);
  }

  updateViewport(viewportWidth: number, viewportHeight: number, cameraX: number, cameraY: number, zoom: number): void {
    const halfWidth = viewportWidth / 2 / zoom;
    const halfHeight = viewportHeight / 2 / zoom;

    this.hitBounds.x = cameraX - halfWidth;
    this.hitBounds.y = cameraY - halfHeight;
    this.hitBounds.width = halfWidth * 2;
    this.hitBounds.height = halfHeight * 2;

    const mode = this.gridOverlay.getMode();
    if (this.activeTool === 'plant' || mode === GridMode.L1 || mode === GridMode.P0) {
      this.hitSurface.cursor = 'crosshair';
    } else {
      this.hitSurface.cursor = 'default';
    }
  }

  /** The world-space rectangle currently receiving pointer events. */
  getHitBounds(): Rectangle {
    return this.hitBounds;
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
    const nearest = this.selectionManager.findNearestCreature(
      this.getFrame(),
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
    this.hitSurface.off('pointerdown', this.handlePointerDown);
    this.hitSurface.off('pointermove', this.handlePointerMove);
    this.hitSurface.off('pointerout', this.handlePointerOut);
    this.hitSurface.removeAllListeners('pointerup');
    this.hitSurface.removeAllListeners('pointerupoutside');
    this.hitSurface.destroy();
  }
}

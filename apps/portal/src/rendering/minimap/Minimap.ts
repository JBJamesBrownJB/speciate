import { Container, FederatedPointerEvent, Graphics } from "pixi.js";
import { BaseOverlay, type OverlayConfig } from "../overlays/IOverlay";
import type { WorldBounds } from "@/domain/WorldBounds";

export interface ViewportRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export class Minimap extends BaseOverlay {
  readonly config: OverlayConfig = {
    name: "minimap",
    devToolsOnly: false,
    keyboardShortcut: "m",
  };

  onMinimapClick: ((worldX: number, worldY: number) => void) | null = null;

  private minimapContainer: Container;
  private background: Graphics;
  private viewportGraphics: Graphics;
  private border: Graphics;
  private readonly size: number;
  private readonly worldBounds: WorldBounds;
  private readonly worldWidth: number;
  private readonly worldHeight: number;
  private lastViewportRect: ViewportRect = { x: 0, y: 0, width: 0, height: 0 };
  private destroyed = false;
  private readonly boundPointerDown: (event: FederatedPointerEvent) => void;

  constructor(
    parent: Container,
    worldBounds: WorldBounds,
    size: number = 180
  ) {
    super(parent);

    if (size <= 0) {
      throw new Error("Minimap size must be positive");
    }

    const worldWidth = worldBounds.maxX - worldBounds.minX;
    const worldHeight = worldBounds.maxY - worldBounds.minY;

    if (worldWidth <= 0 || worldHeight <= 0) {
      throw new Error("WorldBounds must have positive dimensions");
    }

    this.size = size;
    this.worldBounds = worldBounds;
    this.worldWidth = worldWidth;
    this.worldHeight = worldHeight;

    this.boundPointerDown = this.onPointerDown.bind(this);

    this.minimapContainer = new Container();
    this.minimapContainer.eventMode = "static";
    this.minimapContainer.on("pointerdown", this.boundPointerDown);

    this.background = new Graphics();
    this.background.rect(0, 0, size, size);
    this.background.fill({ color: 0x000000, alpha: 0.6 });

    this.viewportGraphics = new Graphics();

    this.border = new Graphics();
    this.border.setStrokeStyle({ width: 1, color: 0xffffff, alpha: 0.3 });
    this.border.rect(0, 0, size, size);
    this.border.stroke();

    this.minimapContainer.addChild(this.background);
    this.minimapContainer.addChild(this.viewportGraphics);
    this.minimapContainer.addChild(this.border);

    parent.addChild(this.minimapContainer);

    this.visible = true;
  }

  update(
    camera: { x: number; y: number; zoom: number },
    viewportWidth: number,
    viewportHeight: number
  ): void {
    if (!this.visible || this.destroyed) return;
    if (camera.zoom <= 0) return;
    this.updateViewportRect(camera, viewportWidth, viewportHeight);
  }

  handleClick(minimapX: number, minimapY: number): void {
    if (!this.onMinimapClick) return;

    const worldX = this.worldBounds.minX + (minimapX / this.size) * this.worldWidth;
    const worldY = this.worldBounds.minY + (minimapY / this.size) * this.worldHeight;

    this.onMinimapClick(worldX, worldY);
  }

  getViewportRect(): ViewportRect {
    return { ...this.lastViewportRect };
  }

  setPosition(x: number, y: number): void {
    this.minimapContainer.x = x;
    this.minimapContainer.y = y;
  }

  destroy(): void {
    if (this.destroyed) return;
    this.destroyed = true;

    this.minimapContainer.off("pointerdown", this.boundPointerDown);
    this.background.destroy();
    this.viewportGraphics.destroy();
    this.border.destroy();
    this.minimapContainer.destroy();
  }

  protected onVisibilityChange(visible: boolean): void {
    this.minimapContainer.visible = visible;
  }

  private onPointerDown(event: FederatedPointerEvent): void {
    const local = event.getLocalPosition(this.minimapContainer);
    this.handleClick(local.x, local.y);
  }

  private updateViewportRect(
    camera: { x: number; y: number; zoom: number },
    viewportWidth: number,
    viewportHeight: number
  ): void {
    const viewWorldWidth = viewportWidth / camera.zoom;
    const viewWorldHeight = viewportHeight / camera.zoom;

    const rectWidth = (viewWorldWidth / this.worldWidth) * this.size;
    const rectHeight = (viewWorldHeight / this.worldHeight) * this.size;

    const cameraMinimapX = ((camera.x - this.worldBounds.minX) / this.worldWidth) * this.size;
    const cameraMinimapY = ((camera.y - this.worldBounds.minY) / this.worldHeight) * this.size;

    const rectX = cameraMinimapX - rectWidth / 2;
    const rectY = cameraMinimapY - rectHeight / 2;

    // Skip the clear+stroke when the rect is identical to last frame (static camera).
    const last = this.lastViewportRect;
    if (
      rectX === last.x &&
      rectY === last.y &&
      rectWidth === last.width &&
      rectHeight === last.height
    ) {
      return;
    }

    this.lastViewportRect = { x: rectX, y: rectY, width: rectWidth, height: rectHeight };

    this.viewportGraphics.clear();
    this.viewportGraphics.setStrokeStyle({ width: 2, color: 0xffffff, alpha: 1.0 });
    this.viewportGraphics.rect(rectX, rectY, rectWidth, rectHeight);
    this.viewportGraphics.stroke();
  }
}

import { Graphics, Container } from 'pixi.js';
import type { IOverlay, OverlayConfig } from './IOverlay';

const HIGHLIGHT_COLOR = 0xffff00;
const HIGHLIGHT_LINE_WIDTH = 1;
const HIGHLIGHT_ALPHA = 0.3;
const RADIUS_PADDING = 0.07;

export class SelectionHighlight implements IOverlay {
  readonly config: OverlayConfig = {
    name: 'selection',
    devToolsOnly: false,
  };

  private graphics: Graphics;
  private visible: boolean = false;
  private positionX: number = 0;
  private positionY: number = 0;
  private baseRadius: number = 15;

  constructor(container: Container) {
    this.graphics = new Graphics();
    container.addChild(this.graphics);
  }

  showAt(worldX: number, worldY: number, radius: number): void {
    this.positionX = worldX;
    this.positionY = worldY;
    this.baseRadius = radius;
    this.visible = true;
    this.render();
  }

  show(): void {
    this.visible = true;
    this.render();
  }

  hide(): void {
    this.visible = false;
    this.graphics.clear();
  }

  toggle(): void {
    if (this.visible) {
      this.hide();
    } else {
      this.show();
    }
  }

  isVisible(): boolean {
    return this.visible;
  }

  updatePosition(worldX: number, worldY: number): void {
    if (!this.visible) return;
    this.positionX = worldX;
    this.positionY = worldY;
    this.render();
  }

  update(_deltaMs: number): void {
    if (!this.visible) return;
    this.render();
  }

  destroy(): void {
    this.graphics.destroy();
  }

  private render(): void {
    const radius = this.baseRadius + RADIUS_PADDING;

    this.graphics.clear();
    this.graphics.setStrokeStyle({
      width: HIGHLIGHT_LINE_WIDTH,
      color: HIGHLIGHT_COLOR,
      alpha: HIGHLIGHT_ALPHA,
    });
    this.graphics.circle(this.positionX, this.positionY, radius);
    this.graphics.stroke();
  }
}

import { Graphics, Container } from 'pixi.js';
import type { IOverlay, OverlayConfig } from './IOverlay';

const FORCE_LINE_COLOR = 0x3366ff;
const FORCE_LINE_WIDTH = 0.15;
const FORCE_LINE_ALPHA = 0.8;
const FORCE_SCALE = 0.5;
const ARROW_HEAD_LENGTH = 0.5;
const ARROW_HEAD_ANGLE = Math.PI / 6;

export interface ForceData {
  x: number;
  y: number;
  radius: number;
  ax: number;
  ay: number;
}

export class ForceOverlay implements IOverlay {
  readonly config: OverlayConfig = {
    name: 'force',
    devToolsOnly: true,
    keyboardShortcut: 'f',
  };

  private graphics: Graphics;
  private visible: boolean = false;

  constructor(container: Container) {
    this.graphics = new Graphics();
    container.addChild(this.graphics);
  }

  update(data: ForceData | undefined): void {
    this.graphics.clear();

    if (!this.visible || !data) {
      return;
    }

    const magnitude = Math.sqrt(data.ax * data.ax + data.ay * data.ay);
    if (magnitude < 0.001) {
      return;
    }

    const dirX = data.ax / magnitude;
    const dirY = data.ay / magnitude;

    const originX = data.x + dirX * data.radius;
    const originY = data.y + dirY * data.radius;

    const endX = originX + data.ax * FORCE_SCALE;
    const endY = originY + data.ay * FORCE_SCALE;

    this.graphics.setStrokeStyle({
      width: FORCE_LINE_WIDTH,
      color: FORCE_LINE_COLOR,
      alpha: FORCE_LINE_ALPHA,
    });

    this.graphics.moveTo(originX, originY);
    this.graphics.lineTo(endX, endY);

    this.drawArrowHead(endX, endY, dirX, dirY);

    this.graphics.stroke();
  }

  private drawArrowHead(tipX: number, tipY: number, dirX: number, dirY: number): void {
    const angle = Math.atan2(dirY, dirX);

    const leftAngle = angle + Math.PI - ARROW_HEAD_ANGLE;
    const rightAngle = angle + Math.PI + ARROW_HEAD_ANGLE;

    const leftX = tipX + Math.cos(leftAngle) * ARROW_HEAD_LENGTH;
    const leftY = tipY + Math.sin(leftAngle) * ARROW_HEAD_LENGTH;

    const rightX = tipX + Math.cos(rightAngle) * ARROW_HEAD_LENGTH;
    const rightY = tipY + Math.sin(rightAngle) * ARROW_HEAD_LENGTH;

    this.graphics.moveTo(tipX, tipY);
    this.graphics.lineTo(leftX, leftY);
    this.graphics.moveTo(tipX, tipY);
    this.graphics.lineTo(rightX, rightY);
  }

  show(): void {
    this.visible = true;
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

  destroy(): void {
    this.graphics.destroy();
  }
}

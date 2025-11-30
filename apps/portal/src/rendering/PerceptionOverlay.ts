import { Graphics, Container } from 'pixi.js';
import type { PerceptionDebugData } from '../types/GameState';

const PERCEPTION_CIRCLE_COLOR = 0x00ffff;
const PERCEPTION_LINE_WIDTH = 0.8;
const PERCEPTION_ALPHA = 0.20;

const NEIGHBOR_LINE_COLOR = 0xffffff;
const NEIGHBOR_LINE_WIDTH = 0.1;
const NEIGHBOR_LINE_ALPHA = 0.89;

export class PerceptionOverlay {
  private graphics: Graphics;
  private visible: boolean = false;

  constructor(container: Container) {
    this.graphics = new Graphics();
    container.addChild(this.graphics);
  }

  update(debugData: PerceptionDebugData | undefined): void {
    if (!debugData) {
      this.clear();
      return;
    }

    this.visible = true;
    this.render(debugData);
  }

  clear(): void {
    this.visible = false;
    this.graphics.clear();
  }

  isVisible(): boolean {
    return this.visible;
  }

  destroy(): void {
    this.graphics.destroy();
  }

  private render(data: PerceptionDebugData): void {
    this.graphics.clear();

    this.graphics.setStrokeStyle({
      width: PERCEPTION_LINE_WIDTH,
      color: PERCEPTION_CIRCLE_COLOR,
      alpha: PERCEPTION_ALPHA,
    });
    this.graphics.circle(data.x, data.y, data.perceptionRange);
    this.graphics.stroke();

    if (data.neighbors.length > 0) {
      this.graphics.setStrokeStyle({
        width: NEIGHBOR_LINE_WIDTH,
        color: NEIGHBOR_LINE_COLOR,
        alpha: NEIGHBOR_LINE_ALPHA,
      });

      for (const neighbor of data.neighbors) {
        this.graphics.moveTo(data.x, data.y);
        this.graphics.lineTo(neighbor.x, neighbor.y);
      }
      this.graphics.stroke();
    }
  }
}

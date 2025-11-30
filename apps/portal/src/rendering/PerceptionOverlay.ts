import { Graphics, Container } from 'pixi.js';
import type { PerceptionDebugData } from '../types/GameState';

const PERCEPTION_WEDGE_COLOR = 0x00ffff;
const PERCEPTION_FILL_ALPHA = 0.15;

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

    const halfFov = data.fovAngle / 2;
    const startAngle = data.rotation - halfFov;
    const endAngle = data.rotation + halfFov;

    this.graphics.setFillStyle({
      color: PERCEPTION_WEDGE_COLOR,
      alpha: PERCEPTION_FILL_ALPHA,
    });
    this.graphics.moveTo(data.x, data.y);
    this.graphics.arc(data.x, data.y, data.perceptionRange, startAngle, endAngle);
    this.graphics.lineTo(data.x, data.y);
    this.graphics.fill();

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

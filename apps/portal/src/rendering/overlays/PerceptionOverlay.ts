import { Graphics, Container } from 'pixi.js';
import type { PerceptionDebugData } from '@/types/GameState';
import type { IOverlay, OverlayConfig } from './IOverlay';

const PERCEPTION_WEDGE_COLOR = 0x00ffff;
const PERCEPTION_FILL_ALPHA = 0.15;
const PERCEPTION_STROKE_WIDTH = 0.15;
const PERCEPTION_STROKE_ALPHA = 0.6;

const QUERY_RADIUS_COLOR = 0xff8800;
const QUERY_RADIUS_ALPHA = 0.25;
const QUERY_RADIUS_STROKE_WIDTH = 0.1;

const NEIGHBOR_LINE_COLOR = 0xffffff;
const NEIGHBOR_LINE_WIDTH = 0.1;
const NEIGHBOR_LINE_ALPHA = 0.89;

export class PerceptionOverlay implements IOverlay {
  readonly config: OverlayConfig = {
    name: 'perception',
    devToolsOnly: true,
    keyboardShortcut: 'p',
  };

  private graphics: Graphics;
  private visible: boolean = false;
  private hasData: boolean = false;

  constructor(container: Container) {
    this.graphics = new Graphics();
    container.addChild(this.graphics);
  }

  update(debugData: PerceptionDebugData | undefined): void {
    if (!debugData) {
      this.clear();
      return;
    }

    this.hasData = true;
    if (this.visible) {
      this.render(debugData);
    }
  }

  clear(): void {
    this.hasData = false;
    this.graphics.clear();
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
    return this.visible && this.hasData;
  }

  destroy(): void {
    this.graphics.destroy();
  }

  private render(data: PerceptionDebugData): void {
    this.graphics.clear();

    const halfFov = data.fovAngle / 2;
    const startAngle = data.rotation - halfFov;
    const endAngle = data.rotation + halfFov;

    // Draw query radius (outer, orange) - shows spatial query boundary
    this.graphics.setFillStyle({
      color: QUERY_RADIUS_COLOR,
      alpha: QUERY_RADIUS_ALPHA,
    });
    this.graphics.moveTo(data.x, data.y);
    this.graphics.arc(data.x, data.y, data.queryRadius, startAngle, endAngle);
    this.graphics.lineTo(data.x, data.y);
    this.graphics.fill();

    // Draw query radius stroke
    this.graphics.setStrokeStyle({
      width: QUERY_RADIUS_STROKE_WIDTH,
      color: QUERY_RADIUS_COLOR,
      alpha: QUERY_RADIUS_ALPHA + 0.2,
    });
    this.graphics.moveTo(data.x, data.y);
    this.graphics.arc(data.x, data.y, data.queryRadius, startAngle, endAngle);
    this.graphics.lineTo(data.x, data.y);
    this.graphics.stroke();

    // Draw perception range (inner, cyan) - shows biological sensing distance
    this.graphics.setFillStyle({
      color: PERCEPTION_WEDGE_COLOR,
      alpha: PERCEPTION_FILL_ALPHA,
    });
    this.graphics.moveTo(data.x, data.y);
    this.graphics.arc(data.x, data.y, data.perceptionRange, startAngle, endAngle);
    this.graphics.lineTo(data.x, data.y);
    this.graphics.fill();

    // Draw perception range stroke
    this.graphics.setStrokeStyle({
      width: PERCEPTION_STROKE_WIDTH,
      color: PERCEPTION_WEDGE_COLOR,
      alpha: PERCEPTION_STROKE_ALPHA,
    });
    this.graphics.moveTo(data.x, data.y);
    this.graphics.arc(data.x, data.y, data.perceptionRange, startAngle, endAngle);
    this.graphics.lineTo(data.x, data.y);
    this.graphics.stroke();

    // Draw lines to neighbors
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

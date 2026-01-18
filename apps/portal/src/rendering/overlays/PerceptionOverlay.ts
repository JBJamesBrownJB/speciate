import { Graphics, Container } from 'pixi.js';
import type { PerceptionDebugData } from '@/types/GameState';
import type { IOverlay, OverlayConfig } from './IOverlay';

const PERCEPTION_WEDGE_COLOR = 0x00ffff;
const PERCEPTION_FILL_ALPHA = 0.15;
const PERCEPTION_STROKE_WIDTH = 0.15;
const PERCEPTION_STROKE_ALPHA = 0.6;

const NEIGHBOR_LINE_COLOR = 0xffffff;
const NEIGHBOR_LINE_WIDTH = 0.1;
const NEIGHBOR_LINE_ALPHA = 0.89;

const L1_VISION_LINE_COLOR = 0x888888;
const L1_VISION_LINE_WIDTH = 0.15;
const L1_VISION_LINE_ALPHA = 0.6;

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

    // Draw perception range (cyan) - shows biological sensing distance
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

    // Draw L1 vision lines to L1 cell centers (gray for now - not used for behavior yet)
    if (data.l1Vision && data.l1Vision.length > 0) {
      this.graphics.setStrokeStyle({
        width: L1_VISION_LINE_WIDTH,
        color: L1_VISION_LINE_COLOR,
        alpha: L1_VISION_LINE_ALPHA,
      });

      for (const entry of data.l1Vision) {
        this.graphics.moveTo(data.x, data.y);
        this.graphics.lineTo(entry.centerX, entry.centerY);
      }
      this.graphics.stroke();
    }
  }
}

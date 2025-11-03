import { Graphics, Text, Container } from 'pixi.js';

export interface DiagnosticData {
  fps: number;
  ping: number;
  tick: number;
  creatureCount: number;
  interpolationAlpha?: number;
  worldBounds?: { width: number; height: number };
  serverTime?: number;
  clientTime?: number;
}

export class DiagnosticOverlay {
  private container: Container;
  private fpsText: Text;
  private statsText: Text;
  private boundaryBox: Graphics;
  private grid: Graphics;
  private boundaryLabels: {
    tl: Text;
    tr: Text;
    br: Text;
  } | null = null;
  private enabled: boolean = true;

  constructor(stage: Container) {
    this.container = new Container();
    stage.addChild(this.container);

    this.fpsText = this.createFPSText();
    this.statsText = this.createStatsText();
    this.boundaryBox = new Graphics();
    this.grid = new Graphics();

    this.container.addChild(this.fpsText);
    this.container.addChild(this.statsText);
    this.container.addChild(this.boundaryBox);
    this.container.addChild(this.grid);
  }

  toggle(): void {
    this.enabled = !this.enabled;
    this.container.visible = this.enabled;
  }

  update(data: DiagnosticData): void {
    if (!this.enabled) return;

    this.updateFPS(data.fps);
    this.updateStats(data);
  }

  drawWorldBoundary(x: number, y: number, width: number, height: number): void {
    if (!this.enabled) return;

    this.boundaryBox.clear();
    this.boundaryBox.rect(x, y, width, height);
    this.boundaryBox.stroke({ color: 0xff6b6b, width: 2, alpha: 0.8 });
    this.boundaryBox.circle(x + width / 2, y + height / 2, 5);
    this.boundaryBox.fill({ color: 0x4ecdc4, alpha: 0.5 });

    this.updateBoundaryLabels(x, y, width, height);
  }

  drawGrid(x: number, y: number, width: number, height: number, cellSize: number = 50): void {
    if (!this.enabled) return;

    this.grid.clear();

    for (let i = 0; i <= width; i += cellSize) {
      this.grid.moveTo(x + i, y);
      this.grid.lineTo(x + i, y + height);
    }

    for (let i = 0; i <= height; i += cellSize) {
      this.grid.moveTo(x, y + i);
      this.grid.lineTo(x + width, y + i);
    }

    this.grid.stroke({ width: 1, color: 0x333333, alpha: 0.3 });
  }

  destroy(): void {
    this.container.destroy({ children: true });
  }

  private createFPSText(): Text {
    const text = new Text({
      text: 'FPS: 60',
      style: {
        fontFamily: 'Monaco, monospace',
        fontSize: 16,
        fill: 0x51cf66,
        align: 'left',
      }
    });
    text.position.set(10, 10);
    return text;
  }

  private createStatsText(): Text {
    const text = new Text({
      text: '',
      style: {
        fontFamily: 'Monaco, monospace',
        fontSize: 12,
        fill: 0xaaaaaa,
        align: 'left',
        lineHeight: 18,
      }
    });
    text.position.set(10, 35);
    return text;
  }

  private updateFPS(fps: number): void {
    const fpsColor = fps >= 55 ? 0x51cf66 : fps >= 30 ? 0xf8b500 : 0xff6b6b;
    this.fpsText.text = `FPS: ${fps}`;
    this.fpsText.style.fill = fpsColor;
  }

  private updateStats(data: DiagnosticData): void {
    const lines = [
      `Ping: ${data.ping}ms`,
      `Tick: ${data.tick}`,
      `Creatures: ${data.creatureCount}`,
    ];

    if (data.interpolationAlpha !== undefined) {
      lines.push(`Interp α: ${data.interpolationAlpha.toFixed(3)}`);
    }

    if (data.worldBounds) {
      lines.push(`World: ${data.worldBounds.width} × ${data.worldBounds.height}`);
    }

    if (data.serverTime && data.clientTime) {
      const drift = data.clientTime - data.serverTime;
      lines.push(`Time drift: ${drift}ms`);
    }

    this.statsText.text = lines.join('\n');
  }

  private updateBoundaryLabels(x: number, y: number, width: number, height: number): void {
    if (!this.boundaryLabels) {
      this.boundaryLabels = this.createBoundaryLabels();
      this.container.addChild(this.boundaryLabels.tl);
      this.container.addChild(this.boundaryLabels.tr);
      this.container.addChild(this.boundaryLabels.br);
    }

    this.boundaryLabels.tl.text = '(0, 0)';
    this.boundaryLabels.tl.position.set(x + 5, y + 5);

    this.boundaryLabels.tr.text = `(${width.toFixed(0)}, 0)`;
    this.boundaryLabels.tr.position.set(x + width - 60, y + 5);

    this.boundaryLabels.br.text = `(${width.toFixed(0)}, ${height.toFixed(0)})`;
    this.boundaryLabels.br.position.set(x + width - 80, y + height - 20);
  }

  private createBoundaryLabels() {
    const labelStyle = {
      fontFamily: 'Monaco, monospace',
      fontSize: 10,
      fill: 0xff6b6b,
    };

    return {
      tl: new Text({ text: '', style: labelStyle }),
      tr: new Text({ text: '', style: labelStyle }),
      br: new Text({ text: '', style: labelStyle }),
    };
  }
}

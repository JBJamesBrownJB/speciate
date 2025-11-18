import { RENDERING_CONFIG } from "@/core/constants";

export class FPSSparkline {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private fpsHistory: number[] = [];
  private maxHistory = RENDERING_CONFIG.TARGET_FPS;

  constructor(canvasId: string) {
    const canvas = document.getElementById(canvasId) as HTMLCanvasElement;
    if (!canvas) throw new Error(`Canvas ${canvasId} not found`);

    this.canvas = canvas;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("Could not get 2d context");
    this.ctx = ctx;

    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    this.ctx.scale(dpr, dpr);
  }

  update(fps: number): void {
    this.fpsHistory.push(fps);
    if (this.fpsHistory.length > this.maxHistory) {
      this.fpsHistory.shift();
    }
    this.render();
  }

  private render(): void {
    const width = this.canvas.width / (window.devicePixelRatio || 1);
    const height = this.canvas.height / (window.devicePixelRatio || 1);

    this.ctx.clearRect(0, 0, width, height);

    if (this.fpsHistory.length < 2) return;

    const maxFPS = RENDERING_CONFIG.TARGET_FPS;
    const minFPS = 0;

    this.ctx.beginPath();
    this.ctx.strokeStyle = "#6fb83f";
    this.ctx.lineWidth = 1.5;

    const xStep = width / (this.maxHistory - 1);

    this.fpsHistory.forEach((fps, i) => {
      const x = i * xStep;
      const normalizedFPS = Math.max(minFPS, Math.min(maxFPS, fps));
      const y = height - (normalizedFPS / maxFPS) * height;

      if (i === 0) {
        this.ctx.moveTo(x, y);
      } else {
        this.ctx.lineTo(x, y);
      }
    });

    this.ctx.stroke();

    this.ctx.beginPath();
    this.ctx.strokeStyle = "rgba(111, 184, 63, 0.3)";
    this.ctx.lineWidth = 1;
    this.ctx.setLineDash([2, 2]);
    this.ctx.moveTo(0, 0);
    this.ctx.lineTo(width, 0);
    this.ctx.stroke();
    this.ctx.setLineDash([]);
  }
}

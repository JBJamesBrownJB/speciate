import { Application, Graphics } from 'pixi.js';

export class PixiApp {
  public app: Application | null = null;
  private container: HTMLElement;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  public async init(): Promise<void> {
    this.app = new Application();
    await this.app.init({
      width: window.innerWidth,
      height: window.innerHeight,
      backgroundColor: 0x000000,
      resolution: window.devicePixelRatio || 1,
      autoDensity: true,
      antialias: true,
    });

    this.container.appendChild(this.app.canvas);
    this.setupResizeHandler();
  }

  public destroy(): void {
    if (this.app) {
      this.app.destroy(true);
      this.app = null;
    }
  }

  public getStage() {
    if (!this.app) throw new Error('Application not initialized');
    return this.app.stage;
  }

  public getDimensions(): { width: number; height: number } {
    if (!this.app) throw new Error('Application not initialized');
    return {
      width: this.app.renderer.width,
      height: this.app.renderer.height,
    };
  }

  private setupResizeHandler(): void {
    const handleResize = () => {
      if (!this.app) return;
      const width = window.innerWidth;
      const height = window.innerHeight;
      this.app.renderer.resize(width, height);
    };
    window.addEventListener('resize', handleResize);
  }

  public createGraphics(): Graphics {
    return new Graphics();
  }
}

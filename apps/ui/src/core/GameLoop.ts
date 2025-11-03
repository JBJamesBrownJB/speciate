export type UpdateCallback = (deltaTime: number, time: number) => void;

export class GameLoop {
  private animationFrameId: number | null = null;
  private isRunning: boolean = false;
  private lastTime: number = 0;
  private updateCallbacks: Set<UpdateCallback> = new Set();
  private frameCount: number = 0;
  private fpsUpdateTime: number = 0;
  private currentFPS: number = 0;
  private readonly FPS_UPDATE_INTERVAL = 500;

  public start(): void {
    if (this.isRunning) return;
    this.isRunning = true;
    this.lastTime = performance.now();
    this.fpsUpdateTime = this.lastTime;
    this.frameCount = 0;
    this.tick(this.lastTime);
  }

  public stop(): void {
    if (!this.isRunning) return;
    this.isRunning = false;
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
  }

  public onUpdate(callback: UpdateCallback): () => void {
    this.updateCallbacks.add(callback);
    return () => this.updateCallbacks.delete(callback);
  }

  public getFPS(): number {
    return this.currentFPS;
  }

  private tick(currentTime: number): void {
    if (!this.isRunning) return;
    const deltaTime = (currentTime - this.lastTime) / 1000;
    this.lastTime = currentTime;

    this.frameCount++;
    if (currentTime - this.fpsUpdateTime >= this.FPS_UPDATE_INTERVAL) {
      const elapsed = (currentTime - this.fpsUpdateTime) / 1000;
      this.currentFPS = Math.round(this.frameCount / elapsed);
      this.frameCount = 0;
      this.fpsUpdateTime = currentTime;
    }

    this.updateCallbacks.forEach(callback => {
      try {
        callback(deltaTime, currentTime);
      } catch (error) {
        console.error('[GameLoop] Error in update callback:', error);
      }
    });

    this.animationFrameId = requestAnimationFrame((time) => this.tick(time));
  }
}

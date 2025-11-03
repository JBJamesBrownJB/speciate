import * as PIXI from 'pixi.js';

export class SpritePool {
  private available: PIXI.Graphics[] = [];
  private inUse = new Map<string, PIXI.Graphics>();

  acquire(id: string, color: number, radius: number): PIXI.Graphics {
    const existing = this.inUse.get(id);
    if (existing) return existing;

    const sprite = this.available.pop() || this.createSprite();
    this.updateSprite(sprite, color, radius);
    this.inUse.set(id, sprite);

    return sprite;
  }

  release(id: string): void {
    const sprite = this.inUse.get(id);
    if (!sprite) return;

    this.inUse.delete(id);
    sprite.visible = false;
    this.available.push(sprite);
  }

  releaseAll(): void {
    for (const [id] of this.inUse) {
      this.release(id);
    }
  }

  destroy(): void {
    this.releaseAll();

    for (const sprite of this.available) {
      sprite.destroy();
    }

    this.available = [];
  }

  private createSprite(): PIXI.Graphics {
    return new PIXI.Graphics();
  }

  private updateSprite(
    sprite: PIXI.Graphics,
    color: number,
    radius: number
  ): void {
    sprite.clear();
    sprite.circle(0, 0, radius);
    sprite.fill(color);
    sprite.visible = true;
  }
}

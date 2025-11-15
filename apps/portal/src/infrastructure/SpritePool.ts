import { Sprite, Texture } from 'pixi.js';

export class SpritePool {
  private pool = new Map<number, Sprite>();
  private active = new Set<number>();

  acquire(entityId: number, texture: Texture): Sprite {
    let sprite = this.pool.get(entityId);
    if (!sprite) {
      sprite = new Sprite(texture);
      sprite.anchor.set(0.5, 0.5);
      this.pool.set(entityId, sprite);
    }
    this.active.add(entityId);
    return sprite;
  }

  release(entityId: number): void {
    this.active.delete(entityId);
    const sprite = this.pool.get(entityId);
    if (sprite?.parent) {
      sprite.parent.removeChild(sprite);
    }
  }

  releaseAll(): void {
    for (const entityId of this.active) {
      const sprite = this.pool.get(entityId);
      if (sprite?.parent) {
        sprite.parent.removeChild(sprite);
      }
    }
    this.active.clear();
  }

  isActive(entityId: number): boolean {
    return this.active.has(entityId);
  }

  getPoolSize(): number {
    return this.pool.size;
  }

  getActiveCount(): number {
    return this.active.size;
  }

  getActiveIds(): number[] {
    return Array.from(this.active);
  }
}

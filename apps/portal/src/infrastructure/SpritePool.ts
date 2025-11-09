import { Sprite, Texture } from 'pixi.js';

/**
 * Object pool for managing Sprite instances to avoid creating/destroying sprites every frame.
 * This is MANDATORY for performance with 1000+ entities.
 *
 * Usage:
 * - acquire(entityId, texture) - Get a sprite for an entity (creates new or reuses existing)
 * - release(entityId) - Mark sprite as inactive and remove from scene (but keep in pool)
 * - releaseAll() - Release all active sprites
 */
export class SpritePool {
  private pool = new Map<number, Sprite>();
  private active = new Set<number>();

  /**
   * Get a sprite for an entity ID. Creates new sprite if needed, or reuses existing.
   * @param entityId Unique entity identifier
   * @param texture Texture to use for the sprite
   * @returns Sprite instance (may be new or reused)
   */
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

  /**
   * Release a sprite back to the pool. Removes from scene but keeps in pool for reuse.
   * @param entityId Entity ID to release
   */
  release(entityId: number): void {
    this.active.delete(entityId);
    const sprite = this.pool.get(entityId);
    if (sprite?.parent) {
      sprite.parent.removeChild(sprite);
    }
  }

  /**
   * Release all active sprites
   */
  releaseAll(): void {
    for (const entityId of this.active) {
      const sprite = this.pool.get(entityId);
      if (sprite?.parent) {
        sprite.parent.removeChild(sprite);
      }
    }
    this.active.clear();
  }

  /**
   * Check if a sprite is currently active
   * @param entityId Entity ID to check
   * @returns True if sprite is active
   */
  isActive(entityId: number): boolean {
    return this.active.has(entityId);
  }

  /**
   * Get total number of sprites in the pool (active + inactive)
   * @returns Total pool size
   */
  getPoolSize(): number {
    return this.pool.size;
  }

  /**
   * Get number of currently active sprites
   * @returns Active sprite count
   */
  getActiveCount(): number {
    return this.active.size;
  }

  /**
   * Get all currently active entity IDs
   * @returns Array of active entity IDs
   */
  getActiveIds(): number[] {
    return Array.from(this.active);
  }
}

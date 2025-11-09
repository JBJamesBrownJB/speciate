import { describe, it, expect, vi, beforeEach } from 'vitest';
import { SpritePool } from './SpritePool';
import { Texture, Sprite } from 'pixi.js';

describe('SpritePool', () => {
  let pool: SpritePool;
  let mockTexture: Texture;

  beforeEach(() => {
    pool = new SpritePool();
    // Create a mock texture
    mockTexture = {} as Texture;
  });

  describe('acquire', () => {
    it('should create a new sprite for a new entity ID', () => {
      const sprite = pool.acquire(1, mockTexture);

      expect(sprite).toBeInstanceOf(Sprite);
      // Sprite is created with the texture (Pixi.js may create actual Texture object)
      expect(sprite.texture).toBeTruthy();
    });

    it('should set sprite anchor to center (0.5, 0.5)', () => {
      const sprite = pool.acquire(1, mockTexture);

      expect(sprite.anchor.x).toBe(0.5);
      expect(sprite.anchor.y).toBe(0.5);
    });

    it('should reuse existing sprite for same entity ID', () => {
      const sprite1 = pool.acquire(1, mockTexture);
      const sprite2 = pool.acquire(1, mockTexture);

      expect(sprite2).toBe(sprite1);
    });

    it('should mark sprite as active', () => {
      pool.acquire(1, mockTexture);

      expect(pool.isActive(1)).toBe(true);
    });

    it('should create different sprites for different entity IDs', () => {
      const sprite1 = pool.acquire(1, mockTexture);
      const sprite2 = pool.acquire(2, mockTexture);

      expect(sprite2).not.toBe(sprite1);
    });

    it('should handle multiple acquires and releases correctly', () => {
      const sprite1 = pool.acquire(1, mockTexture);
      pool.release(1);
      const sprite2 = pool.acquire(1, mockTexture);

      // Should reuse the same sprite
      expect(sprite2).toBe(sprite1);
    });
  });

  describe('release', () => {
    it('should mark sprite as inactive', () => {
      pool.acquire(1, mockTexture);
      pool.release(1);

      expect(pool.isActive(1)).toBe(false);
    });

    it('should remove sprite from parent container if it has one', () => {
      const sprite = pool.acquire(1, mockTexture);
      const mockParent = {
        removeChild: vi.fn()
      };
      (sprite as any).parent = mockParent;

      pool.release(1);

      expect(mockParent.removeChild).toHaveBeenCalledWith(sprite);
    });

    it('should not throw if sprite has no parent', () => {
      pool.acquire(1, mockTexture);

      expect(() => pool.release(1)).not.toThrow();
    });

    it('should handle releasing non-existent entity ID', () => {
      expect(() => pool.release(999)).not.toThrow();
    });

    it('should keep sprite in pool for reuse after release', () => {
      const sprite1 = pool.acquire(1, mockTexture);
      pool.release(1);
      const sprite2 = pool.acquire(1, mockTexture);

      expect(sprite2).toBe(sprite1);
    });
  });

  describe('isActive', () => {
    it('should return false for new entity ID', () => {
      expect(pool.isActive(1)).toBe(false);
    });

    it('should return true after acquire', () => {
      pool.acquire(1, mockTexture);

      expect(pool.isActive(1)).toBe(true);
    });

    it('should return false after release', () => {
      pool.acquire(1, mockTexture);
      pool.release(1);

      expect(pool.isActive(1)).toBe(false);
    });
  });

  describe('releaseAll', () => {
    it('should release all active sprites', () => {
      pool.acquire(1, mockTexture);
      pool.acquire(2, mockTexture);
      pool.acquire(3, mockTexture);

      pool.releaseAll();

      expect(pool.isActive(1)).toBe(false);
      expect(pool.isActive(2)).toBe(false);
      expect(pool.isActive(3)).toBe(false);
    });

    it('should remove all sprites from their parents', () => {
      const sprite1 = pool.acquire(1, mockTexture);
      const sprite2 = pool.acquire(2, mockTexture);
      const mockParent = {
        removeChild: vi.fn()
      };
      (sprite1 as any).parent = mockParent;
      (sprite2 as any).parent = mockParent;

      pool.releaseAll();

      expect(mockParent.removeChild).toHaveBeenCalledWith(sprite1);
      expect(mockParent.removeChild).toHaveBeenCalledWith(sprite2);
    });

    it('should not throw when pool is empty', () => {
      expect(() => pool.releaseAll()).not.toThrow();
    });
  });

  describe('getPoolSize', () => {
    it('should return 0 for empty pool', () => {
      expect(pool.getPoolSize()).toBe(0);
    });

    it('should return correct count after acquiring sprites', () => {
      pool.acquire(1, mockTexture);
      pool.acquire(2, mockTexture);
      pool.acquire(3, mockTexture);

      expect(pool.getPoolSize()).toBe(3);
    });

    it('should maintain size after release (sprites stay in pool)', () => {
      pool.acquire(1, mockTexture);
      pool.acquire(2, mockTexture);
      pool.release(1);

      expect(pool.getPoolSize()).toBe(2);
    });
  });

  describe('getActiveCount', () => {
    it('should return 0 for empty pool', () => {
      expect(pool.getActiveCount()).toBe(0);
    });

    it('should return correct count of active sprites', () => {
      pool.acquire(1, mockTexture);
      pool.acquire(2, mockTexture);
      pool.acquire(3, mockTexture);

      expect(pool.getActiveCount()).toBe(3);
    });

    it('should decrease after release', () => {
      pool.acquire(1, mockTexture);
      pool.acquire(2, mockTexture);
      pool.release(1);

      expect(pool.getActiveCount()).toBe(1);
    });

    it('should be 0 after releaseAll', () => {
      pool.acquire(1, mockTexture);
      pool.acquire(2, mockTexture);
      pool.releaseAll();

      expect(pool.getActiveCount()).toBe(0);
    });
  });
});

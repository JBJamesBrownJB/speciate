import { describe, it, expect, beforeEach } from 'vitest';
import { SpritePool } from './SpritePool';

describe('SpritePool', () => {
  let pool: SpritePool;

  beforeEach(() => {
    pool = new SpritePool();
  });

  it('should create new sprite when pool is empty', () => {
    const sprite = pool.acquire('entity1', 0xff0000, 10);
    expect(sprite).toBeDefined();
    expect(sprite.visible).toBe(true);
  });

  it('should reuse released sprites', () => {
    const sprite1 = pool.acquire('entity1', 0xff0000, 10);
    pool.release('entity1');

    const sprite2 = pool.acquire('entity2', 0x00ff00, 10);
    expect(sprite1).toBe(sprite2);
  });

  it('should return same sprite for same id', () => {
    const sprite1 = pool.acquire('entity1', 0xff0000, 10);
    const sprite2 = pool.acquire('entity1', 0xff0000, 10);
    expect(sprite1).toBe(sprite2);
  });

  it('should hide sprite when released', () => {
    const sprite = pool.acquire('entity1', 0xff0000, 10);
    pool.release('entity1');
    expect(sprite.visible).toBe(false);
  });

  it('should release all sprites', () => {
    pool.acquire('entity1', 0xff0000, 10);
    pool.acquire('entity2', 0x00ff00, 10);

    pool.releaseAll();

    const sprite = pool.acquire('entity3', 0x0000ff, 10);
    expect(sprite).toBeDefined();
  });
});

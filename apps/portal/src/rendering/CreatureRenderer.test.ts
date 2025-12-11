import { describe, it, expect, beforeEach, vi } from 'vitest';
import { CreatureRenderer } from './CreatureRenderer';
import type { ParticlePool } from '@/infrastructure/ParticlePool';
import type { ParticleContainer, Texture } from 'pixi.js';
import type { CreatureData } from '@/types/GameState';
import { createMockTexture, createMockParticle } from '@/test-utils';

describe('CreatureRenderer', () => {
  let mockParticlePool: ParticlePool;
  let mockParticleContainer: ParticleContainer;
  let mockTexture: Texture;
  let renderer: CreatureRenderer;

  beforeEach(() => {
    mockTexture = createMockTexture(32, 32);

    mockParticlePool = {
      acquire: vi.fn(),
      hasEntity: vi.fn(),
      beginFrame: vi.fn(),
      getStaleEntities: vi.fn().mockReturnValue([]),
      removeEntity: vi.fn(),
    } as unknown as ParticlePool;

    mockParticleContainer = {
      addParticle: vi.fn(),
      removeParticle: vi.fn(),
    } as unknown as ParticleContainer;

    renderer = new CreatureRenderer(
      mockParticlePool,
      mockParticleContainer,
      mockTexture
    );
  });

  describe('constructor', () => {
    it('should store dependencies', () => {
      expect(renderer).toBeDefined();
    });
  });

  describe('render', () => {
    describe('new creatures', () => {
      it('should acquire particle from pool for new creature', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(false);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 100, y: 200, rotation: 0.5, size: 10 },
        ];

        renderer.render(creatures);

        expect(mockParticlePool.acquire).toHaveBeenCalledWith(1, mockTexture);
      });

      it('should add new particle to container', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(false);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 100, y: 200, rotation: 0.5, size: 10 },
        ];

        renderer.render(creatures);

        expect(mockParticleContainer.addParticle).toHaveBeenCalledWith(mockParticle);
      });

      it('should set particle position from creature data', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(false);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 100, y: 200, rotation: 0.5, size: 10 },
        ];

        renderer.render(creatures);

        expect(mockParticle.x).toBe(100);
        expect(mockParticle.y).toBe(200);
      });

      it('should set particle rotation from creature data', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(false);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 100, y: 200, rotation: 1.57, size: 10 },
        ];

        renderer.render(creatures);

        expect(mockParticle.rotation).toBe(1.57);
      });
    });

    describe('existing creatures', () => {
      it('should not re-add existing creature to container', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(true);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 100, y: 200, rotation: 0.5, size: 10 },
        ];

        renderer.render(creatures);

        expect(mockParticleContainer.addParticle).not.toHaveBeenCalled();
      });

      it('should update position for existing creature', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(true);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 300, y: 400, rotation: 0.5, size: 10 },
        ];

        renderer.render(creatures);

        expect(mockParticle.x).toBe(300);
        expect(mockParticle.y).toBe(400);
      });
    });

    describe('scale calculation', () => {
      it('should calculate scale for square creatures', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(false);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 100, y: 200, rotation: 0, size: 16 },
        ];

        renderer.render(creatures);

        expect(mockParticle.scaleX).toBe(0.5);
        expect(mockParticle.scaleY).toBe(0.5);
      });

      it('should calculate scale for medium creatures', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(false);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 100, y: 200, rotation: 0, size: 64 },
        ];

        renderer.render(creatures);

        expect(mockParticle.scaleX).toBe(2);
        expect(mockParticle.scaleY).toBe(2);
      });

      it('should handle very small creatures', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(false);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 100, y: 200, rotation: 0, size: 1 },
        ];

        renderer.render(creatures);

        expect(mockParticle.scaleX).toBeCloseTo(0.03125);
        expect(mockParticle.scaleY).toBeCloseTo(0.03125);
      });

      it('should handle very large creatures', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(false);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 100, y: 200, rotation: 0, size: 320 },
        ];

        renderer.render(creatures);

        expect(mockParticle.scaleX).toBe(10);
        expect(mockParticle.scaleY).toBe(10);
      });
    });

    describe('stale entity cleanup', () => {
      it('should detect stale entities', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(false);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 100, y: 200, rotation: 0, size: 10 },
          { id: 2, x: 150, y: 250, rotation: 0, size: 10 },
        ];

        renderer.render(creatures);

        expect(mockParticlePool.beginFrame).toHaveBeenCalled();
        expect(mockParticlePool.getStaleEntities).toHaveBeenCalled();
      });

      it('should remove stale entities from pool', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([99]);
        vi.mocked(mockParticlePool.removeEntity).mockReturnValue(mockParticle);

        const creatures: CreatureData[] = [];

        renderer.render(creatures);

        expect(mockParticlePool.removeEntity).toHaveBeenCalledWith(99);
      });

      it('should remove stale particles from container', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([99]);
        vi.mocked(mockParticlePool.removeEntity).mockReturnValue(mockParticle);

        const creatures: CreatureData[] = [];

        renderer.render(creatures);

        expect(mockParticleContainer.removeParticle).toHaveBeenCalledWith(mockParticle);
      });

      it('should handle null particle from removeEntity', () => {
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([99]);
        vi.mocked(mockParticlePool.removeEntity).mockReturnValue(undefined);

        const creatures: CreatureData[] = [];

        expect(() => renderer.render(creatures)).not.toThrow();
        expect(mockParticleContainer.removeParticle).not.toHaveBeenCalled();
      });

      it('should remove multiple stale entities', () => {
        const mockParticle1 = createMockParticle();
        const mockParticle2 = createMockParticle();
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([98, 99]);
        vi.mocked(mockParticlePool.removeEntity)
          .mockReturnValueOnce(mockParticle1)
          .mockReturnValueOnce(mockParticle2);

        const creatures: CreatureData[] = [];

        renderer.render(creatures);

        expect(mockParticlePool.removeEntity).toHaveBeenCalledTimes(2);
        expect(mockParticleContainer.removeParticle).toHaveBeenCalledTimes(2);
      });
    });

    describe('edge cases', () => {
      it('should handle empty creature list', () => {
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [];

        expect(() => renderer.render(creatures)).not.toThrow();
        expect(mockParticlePool.acquire).not.toHaveBeenCalled();
      });

      it('should handle many creatures', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(false);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = Array.from({ length: 100 }, (_, i) => ({
          id: i,
          x: i * 10,
          y: i * 10,
          rotation: 0,
          size: 10,
        }));

        renderer.render(creatures);

        expect(mockParticlePool.acquire).toHaveBeenCalledTimes(100);
        expect(mockParticleContainer.addParticle).toHaveBeenCalledTimes(100);
      });

      it('should handle mixed new and existing creatures', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity)
          .mockReturnValueOnce(true)
          .mockReturnValueOnce(false);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 100, y: 200, rotation: 0, size: 10 },
          { id: 2, x: 150, y: 250, rotation: 0, size: 10 },
        ];

        renderer.render(creatures);

        expect(mockParticleContainer.addParticle).toHaveBeenCalledTimes(1);
      });

      it('should handle creatures with zero dimensions', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(false);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 100, y: 200, rotation: 0, size: 0 },
        ];

        renderer.render(creatures);

        expect(mockParticle.scaleX).toBe(0);
        expect(mockParticle.scaleY).toBe(0);
      });

      it('should handle negative rotation values', () => {
        const mockParticle = createMockParticle();
        vi.mocked(mockParticlePool.hasEntity).mockReturnValue(false);
        vi.mocked(mockParticlePool.acquire).mockReturnValue(mockParticle);
        vi.mocked(mockParticlePool.getStaleEntities).mockReturnValue([]);

        const creatures: CreatureData[] = [
          { id: 1, x: 100, y: 200, rotation: -1.57, size: 10 },
        ];

        renderer.render(creatures);

        expect(mockParticle.rotation).toBe(-1.57);
      });
    });
  });
});

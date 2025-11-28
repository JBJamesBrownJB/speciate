import { describe, it, expect, beforeEach } from "vitest";
import { Texture } from "pixi.js";
import { InterpolatedCreatureRenderer } from "./InterpolatedCreatureRenderer";
import type { CreatureData } from "@/types/GameState";

describe("InterpolatedCreatureRenderer", () => {
  let renderer: InterpolatedCreatureRenderer;
  let mockTexture: Texture;

  beforeEach(() => {
    // Create a minimal mock texture with source that PixiJS v8 recognizes as a TextureSource
    // TextureSource objects need uid and resourceType to be recognized
    mockTexture = {
      width: 32,
      height: 32,
      source: {
        width: 32,
        height: 32,
        uid: 1,
        _resourceType: "textureSource",
        _resourceId: 1,
        destroyed: false,
      },
    } as unknown as Texture;

    renderer = new InterpolatedCreatureRenderer(mockTexture, 1000);
  });

  describe("initialization", () => {
    it("should create renderer with geometry and mesh", () => {
      expect(renderer).toBeDefined();
      expect(renderer.getMesh()).toBeDefined();
    });

    it("should initialize with zero creatures", () => {
      expect(renderer.getCreatureCount()).toBe(0);
    });

    it("should have interpolation uniforms", () => {
      const uniforms = renderer.getUniforms();
      expect(uniforms).toHaveProperty("uInterpolation");
      expect(uniforms.uInterpolation).toBe(0.0);
    });
  });

  describe("rendering creatures", () => {
    it("should initialize with creatures on first render", () => {
      const creatures: CreatureData[] = [
        { id: 1, x: 100, y: 50, rotation: 0, size: 10 },
      ];

      renderer.initialize(creatures);

      expect(renderer.getCreatureCount()).toBe(1);
    });

    it("should update on simulation tick", () => {
      const creatures: CreatureData[] = [
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
      ];
      renderer.initialize(creatures);

      const newCreatures: CreatureData[] = [
        { id: 1, x: 100, y: 50, rotation: 1.5, size: 10 },
      ];
      renderer.onSimulationTick(newCreatures);

      expect(renderer.getCreatureCount()).toBe(1);
    });

    it("should handle multiple creatures", () => {
      const creatures: CreatureData[] = [
        { id: 1, x: 100, y: 50, rotation: 0, size: 10 },
        { id: 2, x: 200, y: 75, rotation: 0.5, size: 12 },
        { id: 3, x: 300, y: 100, rotation: 1.0, size: 15 },
      ];

      renderer.initialize(creatures);

      expect(renderer.getCreatureCount()).toBe(3);
    });
  });

  describe("interpolation", () => {
    it("should start with interpolation alpha at 0.0", () => {
      const uniforms = renderer.getUniforms();
      expect(uniforms.uInterpolation).toBe(0.0);
    });

    it("should update interpolation alpha on render", () => {
      const creatures: CreatureData[] = [
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
      ];
      renderer.initialize(creatures);
      renderer.setTickRate(20.0); // Set tick rate (required for interpolation to work)

      // Simulate render frame (16.67ms @ 60 FPS)
      // Pass camera parameters: cameraX, cameraY, zoom, width, height
      renderer.render(16.67, 0, 0, 10, 800, 600);

      const uniforms = renderer.getUniforms();
      // Alpha should be deltaMS / tickInterval
      // 16.67ms / 50ms = ~0.33
      expect(uniforms.uInterpolation).toBeGreaterThan(0.0);
      expect(uniforms.uInterpolation).toBeLessThan(1.0);
    });

    it("should reset interpolation on simulation tick", () => {
      const creatures: CreatureData[] = [
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
      ];
      renderer.initialize(creatures);
      renderer.setTickRate(20.0); // Set tick rate (required for interpolation to work)

      // Advance interpolation
      renderer.render(16.67, 0, 0, 10, 800, 600);
      expect(renderer.getUniforms().uInterpolation).toBeGreaterThan(0.0);

      // Simulation tick should reset
      renderer.onSimulationTick([
        { id: 1, x: 100, y: 50, rotation: 1.0, size: 10 },
      ]);

      expect(renderer.getUniforms().uInterpolation).toBe(0.0);
    });

    it("should clamp interpolation to [0, 1] range", () => {
      const creatures: CreatureData[] = [
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
      ];
      renderer.initialize(creatures);

      // Render with huge delta (simulate lag)
      renderer.render(1000, 0, 0, 10, 800, 600); // 1 second

      const uniforms = renderer.getUniforms();
      expect(uniforms.uInterpolation).toBeLessThanOrEqual(1.0);
    });
  });

  describe("creature count changes", () => {
    it("should handle spawning new creatures", () => {
      renderer.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      renderer.onSimulationTick([
        { id: 1, x: 10, y: 10, rotation: 0, size: 10 },
        { id: 2, x: 20, y: 20, rotation: 0.5, size: 12 },
      ]);

      expect(renderer.getCreatureCount()).toBe(2);
    });

    it("should handle despawning creatures", () => {
      renderer.initialize([
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 2, x: 100, y: 100, rotation: 0, size: 10 },
      ]);

      renderer.onSimulationTick([{ id: 1, x: 10, y: 10, rotation: 0, size: 10 }]);

      expect(renderer.getCreatureCount()).toBe(1);
    });

    it("should handle complete population change", () => {
      renderer.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      const newCreatures: CreatureData[] = [];
      for (let i = 10; i < 20; i++) {
        newCreatures.push({
          id: i,
          x: i * 10,
          y: i * 10,
          rotation: i * 0.1,
          size: 10,
        });
      }

      renderer.onSimulationTick(newCreatures);

      expect(renderer.getCreatureCount()).toBe(10);
    });
  });

  describe("buffer management", () => {
    it("should mark buffer dirty after initialization", () => {
      const creatures: CreatureData[] = [
        { id: 1, x: 100, y: 50, rotation: 0, size: 10 },
      ];

      renderer.initialize(creatures);

      expect(renderer.isBufferDirty()).toBe(true);
    });

    it("should mark buffer dirty after simulation tick", () => {
      renderer.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      // Clean the buffer
      renderer.render(16.67, 0, 0, 10, 800, 600);
      expect(renderer.isBufferDirty()).toBe(false);

      // Update should make it dirty again
      renderer.onSimulationTick([
        { id: 1, x: 100, y: 50, rotation: 1.0, size: 10 },
      ]);

      expect(renderer.isBufferDirty()).toBe(true);
    });

    it("should clean buffer after render", () => {
      renderer.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      expect(renderer.isBufferDirty()).toBe(true);

      renderer.render(16.67, 0, 0, 10, 800, 600);

      expect(renderer.isBufferDirty()).toBe(false);
    });
  });

  describe("performance", () => {
    it("should handle 100K creatures efficiently", () => {
      const creatures: CreatureData[] = [];
      for (let i = 0; i < 100000; i++) {
        creatures.push({
          id: i,
          x: Math.random() * 1000,
          y: Math.random() * 1000,
          rotation: Math.random() * Math.PI * 2,
          size: 10,
        });
      }

      const startInit = performance.now();
      renderer.initialize(creatures);
      const initTime = performance.now() - startInit;

      expect(initTime).toBeLessThan(100); // Should init in <100ms
      expect(renderer.getCreatureCount()).toBe(100000);

      // Render should be fast (just updates uniforms)
      const startRender = performance.now();
      renderer.render(16.67, 0, 0, 10, 800, 600);
      const renderTime = performance.now() - startRender;

      expect(renderTime).toBeLessThan(5); // Should render in <5ms
    });

    it("should handle rapid updates efficiently", () => {
      const creatures: CreatureData[] = [];
      for (let i = 0; i < 10000; i++) {
        creatures.push({
          id: i,
          x: i * 10,
          y: i * 10,
          rotation: 0,
          size: 10,
        });
      }

      renderer.initialize(creatures);

      // Simulate 60 updates (1 second @ 60 FPS)
      const startTime = performance.now();
      for (let frame = 0; frame < 60; frame++) {
        renderer.render(16.67, 0, 0, 10, 800, 600);
      }
      const totalTime = performance.now() - startTime;

      expect(totalTime).toBeLessThan(100); // 60 frames in <100ms
    });
  });

  describe("edge cases", () => {
    it("should handle empty creature list", () => {
      renderer.initialize([]);

      expect(renderer.getCreatureCount()).toBe(0);
      expect(() => renderer.render(16.67, 0, 0, 10, 800, 600)).not.toThrow();
    });

    it("should handle render before initialization", () => {
      expect(() => renderer.render(16.67, 0, 0, 10, 800, 600)).not.toThrow();
    });

    it("should handle multiple initializations", () => {
      renderer.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);
      expect(renderer.getCreatureCount()).toBe(1);

      renderer.initialize([
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 2, x: 100, y: 100, rotation: 0, size: 10 },
      ]);
      expect(renderer.getCreatureCount()).toBe(2);
    });
  });

  describe("mesh integration", () => {
    it("should provide access to PixiJS mesh", () => {
      const mesh = renderer.getMesh();

      expect(mesh).toBeDefined();
      expect(mesh.geometry).toBeDefined();
      expect(mesh.shader).toBeDefined();
    });

    it("should update mesh visibility based on creature count", () => {
      const mesh = renderer.getMesh();

      // No creatures = not visible
      renderer.initialize([]);
      expect(mesh.visible).toBe(false);

      // Has creatures = visible
      renderer.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);
      expect(mesh.visible).toBe(true);
    });
  });

  describe("texture aspect ratio", () => {
    it("should set texture aspect ratio uniform based on texture dimensions", () => {
      const rectangularTexture = {
        width: 239,
        height: 163,
        source: {
          width: 239,
          height: 163,
          uid: 2,
          _resourceType: "textureSource",
          _resourceId: 2,
          destroyed: false,
        },
      } as unknown as Texture;

      const aspectRenderer = new InterpolatedCreatureRenderer(rectangularTexture, 1000);
      const uniforms = aspectRenderer.getUniforms();

      const expectedAspectRatio = 163 / 239;
      expect(uniforms.uTextureAspectRatio).toBeCloseTo(expectedAspectRatio, 3);
    });

    it("should create square quad for square texture (aspect ratio 1.0)", () => {
      const squareTexture = {
        width: 64,
        height: 64,
        source: {
          width: 64,
          height: 64,
          uid: 3,
          _resourceType: "textureSource",
          _resourceId: 3,
          destroyed: false,
        },
      } as unknown as Texture;

      const squareRenderer = new InterpolatedCreatureRenderer(squareTexture, 1000);
      const uniforms = squareRenderer.getUniforms();

      expect(uniforms.uTextureAspectRatio).toBe(1.0);
    });

    it("should handle tall textures (height > width)", () => {
      const tallTexture = {
        width: 100,
        height: 200,
        source: {
          width: 100,
          height: 200,
          uid: 4,
          _resourceType: "textureSource",
          _resourceId: 4,
          destroyed: false,
        },
      } as unknown as Texture;

      const tallRenderer = new InterpolatedCreatureRenderer(tallTexture, 1000);
      const uniforms = tallRenderer.getUniforms();

      expect(uniforms.uTextureAspectRatio).toBe(2.0);
    });
  });
});

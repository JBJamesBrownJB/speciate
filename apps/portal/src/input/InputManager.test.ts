import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { InputManager } from "./InputManager";

describe("InputManager", () => {
  let inputManager: InputManager;

  beforeEach(() => {
    inputManager = new InputManager();
  });

  describe("Keyboard State", () => {
    it("should track pressed keys", () => {
      inputManager.handleKeyDown("w");
      expect(inputManager.isKeyPressed("w")).toBe(true);
    });

    it("should clear keys on key up", () => {
      inputManager.handleKeyDown("w");
      inputManager.handleKeyUp("w");
      expect(inputManager.isKeyPressed("w")).toBe(false);
    });

    it("should handle multiple simultaneous keys", () => {
      inputManager.handleKeyDown("w");
      inputManager.handleKeyDown("d");
      expect(inputManager.isKeyPressed("w")).toBe(true);
      expect(inputManager.isKeyPressed("d")).toBe(true);
    });

    it("should be case-insensitive", () => {
      inputManager.handleKeyDown("W");
      expect(inputManager.isKeyPressed("w")).toBe(true);
      expect(inputManager.isKeyPressed("W")).toBe(true);
    });

    it("should handle key repeat events gracefully", () => {
      inputManager.handleKeyDown("w");
      inputManager.handleKeyDown("w");
      inputManager.handleKeyDown("w");
      expect(inputManager.isKeyPressed("w")).toBe(true);
      inputManager.handleKeyUp("w");
      expect(inputManager.isKeyPressed("w")).toBe(false);
    });

    it("should handle releasing a key that was never pressed", () => {
      inputManager.handleKeyUp("x");
      expect(inputManager.isKeyPressed("x")).toBe(false);
    });
  });

  describe("Pan Velocity", () => {
    it("should return zero velocity with no keys pressed", () => {
      const vel = inputManager.getPanVelocity();
      expect(vel).toEqual({ x: 0, y: 0 });
    });

    it("should return upward velocity for W key", () => {
      inputManager.handleKeyDown("w");
      const vel = inputManager.getPanVelocity();
      expect(vel.x).toBe(0);
      expect(vel.y).toBeLessThan(0);
    });

    it("should return downward velocity for S key", () => {
      inputManager.handleKeyDown("s");
      const vel = inputManager.getPanVelocity();
      expect(vel.x).toBe(0);
      expect(vel.y).toBeGreaterThan(0);
    });

    it("should return leftward velocity for A key", () => {
      inputManager.handleKeyDown("a");
      const vel = inputManager.getPanVelocity();
      expect(vel.x).toBeLessThan(0);
      expect(vel.y).toBe(0);
    });

    it("should return rightward velocity for D key", () => {
      inputManager.handleKeyDown("d");
      const vel = inputManager.getPanVelocity();
      expect(vel.x).toBeGreaterThan(0);
      expect(vel.y).toBe(0);
    });

    it("should return upward velocity for ArrowUp key", () => {
      inputManager.handleKeyDown("ArrowUp");
      const vel = inputManager.getPanVelocity();
      expect(vel.x).toBe(0);
      expect(vel.y).toBeLessThan(0);
    });

    it("should return downward velocity for ArrowDown key", () => {
      inputManager.handleKeyDown("ArrowDown");
      const vel = inputManager.getPanVelocity();
      expect(vel.x).toBe(0);
      expect(vel.y).toBeGreaterThan(0);
    });

    it("should return leftward velocity for ArrowLeft key", () => {
      inputManager.handleKeyDown("ArrowLeft");
      const vel = inputManager.getPanVelocity();
      expect(vel.x).toBeLessThan(0);
      expect(vel.y).toBe(0);
    });

    it("should return rightward velocity for ArrowRight key", () => {
      inputManager.handleKeyDown("ArrowRight");
      const vel = inputManager.getPanVelocity();
      expect(vel.x).toBeGreaterThan(0);
      expect(vel.y).toBe(0);
    });

    it("should return diagonal velocity for W+D keys", () => {
      inputManager.handleKeyDown("w");
      inputManager.handleKeyDown("d");
      const vel = inputManager.getPanVelocity();
      expect(vel.x).toBeGreaterThan(0);
      expect(vel.y).toBeLessThan(0);
    });

    it("should normalize diagonal movement to magnitude 1", () => {
      inputManager.handleKeyDown("w");
      inputManager.handleKeyDown("d");
      const vel = inputManager.getPanVelocity();
      const magnitude = Math.sqrt(vel.x ** 2 + vel.y ** 2);
      expect(magnitude).toBeCloseTo(1.0, 5);
    });

    it("should cancel out opposing keys (W+S)", () => {
      inputManager.handleKeyDown("w");
      inputManager.handleKeyDown("s");
      const vel = inputManager.getPanVelocity();
      expect(vel.y).toBe(0);
    });

    it("should cancel out opposing keys (A+D)", () => {
      inputManager.handleKeyDown("a");
      inputManager.handleKeyDown("d");
      const vel = inputManager.getPanVelocity();
      expect(vel.x).toBe(0);
    });
  });

  describe("Text Input Focus", () => {
    let inputElement: HTMLInputElement;
    let textareaElement: HTMLTextAreaElement;

    beforeEach(() => {
      inputElement = document.createElement("input");
      textareaElement = document.createElement("textarea");
      document.body.appendChild(inputElement);
      document.body.appendChild(textareaElement);
    });

    afterEach(() => {
      document.body.removeChild(inputElement);
      document.body.removeChild(textareaElement);
    });

    it("should return zero velocity when input is focused", () => {
      inputElement.focus();
      inputManager.handleKeyDown("w");
      const vel = inputManager.getPanVelocity();
      expect(vel).toEqual({ x: 0, y: 0 });
    });

    it("should return zero velocity when textarea is focused", () => {
      textareaElement.focus();
      inputManager.handleKeyDown("w");
      const vel = inputManager.getPanVelocity();
      expect(vel).toEqual({ x: 0, y: 0 });
    });

    it("should still track keys even when input focused", () => {
      inputElement.focus();
      inputManager.handleKeyDown("w");
      expect(inputManager.isKeyPressed("w")).toBe(true);
    });
  });

  describe("Mouse Drag", () => {
    it("should start drag on right mouse button down", () => {
      inputManager.handlePointerDown(100, 200, 2);
      expect(inputManager.isDragging()).toBe(true);
    });

    it("should not start drag on left mouse button down", () => {
      inputManager.handlePointerDown(100, 200, 0);
      expect(inputManager.isDragging()).toBe(false);
    });

    it("should not start drag on middle mouse button down", () => {
      inputManager.handlePointerDown(100, 200, 1);
      expect(inputManager.isDragging()).toBe(false);
    });

    it("should end drag on pointer up", () => {
      inputManager.handlePointerDown(100, 200, 2);
      inputManager.handlePointerUp();
      expect(inputManager.isDragging()).toBe(false);
    });

    it("should calculate drag delta correctly", () => {
      inputManager.handlePointerDown(100, 200, 2);
      inputManager.handlePointerMove(150, 250);
      const delta = inputManager.getDragDelta();
      expect(delta).toEqual({ x: 50, y: 50 });
    });

    it("should return zero delta when not dragging", () => {
      const delta = inputManager.getDragDelta();
      expect(delta).toEqual({ x: 0, y: 0 });
    });

    it("should track negative deltas", () => {
      inputManager.handlePointerDown(200, 300, 2);
      inputManager.handlePointerMove(100, 150);
      const delta = inputManager.getDragDelta();
      expect(delta).toEqual({ x: -100, y: -150 });
    });

    it("should ignore pointer move when not dragging", () => {
      inputManager.handlePointerMove(100, 200);
      const delta = inputManager.getDragDelta();
      expect(delta).toEqual({ x: 0, y: 0 });
    });
  });

  describe("Consume Drag Delta", () => {
    it("should return delta and reset start position", () => {
      inputManager.handlePointerDown(100, 200, 2);
      inputManager.handlePointerMove(150, 250);

      const delta1 = inputManager.consumeDragDelta();
      expect(delta1).toEqual({ x: 50, y: 50 });

      const delta2 = inputManager.consumeDragDelta();
      expect(delta2).toEqual({ x: 0, y: 0 });
    });

    it("should allow continuous drag tracking", () => {
      inputManager.handlePointerDown(100, 200, 2);
      inputManager.handlePointerMove(150, 250);
      inputManager.consumeDragDelta();

      inputManager.handlePointerMove(200, 300);
      const delta = inputManager.consumeDragDelta();
      expect(delta).toEqual({ x: 50, y: 50 });
    });
  });

  describe("Clear All Keys", () => {
    it("should clear all pressed keys", () => {
      inputManager.handleKeyDown("w");
      inputManager.handleKeyDown("a");
      inputManager.handleKeyDown("s");
      inputManager.handleKeyDown("d");

      inputManager.clearAllKeys();

      expect(inputManager.isKeyPressed("w")).toBe(false);
      expect(inputManager.isKeyPressed("a")).toBe(false);
      expect(inputManager.isKeyPressed("s")).toBe(false);
      expect(inputManager.isKeyPressed("d")).toBe(false);
    });
  });
});

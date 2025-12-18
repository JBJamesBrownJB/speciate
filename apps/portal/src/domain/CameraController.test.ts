import { describe, it, expect, beforeEach } from "vitest";
import { CameraController } from "./CameraController";
import { Camera } from "./Camera";
import { InputManager } from "../input/InputManager";

describe("CameraController", () => {
  let camera: Camera;
  let inputManager: InputManager;
  let controller: CameraController;

  beforeEach(() => {
    camera = new Camera(500, 500, 10);
    inputManager = new InputManager();
    controller = new CameraController(camera, inputManager);
  });

  describe("Keyboard Panning", () => {
    it("should not move camera when no keys pressed", () => {
      const initialX = camera.x;
      const initialY = camera.y;

      controller.update(1 / 60);

      expect(camera.x).toBe(initialX);
      expect(camera.y).toBe(initialY);
    });

    it("should move camera up when W key pressed", () => {
      const initialY = camera.y;
      inputManager.handleKeyDown("w");

      controller.update(1 / 60);

      expect(camera.y).toBeLessThan(initialY);
    });

    it("should move camera down when S key pressed", () => {
      const initialY = camera.y;
      inputManager.handleKeyDown("s");

      controller.update(1 / 60);

      expect(camera.y).toBeGreaterThan(initialY);
    });

    it("should move camera left when A key pressed", () => {
      const initialX = camera.x;
      inputManager.handleKeyDown("a");

      controller.update(1 / 60);

      expect(camera.x).toBeLessThan(initialX);
    });

    it("should move camera right when D key pressed", () => {
      const initialX = camera.x;
      inputManager.handleKeyDown("d");

      controller.update(1 / 60);

      expect(camera.x).toBeGreaterThan(initialX);
    });

    it("should move camera with arrow keys", () => {
      const initialX = camera.x;
      const initialY = camera.y;

      inputManager.handleKeyDown("ArrowRight");
      inputManager.handleKeyDown("ArrowDown");
      controller.update(1 / 60);

      expect(camera.x).toBeGreaterThan(initialX);
      expect(camera.y).toBeGreaterThan(initialY);
    });

    it("should move faster when zoomed out", () => {
      camera.setZoom(5);
      const initialX = camera.x;
      inputManager.handleKeyDown("d");
      controller.update(1);
      const distanceZoom5 = camera.x - initialX;

      camera = new Camera(500, 500, 20);
      controller = new CameraController(camera, inputManager);
      const initialX2 = camera.x;
      controller.update(1);
      const distanceZoom20 = camera.x - initialX2;

      expect(distanceZoom5).toBeGreaterThan(distanceZoom20);
    });

    it("should move proportionally with delta time", () => {
      inputManager.handleKeyDown("d");

      const cam1 = new Camera(500, 500, 10);
      const ctrl1 = new CameraController(cam1, inputManager);
      ctrl1.update(0.5);
      const distance1 = cam1.x - 500;

      const cam2 = new Camera(500, 500, 10);
      const ctrl2 = new CameraController(cam2, inputManager);
      ctrl2.update(1.0);
      const distance2 = cam2.x - 500;

      expect(distance2).toBeCloseTo(distance1 * 2, 5);
    });

    it("should handle diagonal movement", () => {
      const initialX = camera.x;
      const initialY = camera.y;

      inputManager.handleKeyDown("w");
      inputManager.handleKeyDown("d");
      controller.update(1 / 60);

      expect(camera.x).toBeGreaterThan(initialX);
      expect(camera.y).toBeLessThan(initialY);
    });
  });

  describe("Mouse Drag Panning", () => {
    it("should not move camera when not dragging", () => {
      const initialX = camera.x;
      const initialY = camera.y;

      inputManager.handlePointerMove(100, 100);
      controller.update(1 / 60);

      expect(camera.x).toBe(initialX);
      expect(camera.y).toBe(initialY);
    });

    it("should move camera when dragging right (inverted)", () => {
      const initialX = camera.x;

      inputManager.handlePointerDown(100, 100, 2);
      inputManager.handlePointerMove(150, 100);
      controller.update(1 / 60);

      expect(camera.x).toBeLessThan(initialX);
    });

    it("should move camera when dragging left (inverted)", () => {
      const initialX = camera.x;

      inputManager.handlePointerDown(150, 100, 2);
      inputManager.handlePointerMove(100, 100);
      controller.update(1 / 60);

      expect(camera.x).toBeGreaterThan(initialX);
    });

    it("should move camera when dragging down (inverted)", () => {
      const initialY = camera.y;

      inputManager.handlePointerDown(100, 100, 2);
      inputManager.handlePointerMove(100, 150);
      controller.update(1 / 60);

      expect(camera.y).toBeLessThan(initialY);
    });

    it("should move camera when dragging up (inverted)", () => {
      const initialY = camera.y;

      inputManager.handlePointerDown(100, 150, 2);
      inputManager.handlePointerMove(100, 100);
      controller.update(1 / 60);

      expect(camera.y).toBeGreaterThan(initialY);
    });

    it("should convert screen pixels to world units based on zoom", () => {
      camera.setZoom(10);
      const initialX = camera.x;

      inputManager.handlePointerDown(0, 0, 2);
      inputManager.handlePointerMove(100, 0);
      controller.update(1 / 60);

      const movedDistance = initialX - camera.x;
      expect(movedDistance).toBeCloseTo(10, 1);
    });

    it("should allow continuous drag tracking", () => {
      camera.setZoom(10);
      const initialX = camera.x;

      inputManager.handlePointerDown(0, 0, 2);
      inputManager.handlePointerMove(50, 0);
      controller.update(1 / 60);
      inputManager.handlePointerMove(100, 0);
      controller.update(1 / 60);

      const totalMoved = initialX - camera.x;
      expect(totalMoved).toBeCloseTo(10, 1);
    });

    it("should stop moving when pointer up", () => {
      inputManager.handlePointerDown(100, 100, 2);
      inputManager.handlePointerMove(150, 150);
      inputManager.handlePointerUp();

      const initialX = camera.x;
      const initialY = camera.y;
      controller.update(1 / 60);

      expect(camera.x).toBe(initialX);
      expect(camera.y).toBe(initialY);
    });
  });

  describe("Combined Input", () => {
    it("should combine keyboard and drag panning", () => {
      const initialX = camera.x;

      inputManager.handleKeyDown("d");
      inputManager.handlePointerDown(0, 0, 2);
      inputManager.handlePointerMove(-50, 0);
      controller.update(1 / 60);

      expect(camera.x).toBeGreaterThan(initialX);
    });
  });
});

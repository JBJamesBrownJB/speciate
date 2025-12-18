import { Camera } from "./Camera";
import { InputManager } from "../input/InputManager";
import { CAMERA_CONFIG } from "../core/constants";

export class CameraController {
  constructor(
    private camera: Camera,
    private inputManager: InputManager
  ) {}

  update(deltaTime: number): void {
    this.updateKeyboardPanning(deltaTime);
    this.updateDragPanning();
  }

  private updateKeyboardPanning(deltaTime: number): void {
    const vel = this.inputManager.getPanVelocity();
    if (vel.x === 0 && vel.y === 0) return;

    const speed = CAMERA_CONFIG.PAN_SPEED_BASE / this.camera.zoom;
    this.camera.deltaMove(vel.x * speed * deltaTime, vel.y * speed * deltaTime);
  }

  private updateDragPanning(): void {
    if (!this.inputManager.isDragging()) return;

    const delta = this.inputManager.consumeDragDelta();
    this.camera.deltaMove(-delta.x / this.camera.zoom, -delta.y / this.camera.zoom);
  }
}

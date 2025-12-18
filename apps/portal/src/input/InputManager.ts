export class InputManager {
  private keysPressed = new Set<string>();
  private dragging = false;
  private dragStartX = 0;
  private dragStartY = 0;
  private dragCurrentX = 0;
  private dragCurrentY = 0;

  handleKeyDown(key: string): void {
    this.keysPressed.add(key.toLowerCase());
  }

  handleKeyUp(key: string): void {
    this.keysPressed.delete(key.toLowerCase());
  }

  isKeyPressed(key: string): boolean {
    return this.keysPressed.has(key.toLowerCase());
  }

  getPanVelocity(): { x: number; y: number } {
    if (this.isTextInputFocused()) {
      return { x: 0, y: 0 };
    }

    let x = 0;
    let y = 0;

    if (this.isKeyPressed("w") || this.isKeyPressed("arrowup")) y -= 1;
    if (this.isKeyPressed("s") || this.isKeyPressed("arrowdown")) y += 1;
    if (this.isKeyPressed("a") || this.isKeyPressed("arrowleft")) x -= 1;
    if (this.isKeyPressed("d") || this.isKeyPressed("arrowright")) x += 1;

    if (x !== 0 && y !== 0) {
      const magnitude = Math.sqrt(x * x + y * y);
      x /= magnitude;
      y /= magnitude;
    }

    return { x, y };
  }

  handlePointerDown(x: number, y: number, button: number): void {
    if (button === 2) {
      this.dragging = true;
      this.dragStartX = x;
      this.dragStartY = y;
      this.dragCurrentX = x;
      this.dragCurrentY = y;
    }
  }

  handlePointerMove(x: number, y: number): void {
    if (this.dragging) {
      this.dragCurrentX = x;
      this.dragCurrentY = y;
    }
  }

  handlePointerUp(): void {
    this.dragging = false;
  }

  isDragging(): boolean {
    return this.dragging;
  }

  getDragDelta(): { x: number; y: number } {
    if (!this.dragging) {
      return { x: 0, y: 0 };
    }
    return {
      x: this.dragCurrentX - this.dragStartX,
      y: this.dragCurrentY - this.dragStartY,
    };
  }

  consumeDragDelta(): { x: number; y: number } {
    const delta = this.getDragDelta();
    this.dragStartX = this.dragCurrentX;
    this.dragStartY = this.dragCurrentY;
    return delta;
  }

  clearAllKeys(): void {
    this.keysPressed.clear();
  }

  private isTextInputFocused(): boolean {
    const active = document.activeElement;
    return active?.tagName === "INPUT" || active?.tagName === "TEXTAREA";
  }
}

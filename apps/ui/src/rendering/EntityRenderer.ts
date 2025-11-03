import { Graphics, Container } from 'pixi.js';
import type { Vec3 } from '@/types/entities';

export interface EntityVisual {
  graphics: Graphics;
  container: Container;
}

export class EntityRenderer {
  private entityVisuals: Map<string, EntityVisual> = new Map();
  private stage: Container;
  private worldScale: number = 50;
  private screenCenterX: number;
  private screenCenterY: number;

  constructor(stage: Container, screenWidth: number, screenHeight: number) {
    this.stage = stage;
    this.screenCenterX = screenWidth / 2;
    this.screenCenterY = screenHeight / 2;
  }

  public ensureEntity(entityId: string): EntityVisual {
    let visual = this.entityVisuals.get(entityId);
    if (!visual) {
      const container = new Container();
      const graphics = new Graphics();
      graphics.circle(0, 0, 10);
      graphics.fill({ color: 0x00ffff, alpha: 1.0 });
      graphics.circle(0, 0, 10);
      graphics.stroke({ color: 0xffffff, width: 2, alpha: 0.5 });
      container.addChild(graphics);
      this.stage.addChild(container);
      visual = { graphics, container };
      this.entityVisuals.set(entityId, visual);
    }
    return visual;
  }

  public updateEntityPosition(entityId: string, position: Vec3): void {
    const visual = this.ensureEntity(entityId);
    const screenX = this.screenCenterX + (position.x * this.worldScale);
    const screenY = this.screenCenterY + (position.y * this.worldScale);
    visual.container.position.set(screenX, screenY);
  }

  public removeEntity(entityId: string): void {
    const visual = this.entityVisuals.get(entityId);
    if (visual) {
      this.stage.removeChild(visual.container);
      visual.container.destroy({ children: true });
      this.entityVisuals.delete(entityId);
    }
  }

  public clear(): void {
    this.entityVisuals.forEach((_, entityId) => {
      this.removeEntity(entityId);
    });
  }

  public updateScreenDimensions(width: number, height: number): void {
    this.screenCenterX = width / 2;
    this.screenCenterY = height / 2;
  }
}

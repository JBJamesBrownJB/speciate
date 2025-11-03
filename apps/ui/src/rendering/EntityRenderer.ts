import { Graphics, Container } from 'pixi.js';
import type { Creature, BehaviorMode } from '@/types/messages';
import { ViewportCuller } from './ViewportCuller';
import { RENDERING_CONFIG } from '../core/constants';

export interface EntityVisual {
  graphics: Graphics;
  container: Container;
  energyBar?: Graphics;
}

interface CreatureState {
  targetX: number;
  targetY: number;
  targetRotation: number;
  startX: number;
  startY: number;
  startRotation: number;
  lerpStartTime: number;
  expectedTickDuration: number;
  behavior?: BehaviorMode;
  energy?: number;
  width: number;
  height: number;
}

const BEHAVIOR_COLORS: Record<BehaviorMode | 'default', number> = {
  'Wandering': 0x4ECDC4,
  'Fleeing': 0xEA3546,
  'Feeding': 0x44AF69,
  'Resting': 0x7B68EE,
  'default': 0x808080,
};

const SIZE_COLORS = [
  0x4ECDC4,
  0x44AF69,
  0xF8B500,
  0xF86624,
  0xEA3546,
];

export class EntityRenderer {
  private entityVisuals: Map<number, EntityVisual> = new Map();
  private creatureStates: Map<number, CreatureState> = new Map();
  private stage: Container;
  private culler: ViewportCuller;

  // Sprite pooling for performance
  private visualPool: EntityVisual[] = [];

  private worldWidth: number = 180.0;
  private worldHeight: number = 130.0;
  private viewportX: number = 0;
  private viewportY: number = 0;
  private scale: number = 1;

  private lastUpdateTime: number = 0;
  private estimatedTickDuration: number = 1000;

  constructor(stage: Container, screenWidth: number, screenHeight: number) {
    this.stage = stage;
    this.culler = new ViewportCuller(RENDERING_CONFIG.VIEWPORT_PADDING);

    this.calculateViewportTransform(screenWidth, screenHeight);

    console.log('EntityRenderer initialized:');
    console.log('  Screen:', screenWidth, 'x', screenHeight);
    console.log('  World:', this.worldWidth, 'x', this.worldHeight);
    console.log('  Scale:', this.scale.toFixed(2), 'px per world unit');
    console.log('  Viewport offset:', this.viewportX.toFixed(0), ',', this.viewportY.toFixed(0));
  }

  updateCreatures(creatures: Creature[]): void {
    console.log('Updating', creatures.length, 'creatures');

    const now = performance.now();
    const activeIds = new Set(creatures.map(c => c.id));

    this.updateTickDurationEstimate(now);

    for (const creature of creatures) {
      this.updateCreature(creature, now);
    }

    this.removeInactiveCreatures(activeIds);
  }

  render(): void {
    const now = performance.now();
    const bounds = this.calculateViewportBoundsForCulling();

    for (const [id, visual] of this.entityVisuals) {
      const state = this.creatureStates.get(id);
      if (!state) continue;

      const worldPos = this.interpolatePosition(state, now);

      if (this.culler.isVisible(worldPos, this.getCreatureRadius(id), bounds)) {
        this.renderCreature(visual, worldPos, state, now);
      } else {
        visual.container.visible = false;
      }
    }
  }

  clear(): void {
    this.entityVisuals.forEach((_, entityId) => {
      this.removeEntity(entityId);
    });
    this.creatureStates.clear();
  }

  destroy(): void {
    // Remove all active entities
    this.clear();

    // Destroy all pooled visuals
    for (const visual of this.visualPool) {
      this.stage.removeChild(visual.container);
      visual.container.destroy({ children: true });
    }
    this.visualPool = [];
  }

  removeEntity(entityId: string | number): void {
    const id = typeof entityId === 'string' ? parseInt(entityId) : entityId;
    const visual = this.entityVisuals.get(id);
    if (visual) {
      // Hide and return to pool instead of destroying
      visual.container.visible = false;
      this.visualPool.push(visual);

      this.entityVisuals.delete(id);
      this.creatureStates.delete(id);
    }
  }

  updateScreenDimensions(width: number, height: number): void {
    this.calculateViewportTransform(width, height);
  }

  getWorldBoundsScreen(): { x: number; y: number; width: number; height: number } {
    return {
      x: this.viewportX,
      y: this.viewportY,
      width: this.worldWidth * this.scale,
      height: this.worldHeight * this.scale
    };
  }

  updateEntityPosition(_entityId: string, _position: any): void {
    // Legacy method - not used
  }

  private calculateViewportTransform(width: number, height: number): void {
    const paddingRatio = 0.9;
    const scaleX = (width * paddingRatio) / this.worldWidth;
    const scaleY = (height * paddingRatio) / this.worldHeight;
    this.scale = Math.min(scaleX, scaleY);

    const worldPixelWidth = this.worldWidth * this.scale;
    const worldPixelHeight = this.worldHeight * this.scale;
    this.viewportX = (width - worldPixelWidth) / 2;
    this.viewportY = (height - worldPixelHeight) / 2;
  }

  private worldToScreen(worldX: number, worldY: number): { x: number; y: number } {
    return {
      x: this.viewportX + (worldX * this.scale),
      y: this.viewportY + (worldY * this.scale)
    };
  }

  private calculateViewportBoundsForCulling() {
    return this.culler.calculateBounds(
      this.worldWidth * this.scale,
      this.worldHeight * this.scale,
      -this.viewportX / this.scale,
      -this.viewportY / this.scale
    );
  }

  private updateTickDurationEstimate(now: number): void {
    if (this.lastUpdateTime > 0) {
      const timeSinceLastUpdate = now - this.lastUpdateTime;
      this.estimatedTickDuration = this.estimatedTickDuration * 0.8 + timeSinceLastUpdate * 0.2;
    }
    this.lastUpdateTime = now;
  }

  private updateCreature(creature: Creature, now: number): void {
    let visual = this.entityVisuals.get(creature.id);
    if (!visual) {
      console.log('Creating new creature', creature.id);
      visual = this.createCreatureGraphics(creature);
      this.entityVisuals.set(creature.id, visual);
    }

    if (creature.behavior || creature.energy !== undefined) {
      this.updateCreatureAppearance(visual, creature);
    }

    this.updateCreatureInterpolationState(creature, now);
  }

  private createCreatureGraphics(creature: Creature): EntityVisual {
    // Try to reuse from pool first
    let visual = this.visualPool.pop();

    if (!visual) {
      // Pool empty, create new visual
      const container = new Container();
      const graphics = new Graphics();
      visual = { graphics, container };
      this.stage.addChild(container);
    }

    // Reset and redraw the visual
    const pixelWidth = creature.width * 10;
    const pixelHeight = creature.height * 10;
    const color = this.getCreatureColor(creature);

    this.drawCreatureBody(visual.graphics, pixelWidth, pixelHeight, color);

    // Handle energy bar
    if (creature.energy !== undefined) {
      if (!visual.energyBar) {
        visual.energyBar = new Graphics();
        visual.container.addChild(visual.energyBar);
      }
      this.updateEnergyBar(visual.energyBar, creature.energy, pixelWidth);
    } else if (visual.energyBar) {
      // Hide energy bar if not needed
      visual.energyBar.visible = false;
    }

    // Ensure graphics is in container
    if (!visual.container.children.includes(visual.graphics)) {
      visual.container.addChild(visual.graphics);
    }

    // Make container visible and ensure it's on stage
    visual.container.visible = true;
    if (!this.stage.children.includes(visual.container)) {
      this.stage.addChild(visual.container);
    }

    return visual;
  }

  private drawCreatureBody(graphics: Graphics, pixelWidth: number, pixelHeight: number, color: number): void {
    graphics.clear();
    graphics.ellipse(0, 0, pixelWidth, pixelHeight);
    graphics.fill({ color, alpha: 0.9 });
    graphics.ellipse(0, 0, pixelWidth, pixelHeight);
    graphics.stroke({ color: 0xFFFFFF, width: 2, alpha: 0.3 });
    graphics.circle(pixelWidth * 0.7, 0, pixelWidth * 0.2);
    graphics.fill({ color: 0xFFFFFF, alpha: 0.9 });
  }

  private updateCreatureAppearance(visual: EntityVisual, creature: Creature): void {
    const color = this.getCreatureColor(creature);
    const pixelWidth = creature.width * 10;
    const pixelHeight = creature.height * 10;

    this.drawCreatureBody(visual.graphics, pixelWidth, pixelHeight, color);

    if (creature.energy !== undefined && visual.energyBar) {
      this.updateEnergyBar(visual.energyBar, creature.energy, pixelWidth);
    }
  }

  private updateEnergyBar(energyBar: Graphics, energy: number, pixelWidth: number): void {
    energyBar.clear();

    const barWidth = pixelWidth * 1.5;
    const barHeight = 4;
    const barY = -pixelWidth - 10;

    energyBar.rect(-barWidth/2, barY, barWidth, barHeight);
    energyBar.fill({ color: 0x333333, alpha: 0.5 });

    const energyPercent = energy / 100;
    const energyColor = energyPercent > 0.5 ? 0x44AF69 :
                        energyPercent > 0.3 ? 0xF8B500 : 0xEA3546;

    energyBar.rect(-barWidth/2, barY, barWidth * energyPercent, barHeight);
    energyBar.fill({ color: energyColor, alpha: 0.8 });
  }

  private updateCreatureInterpolationState(creature: Creature, now: number): void {
    let state = this.creatureStates.get(creature.id);

    if (!state) {
      state = {
        startX: creature.x,
        startY: creature.y,
        startRotation: creature.rotation,
        targetX: creature.x,
        targetY: creature.y,
        targetRotation: creature.rotation,
        lerpStartTime: now,
        expectedTickDuration: this.estimatedTickDuration,
        behavior: creature.behavior,
        energy: creature.energy,
        width: creature.width,
        height: creature.height,
      };
      this.creatureStates.set(creature.id, state);
    } else {
      const timeSinceStart = now - state.lerpStartTime;
      const currentT = state.expectedTickDuration > 0
        ? Math.min(timeSinceStart / state.expectedTickDuration, 1.0)
        : 1.0;

      state.startX = this.lerp(state.startX, state.targetX, currentT);
      state.startY = this.lerp(state.startY, state.targetY, currentT);
      state.startRotation = this.lerpAngle(state.startRotation, state.targetRotation, currentT);

      state.targetX = creature.x;
      state.targetY = creature.y;
      state.targetRotation = creature.rotation;
      state.lerpStartTime = now;
      state.expectedTickDuration = this.estimatedTickDuration;
      state.behavior = creature.behavior;
      state.energy = creature.energy;
      state.width = creature.width;
      state.height = creature.height;
    }
  }

  private interpolatePosition(state: CreatureState, now: number): { x: number; y: number } {
    const timeSinceStart = now - state.lerpStartTime;
    const t = state.expectedTickDuration > 0
      ? Math.min(timeSinceStart / state.expectedTickDuration, 1.0)
      : 1.0;

    return {
      x: this.lerp(state.startX, state.targetX, t),
      y: this.lerp(state.startY, state.targetY, t)
    };
  }

  private renderCreature(visual: EntityVisual, worldPos: { x: number; y: number }, state: CreatureState, now: number): void {
    const timeSinceStart = now - state.lerpStartTime;
    const t = state.expectedTickDuration > 0
      ? Math.min(timeSinceStart / state.expectedTickDuration, 1.0)
      : 1.0;

    const rotation = this.lerpAngle(state.startRotation, state.targetRotation, t);
    const screenPos = this.worldToScreen(worldPos.x, worldPos.y);

    visual.container.position.set(screenPos.x, screenPos.y);
    visual.container.rotation = rotation;
    visual.container.visible = true;
  }

  private removeInactiveCreatures(activeIds: Set<number>): void {
    for (const [id] of this.entityVisuals) {
      if (!activeIds.has(id)) {
        this.removeEntity(id);
      }
    }
  }

  private getCreatureRadius(id: number): number {
    const state = this.creatureStates.get(id);
    if (!state) return 10;

    // Calculate pixel dimensions (same as rendering logic)
    const pixelWidth = state.width * 10;
    const pixelHeight = state.height * 10;

    // Return radius of bounding circle (use larger dimension)
    return Math.max(pixelWidth, pixelHeight) / 2;
  }

  private getCreatureColor(creature: Creature): number {
    if (creature.behavior) {
      return BEHAVIOR_COLORS[creature.behavior] || BEHAVIOR_COLORS.default;
    }

    const avgSize = (creature.width + creature.height) / 2;
    const index = Math.floor(((avgSize - 0.5) / 2.5) * SIZE_COLORS.length);
    return SIZE_COLORS[Math.max(0, Math.min(SIZE_COLORS.length - 1, index))];
  }

  private lerp(start: number, end: number, t: number): number {
    return start + (end - start) * t;
  }

  private lerpAngle(start: number, end: number, t: number): number {
    const diff = end - start;
    const shortestDiff = ((diff + Math.PI) % (Math.PI * 2)) - Math.PI;
    return start + shortestDiff * t;
  }
}

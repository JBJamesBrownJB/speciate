import type { CreatureData } from '../types/GameState';

type SelectionEventType = 'creature-selected' | 'creature-deselected';
type SelectionEventHandler = (creature?: CreatureData) => void;

export class SelectionManager {
  private selectedCreature: CreatureData | null = null;
  private eventListeners: Map<SelectionEventType, Set<SelectionEventHandler>> = new Map();

  constructor() {
    this.eventListeners.set('creature-selected', new Set());
    this.eventListeners.set('creature-deselected', new Set());
  }

  getSelected(): CreatureData | null {
    return this.selectedCreature;
  }

  hasSelection(): boolean {
    return this.selectedCreature !== null;
  }

  selectCreature(creature: CreatureData): void {
    this.selectedCreature = { ...creature };
    this.emit('creature-selected', creature);
  }

  deselect(): void {
    if (this.selectedCreature === null) {
      return;
    }
    this.selectedCreature = null;
    this.emit('creature-deselected');
  }

  findNearestCreature(
    creatures: CreatureData[],
    worldX: number,
    worldY: number,
    clickRadius: number
  ): CreatureData | null {
    if (creatures.length === 0) {
      return null;
    }

    let nearest: CreatureData | null = null;
    let minDistSq = Infinity;

    for (const creature of creatures) {
      const dx = worldX - creature.x;
      const dy = worldY - creature.y;
      const distSq = dx * dx + dy * dy;

      const hitRadius = clickRadius + creature.size / 2;
      const hitRadiusSq = hitRadius * hitRadius;

      if (distSq <= hitRadiusSq && distSq < minDistSq) {
        minDistSq = distSq;
        nearest = creature;
      }
    }

    return nearest;
  }

  updateSelectedFromBuffer(creatures: CreatureData[]): void {
    if (!this.selectedCreature) {
      return;
    }

    const updated = creatures.find(c => c.id === this.selectedCreature!.id);
    if (updated) {
      this.selectedCreature = { ...updated };
    } else {
      this.deselect();
    }
  }

  on(event: SelectionEventType, handler: SelectionEventHandler): void {
    this.eventListeners.get(event)?.add(handler);
  }

  off(event: SelectionEventType, handler: SelectionEventHandler): void {
    this.eventListeners.get(event)?.delete(handler);
  }

  private emit(event: SelectionEventType, data?: CreatureData): void {
    const listeners = this.eventListeners.get(event);
    if (listeners) {
      for (const handler of listeners) {
        handler(data);
      }
    }
  }
}

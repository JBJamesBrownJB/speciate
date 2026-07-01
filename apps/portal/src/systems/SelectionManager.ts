import type { CreatureData, CreatureFrameView } from '../types/GameState';

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

  /** Scan the SoA frame directly (no object array); materializes ONE
   *  CreatureData for the winner. Event-driven (clicks), so O(n) is fine. */
  findNearestCreature(
    frame: CreatureFrameView | null,
    worldX: number,
    worldY: number,
    clickRadius: number
  ): CreatureData | null {
    if (!frame || frame.count === 0) {
      return null;
    }

    let nearestIdx = -1;
    let minDistSq = Infinity;

    for (let i = 0; i < frame.count; i++) {
      const dx = worldX - frame.xs[i];
      const dy = worldY - frame.ys[i];
      const distSq = dx * dx + dy * dy;

      const hitRadius = clickRadius + frame.sizes[i] / 2;

      if (distSq <= hitRadius * hitRadius && distSq < minDistSq) {
        minDistSq = distSq;
        nearestIdx = i;
      }
    }

    if (nearestIdx < 0) return null;
    return {
      id: frame.ids[nearestIdx],
      x: frame.xs[nearestIdx],
      y: frame.ys[nearestIdx],
      rotation: frame.rots[nearestIdx],
      size: frame.sizes[nearestIdx],
    };
  }

  /** Track the selected creature into the newest frame: O(1) via the frame's
   *  idToIndex, mutating the owned copy in place (no per-tick allocation). */
  updateSelectedFromFrame(frame: CreatureFrameView | null): void {
    const selected = this.selectedCreature;
    if (!selected) {
      return;
    }

    const idx = frame?.idToIndex.get(selected.id);
    if (frame && idx !== undefined) {
      selected.x = frame.xs[idx];
      selected.y = frame.ys[idx];
      selected.rotation = frame.rots[idx];
      selected.size = frame.sizes[idx];
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

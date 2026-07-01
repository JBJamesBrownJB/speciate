import { describe, it, expect, beforeEach, vi } from 'vitest';
import { SelectionManager } from './SelectionManager';
import { CreatureFrameSlot } from '@/rendering/CreatureFramePool';
import type { CreatureData, CreatureFrameView } from '../types/GameState';

/** Build a SoA frame view from plain creature objects (test convenience). */
function frameOf(creatures: CreatureData[]): CreatureFrameView {
  return new CreatureFrameSlot(Math.max(creatures.length, 1)).fill(creatures);
}

describe('SelectionManager', () => {
  let selectionManager: SelectionManager;

  beforeEach(() => {
    selectionManager = new SelectionManager();
  });

  describe('initial state', () => {
    it('should have no selection initially', () => {
      expect(selectionManager.getSelected()).toBeNull();
    });

    it('should not be in selected state initially', () => {
      expect(selectionManager.hasSelection()).toBe(false);
    });
  });

  describe('selectCreature', () => {
    it('should select a creature by ID', () => {
      const creature: CreatureData = {
        id: 12345,
        x: 100,
        y: 200,
        rotation: 0,
        size: 2.5,
      };

      selectionManager.selectCreature(creature);

      expect(selectionManager.getSelected()).toEqual(creature);
      expect(selectionManager.hasSelection()).toBe(true);
    });

    it('should emit creature-selected event when selecting', () => {
      const handler = vi.fn();
      selectionManager.on('creature-selected', handler);

      const creature: CreatureData = {
        id: 12345,
        x: 100,
        y: 200,
        rotation: 0,
        size: 2.5,
      };

      selectionManager.selectCreature(creature);

      expect(handler).toHaveBeenCalledTimes(1);
      expect(handler).toHaveBeenCalledWith(creature);
    });

    it('should replace previous selection when selecting new creature', () => {
      const creature1: CreatureData = { id: 1, x: 0, y: 0, rotation: 0, size: 1 };
      const creature2: CreatureData = { id: 2, x: 10, y: 10, rotation: 0, size: 1 };

      selectionManager.selectCreature(creature1);
      selectionManager.selectCreature(creature2);

      expect(selectionManager.getSelected()?.id).toBe(2);
    });
  });

  describe('deselect', () => {
    it('should clear selection', () => {
      const creature: CreatureData = { id: 1, x: 0, y: 0, rotation: 0, size: 1 };
      selectionManager.selectCreature(creature);

      selectionManager.deselect();

      expect(selectionManager.getSelected()).toBeNull();
      expect(selectionManager.hasSelection()).toBe(false);
    });

    it('should emit creature-deselected event when deselecting', () => {
      const handler = vi.fn();
      selectionManager.on('creature-deselected', handler);

      const creature: CreatureData = { id: 1, x: 0, y: 0, rotation: 0, size: 1 };
      selectionManager.selectCreature(creature);
      selectionManager.deselect();

      expect(handler).toHaveBeenCalledTimes(1);
    });

    it('should not emit event when deselecting with no selection', () => {
      const handler = vi.fn();
      selectionManager.on('creature-deselected', handler);

      selectionManager.deselect();

      expect(handler).not.toHaveBeenCalled();
    });
  });

  describe('findNearestCreature', () => {
    const creatures: CreatureData[] = [
      { id: 1, x: 100, y: 100, rotation: 0, size: 2 },
      { id: 2, x: 200, y: 200, rotation: 0, size: 2 },
      { id: 3, x: 150, y: 150, rotation: 0, size: 2 },
    ];

    it('should find nearest creature within click radius', () => {
      const result = selectionManager.findNearestCreature(frameOf(creatures), 102, 98, 20);

      expect(result?.id).toBe(1);
    });

    it('should return null if no creature within radius', () => {
      const result = selectionManager.findNearestCreature(frameOf(creatures), 500, 500, 20);

      expect(result).toBeNull();
    });

    it('should find the closest when multiple creatures in radius', () => {
      const result = selectionManager.findNearestCreature(frameOf(creatures), 145, 145, 50);

      expect(result?.id).toBe(3);
    });

    it('should account for creature size in hit detection', () => {
      const largeCreature: CreatureData[] = [
        { id: 1, x: 100, y: 100, rotation: 0, size: 20 },
      ];

      const result = selectionManager.findNearestCreature(frameOf(largeCreature), 115, 100, 5);

      expect(result?.id).toBe(1);
    });

    it('should return null for an empty frame', () => {
      const result = selectionManager.findNearestCreature(frameOf([]), 100, 100, 20);

      expect(result).toBeNull();
    });

    it('should return null when no frame has arrived yet', () => {
      const result = selectionManager.findNearestCreature(null, 100, 100, 20);

      expect(result).toBeNull();
    });

    it('materializes the full creature record for the winner (values, not just id)', () => {
      const result = selectionManager.findNearestCreature(
        frameOf([{ id: 5, x: 10, y: 20, rotation: 1.5, size: 4 }]), 10, 20, 5
      );

      expect(result).toEqual({ id: 5, x: 10, y: 20, rotation: 1.5, size: 4 });
    });
  });

  describe('event handling', () => {
    it('should allow removing event listeners', () => {
      const handler = vi.fn();
      selectionManager.on('creature-selected', handler);
      selectionManager.off('creature-selected', handler);

      const creature: CreatureData = { id: 1, x: 0, y: 0, rotation: 0, size: 1 };
      selectionManager.selectCreature(creature);

      expect(handler).not.toHaveBeenCalled();
    });

    it('should support multiple listeners', () => {
      const handler1 = vi.fn();
      const handler2 = vi.fn();

      selectionManager.on('creature-selected', handler1);
      selectionManager.on('creature-selected', handler2);

      const creature: CreatureData = { id: 1, x: 0, y: 0, rotation: 0, size: 1 };
      selectionManager.selectCreature(creature);

      expect(handler1).toHaveBeenCalledTimes(1);
      expect(handler2).toHaveBeenCalledTimes(1);
    });
  });

  describe('updateSelectedFromFrame', () => {
    it('should update position of selected creature from the frame', () => {
      const creature: CreatureData = { id: 1, x: 100, y: 100, rotation: 0, size: 2 };
      selectionManager.selectCreature(creature);

      selectionManager.updateSelectedFromFrame(frameOf([
        { id: 1, x: 150, y: 200, rotation: 1.5, size: 2 },
      ]));

      const selected = selectionManager.getSelected();
      expect(selected?.x).toBe(150);
      expect(selected?.y).toBe(200);
      expect(selected?.rotation).toBe(1.5);
    });

    it('should deselect if creature no longer in the frame', () => {
      const creature: CreatureData = { id: 1, x: 100, y: 100, rotation: 0, size: 2 };
      selectionManager.selectCreature(creature);

      selectionManager.updateSelectedFromFrame(frameOf([
        { id: 2, x: 150, y: 200, rotation: 1.5, size: 2 },
      ]));

      expect(selectionManager.getSelected()).toBeNull();
    });

    it('is a no-op when nothing is selected', () => {
      expect(() => selectionManager.updateSelectedFromFrame(frameOf([]))).not.toThrow();
      expect(selectionManager.getSelected()).toBeNull();
    });
  });
});

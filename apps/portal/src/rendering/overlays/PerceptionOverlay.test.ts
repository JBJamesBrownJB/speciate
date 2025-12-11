import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { PerceptionOverlay } from './PerceptionOverlay';
import { Container } from 'pixi.js';
import type { PerceptionDebugData } from '@/types/GameState';

describe('PerceptionOverlay', () => {
  let overlay: PerceptionOverlay;
  let container: Container;

  beforeEach(() => {
    container = new Container();
    overlay = new PerceptionOverlay(container);
  });

  afterEach(() => {
    overlay.destroy();
  });

  describe('initial state', () => {
    it('should not be visible initially', () => {
      expect(overlay.isVisible()).toBe(false);
    });

    it('should have correct config', () => {
      expect(overlay.config.name).toBe('perception');
      expect(overlay.config.devToolsOnly).toBe(true);
      expect(overlay.config.keyboardShortcut).toBe('p');
    });
  });

  describe('update with debug data', () => {
    it('should become visible when debug data is provided and shown', () => {
      overlay.show();
      const debugData: PerceptionDebugData = {
        entityId: 1,
        x: 100,
        y: 200,
        perceptionRange: 50,
        fovAngle: Math.PI,
        rotation: 0,
        ax: 0,
        ay: 0,
        neighbors: [],
        cellSize: 50,
        creatureCell: { x: 2, y: 4 },
        queriedCells: [],
        checkedCells: [],
      };
      overlay.update(debugData);
      expect(overlay.isVisible()).toBe(true);
    });

    it('should add graphics to container', () => {
      overlay.show();
      const debugData: PerceptionDebugData = {
        entityId: 1,
        x: 100,
        y: 200,
        perceptionRange: 50,
        fovAngle: Math.PI,
        rotation: 0,
        ax: 0,
        ay: 0,
        neighbors: [],
        cellSize: 50,
        creatureCell: { x: 2, y: 4 },
        queriedCells: [],
        checkedCells: [],
      };
      overlay.update(debugData);
      expect(container.children.length).toBe(1);
    });

    it('should handle debug data with neighbors', () => {
      overlay.show();
      const debugData: PerceptionDebugData = {
        entityId: 1,
        x: 100,
        y: 200,
        perceptionRange: 50,
        fovAngle: Math.PI,
        rotation: 0,
        ax: 0,
        ay: 0,
        neighbors: [
          { id: 2, x: 120, y: 210 },
          { id: 3, x: 80, y: 190 },
        ],
        cellSize: 50,
        creatureCell: { x: 2, y: 4 },
        queriedCells: [{ x: 1, y: 3 }, { x: 2, y: 4 }, { x: 3, y: 5 }],
        checkedCells: [{ x: 1, y: 3 }, { x: 2, y: 4 }],
      };
      expect(() => overlay.update(debugData)).not.toThrow();
      expect(overlay.isVisible()).toBe(true);
    });
  });

  describe('clear', () => {
    it('should hide overlay', () => {
      overlay.show();
      const debugData: PerceptionDebugData = {
        entityId: 1,
        x: 100,
        y: 200,
        perceptionRange: 50,
        fovAngle: Math.PI,
        rotation: 0,
        ax: 0,
        ay: 0,
        neighbors: [],
        cellSize: 50,
        creatureCell: { x: 2, y: 4 },
        queriedCells: [],
        checkedCells: [],
      };
      overlay.update(debugData);
      overlay.clear();
      expect(overlay.isVisible()).toBe(false);
    });
  });

  describe('update with undefined', () => {
    it('should hide overlay when update receives undefined', () => {
      overlay.show();
      const debugData: PerceptionDebugData = {
        entityId: 1,
        x: 100,
        y: 200,
        perceptionRange: 50,
        fovAngle: Math.PI,
        rotation: 0,
        ax: 0,
        ay: 0,
        neighbors: [],
        cellSize: 50,
        creatureCell: { x: 2, y: 4 },
        queriedCells: [],
        checkedCells: [],
      };
      overlay.update(debugData);
      expect(overlay.isVisible()).toBe(true);
      overlay.update(undefined);
      expect(overlay.isVisible()).toBe(false);
    });
  });

  describe('toggle', () => {
    it('should toggle visibility', () => {
      expect(overlay.isVisible()).toBe(false);
      overlay.toggle();
      expect(overlay.config.name).toBe('perception');
    });
  });

  describe('destroy', () => {
    it('should not throw when visible', () => {
      overlay.show();
      const debugData: PerceptionDebugData = {
        entityId: 1,
        x: 100,
        y: 200,
        perceptionRange: 50,
        fovAngle: Math.PI,
        rotation: 0,
        ax: 0,
        ay: 0,
        neighbors: [],
        cellSize: 50,
        creatureCell: { x: 2, y: 4 },
        queriedCells: [],
        checkedCells: [],
      };
      overlay.update(debugData);
      expect(() => overlay.destroy()).not.toThrow();
    });

    it('should not throw when hidden', () => {
      expect(() => overlay.destroy()).not.toThrow();
    });
  });
});

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { CreatureInfoPanel } from './CreatureInfoPanel';
import type { CreatureData, PerceptionDebugData } from '../types/GameState';

describe('CreatureInfoPanel', () => {
  let panel: CreatureInfoPanel;
  let parentElement: HTMLElement;

  beforeEach(() => {
    parentElement = document.createElement('div');
    document.body.appendChild(parentElement);
    panel = new CreatureInfoPanel(parentElement);
  });

  afterEach(() => {
    panel.destroy();
    parentElement.remove();
  });

  describe('per-frame rebuild guard', () => {
    const creature: CreatureData = { id: 7, x: 10.0, y: 20.0, rotation: 0, size: 2.0 };

    /** Wrap the live panel element's innerHTML with a write counter. */
    function countHtmlWrites(): () => number {
      const el = parentElement.querySelector('.creature-info-panel') as HTMLElement;
      let writes = 0;
      const proto = Object.getPrototypeOf(el);
      const desc = Object.getOwnPropertyDescriptor(proto, 'innerHTML') ??
        Object.getOwnPropertyDescriptor(Element.prototype, 'innerHTML')!;
      Object.defineProperty(el, 'innerHTML', {
        get: desc.get,
        set(v: string) {
          writes++;
          desc.set!.call(this, v);
        },
        configurable: true,
      });
      return () => writes;
    }

    it('update() skips the innerHTML rebuild when displayed values are unchanged', () => {
      panel.show(creature);
      const writes = countHtmlWrites();

      panel.update({ ...creature });
      panel.update({ ...creature });
      panel.update({ ...creature });

      expect(writes()).toBe(0);
    });

    it('update() rebuilds when a displayed value changes', () => {
      panel.show(creature);
      const writes = countHtmlWrites();

      panel.update({ ...creature, x: 15.0 });

      expect(writes()).toBe(1);
    });

    it('sub-precision moves that display identically do not rebuild', () => {
      panel.show(creature);
      const writes = countHtmlWrites();

      // 10.0 and 10.04 both display as "10.0"
      panel.update({ ...creature, x: 10.04 });

      expect(writes()).toBe(0);
    });

    it('new debug accel data triggers a rebuild on the next update (dev panel)', () => {
      const devParent = document.createElement('div');
      document.body.appendChild(devParent);
      const devPanel = new CreatureInfoPanel(devParent, { showDebugInfo: true });
      devPanel.show(creature);

      const el = devParent.querySelector('.creature-info-panel') as HTMLElement;
      let writes = 0;
      const desc = Object.getOwnPropertyDescriptor(Element.prototype, 'innerHTML')!;
      Object.defineProperty(el, 'innerHTML', {
        get: desc.get,
        set(v: string) { writes++; desc.set!.call(this, v); },
        configurable: true,
      });

      devPanel.updateDebugData({
        entityId: 7, x: 10, y: 20, perceptionRange: 5, queryRadius: 5,
        fovAngle: 1, rotation: 0, ax: 1.5, ay: -0.5,
        neighbors: [], cellSize: 20, creatureCell: { x: 0, y: 0 },
        queriedCells: [], checkedCells: [],
      });
      devPanel.update({ ...creature });

      expect(writes).toBe(1);
      expect(el.textContent).toContain('1.50');

      devPanel.destroy();
      devParent.remove();
    });
  });

  describe('construction', () => {
    it('should create panel element in parent', () => {
      const panelEl = parentElement.querySelector('.creature-info-panel');
      expect(panelEl).not.toBeNull();
    });

    it('should be hidden initially', () => {
      expect(panel.isVisible()).toBe(false);
    });
  });

  describe('show', () => {
    it('should display creature data', () => {
      const creature: CreatureData = {
        id: 12345,
        x: 100.5,
        y: 200.3,
        rotation: 1.5,
        size: 2.5,
      };

      panel.show(creature);

      expect(panel.isVisible()).toBe(true);
      const panelEl = parentElement.querySelector('.creature-info-panel');
      expect(panelEl?.textContent).toContain('12345');
      expect(panelEl?.textContent).toContain('100.5');
      expect(panelEl?.textContent).toContain('200.3');
    });

    it('should show extended data when provided', () => {
      const creature: CreatureData = {
        id: 12345,
        x: 100,
        y: 200,
        rotation: 0,
        size: 2.5,
      };

      panel.show(creature, { energy: 75.5, behavior: 'Wandering' });

      const panelEl = parentElement.querySelector('.creature-info-panel');
      expect(panelEl?.textContent).toContain('75.5');
      expect(panelEl?.textContent).toContain('Wandering');
    });
  });

  describe('hide', () => {
    it('should hide the panel', () => {
      const creature: CreatureData = {
        id: 1,
        x: 0,
        y: 0,
        rotation: 0,
        size: 1,
      };

      panel.show(creature);
      panel.hide();

      expect(panel.isVisible()).toBe(false);
    });
  });

  describe('update', () => {
    it('should update position values', () => {
      const creature: CreatureData = {
        id: 12345,
        x: 100,
        y: 200,
        rotation: 0,
        size: 2.5,
      };

      panel.show(creature);

      const updatedCreature: CreatureData = {
        ...creature,
        x: 150,
        y: 250,
      };

      panel.update(updatedCreature);

      const panelEl = parentElement.querySelector('.creature-info-panel');
      expect(panelEl?.textContent).toContain('150');
      expect(panelEl?.textContent).toContain('250');
    });
  });

  describe('destroy', () => {
    it('should remove panel from DOM', () => {
      panel.destroy();

      const panelEl = parentElement.querySelector('.creature-info-panel');
      expect(panelEl).toBeNull();
    });
  });

  describe('updateDebugData', () => {
    let devPanel: CreatureInfoPanel;
    let devParent: HTMLElement;

    beforeEach(() => {
      devParent = document.createElement('div');
      document.body.appendChild(devParent);
      devPanel = new CreatureInfoPanel(devParent, { showDebugInfo: true });
    });

    afterEach(() => {
      devPanel.destroy();
      devParent.remove();
    });

    it('does NOT display debug acceleration on the default (player) panel', () => {
      const creature: CreatureData = { id: 1, x: 0, y: 0, rotation: 0, size: 1 };

      panel.show(creature);
      panel.updateDebugData({
        entityId: 1, x: 0, y: 0, perceptionRange: 50, queryRadius: 55,
        fovAngle: Math.PI, rotation: 0, ax: 1.5, ay: 2.5,
        neighbors: [], cellSize: 50, creatureCell: { x: 0, y: 0 },
        queriedCells: [], checkedCells: [],
      });
      panel.update({ ...creature, x: 1 });

      const panelEl = parentElement.querySelector('.creature-info-panel');
      expect(panelEl?.textContent).not.toContain('Accel');
    });

    it('should display acceleration from debug data', () => {
      const creature: CreatureData = {
        id: 12345,
        x: 100,
        y: 200,
        rotation: 0,
        size: 2.5,
      };

      const debugData: PerceptionDebugData = {
        entityId: 12345,
        x: 100,
        y: 200,
        perceptionRange: 50,
        queryRadius: 55,
        fovAngle: Math.PI,
        rotation: 0,
        ax: 1.5,
        ay: 2.5,
        neighbors: [],
        cellSize: 50,
        creatureCell: { x: 2, y: 4 },
        queriedCells: [],
        checkedCells: [],
      };

      devPanel.show(creature);
      devPanel.updateDebugData(debugData);
      devPanel.update({ ...creature, x: creature.x + 1 });

      const panelEl = devParent.querySelector('.creature-info-panel');
      expect(panelEl?.textContent).toContain('Accel');
      expect(panelEl?.textContent).toContain('1.50');
      expect(panelEl?.textContent).toContain('2.50');
    });

    it('should clear acceleration when debug data is null', () => {
      const creature: CreatureData = {
        id: 12345,
        x: 100,
        y: 200,
        rotation: 0,
        size: 2.5,
      };

      const debugData: PerceptionDebugData = {
        entityId: 12345,
        x: 100,
        y: 200,
        perceptionRange: 50,
        queryRadius: 55,
        fovAngle: Math.PI,
        rotation: 0,
        ax: 1.5,
        ay: 2.5,
        neighbors: [],
        cellSize: 50,
        creatureCell: { x: 2, y: 4 },
        queriedCells: [],
        checkedCells: [],
      };

      devPanel.show(creature);
      devPanel.updateDebugData(debugData);
      devPanel.update({ ...creature, x: creature.x + 1 });

      // Now clear the debug data
      devPanel.updateDebugData(null);
      devPanel.update({ ...creature, x: creature.x + 2 });

      const panelEl = devParent.querySelector('.creature-info-panel');
      expect(panelEl?.textContent).not.toContain('1.50');
    });
  });

  describe('keyboard legend (dev-only — overlay shortcuts are gated in the player build)', () => {
    const creature: CreatureData = { id: 12345, x: 100, y: 200, rotation: 0, size: 2.5 };

    it('does not display the debug-overlay legend on the default (player) panel', () => {
      panel.show(creature);

      const panelEl = parentElement.querySelector('.creature-info-panel');
      expect(panelEl?.textContent).not.toContain('[G]');
      expect(panelEl?.textContent).not.toContain('Overlays');
    });

    it('displays keyboard shortcuts when debug info is enabled', () => {
      const devParent = document.createElement('div');
      document.body.appendChild(devParent);
      const devPanel = new CreatureInfoPanel(devParent, { showDebugInfo: true });

      devPanel.show(creature);

      const panelEl = devParent.querySelector('.creature-info-panel');
      expect(panelEl?.textContent).toContain('[G]');
      expect(panelEl?.textContent).toContain('Grid');
      expect(panelEl?.textContent).toContain('[F]');
      expect(panelEl?.textContent).toContain('Force');
      expect(panelEl?.textContent).toContain('[P]');
      expect(panelEl?.textContent).toContain('Perception');

      devPanel.destroy();
      devParent.remove();
    });
  });
});

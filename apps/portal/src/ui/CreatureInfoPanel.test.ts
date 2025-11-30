import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { CreatureInfoPanel } from './CreatureInfoPanel';
import type { CreatureData } from '../types/GameState';

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
});

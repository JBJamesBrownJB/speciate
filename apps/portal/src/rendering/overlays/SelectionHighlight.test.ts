import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { SelectionHighlight } from './SelectionHighlight';
import { Container, Graphics } from 'pixi.js';

describe('SelectionHighlight', () => {
  let highlight: SelectionHighlight;
  let container: Container;

  beforeEach(() => {
    container = new Container();
    highlight = new SelectionHighlight(container);
  });

  afterEach(() => {
    highlight.destroy();
  });

  describe('initial state', () => {
    it('should not be visible initially', () => {
      expect(highlight.isVisible()).toBe(false);
    });

    it('should have correct config', () => {
      expect(highlight.config.name).toBe('selection');
      expect(highlight.config.devToolsOnly).toBe(false);
    });
  });

  describe('showAt', () => {
    it('should set visible state to true', () => {
      highlight.showAt(100, 200, 15);
      expect(highlight.isVisible()).toBe(true);
    });

    it('should add graphics to container', () => {
      highlight.showAt(100, 200, 15);
      expect(container.children.length).toBe(1);
    });
  });

  describe('hide', () => {
    it('should set visible state to false', () => {
      highlight.showAt(100, 200, 15);
      highlight.hide();
      expect(highlight.isVisible()).toBe(false);
    });
  });

  describe('toggle', () => {
    it('should toggle visibility', () => {
      highlight.showAt(100, 200, 15);
      expect(highlight.isVisible()).toBe(true);
      highlight.toggle();
      expect(highlight.isVisible()).toBe(false);
      highlight.toggle();
      expect(highlight.isVisible()).toBe(true);
    });
  });

  describe('updatePosition', () => {
    it('should not throw when visible', () => {
      highlight.showAt(100, 200, 15);
      expect(() => highlight.updatePosition(150, 250)).not.toThrow();
    });

    it('should not throw when hidden', () => {
      expect(() => highlight.updatePosition(150, 250)).not.toThrow();
    });
  });

  describe('update (animation)', () => {
    it('should not throw when visible', () => {
      highlight.showAt(100, 200, 15);
      expect(() => highlight.update(16)).not.toThrow();
    });

    it('should not throw when hidden', () => {
      expect(() => highlight.update(16)).not.toThrow();
    });
  });

  describe('destroy', () => {
    it('should not throw', () => {
      highlight.showAt(100, 200, 15);
      expect(() => highlight.destroy()).not.toThrow();
    });
  });

  describe('render deduplication (one draw per change, not per call)', () => {
    it('updatePosition defers drawing to update() — no double render per frame', () => {
      highlight.showAt(100, 200, 15);
      const draws = vi.spyOn(Graphics.prototype, 'circle');

      // The per-frame pair the render loop makes:
      highlight.updatePosition(150, 250);
      highlight.update(16);

      expect(draws).toHaveBeenCalledTimes(1);
      draws.mockRestore();
    });

    it('update() skips redraw entirely when position is unchanged', () => {
      highlight.showAt(100, 200, 15);
      highlight.updatePosition(150, 250);
      highlight.update(16);

      const draws = vi.spyOn(Graphics.prototype, 'circle');
      highlight.updatePosition(150, 250); // same position
      highlight.update(16);
      highlight.update(16);

      expect(draws).not.toHaveBeenCalled();
      draws.mockRestore();
    });

    it('still draws immediately on showAt (selection feels instant)', () => {
      const draws = vi.spyOn(Graphics.prototype, 'circle');
      highlight.showAt(100, 200, 15);
      expect(draws).toHaveBeenCalledTimes(1);
      draws.mockRestore();
    });
  });
});

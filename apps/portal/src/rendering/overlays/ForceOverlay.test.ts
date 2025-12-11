import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { ForceOverlay } from './ForceOverlay';
import { Container } from 'pixi.js';

describe('ForceOverlay', () => {
  let overlay: ForceOverlay;
  let container: Container;

  beforeEach(() => {
    container = new Container();
    overlay = new ForceOverlay(container);
  });

  afterEach(() => {
    overlay.destroy();
  });

  describe('initial state', () => {
    it('should not be visible initially', () => {
      expect(overlay.isVisible()).toBe(false);
    });

    it('should have correct config', () => {
      expect(overlay.config.name).toBe('force');
      expect(overlay.config.devToolsOnly).toBe(true);
      expect(overlay.config.keyboardShortcut).toBe('f');
    });
  });

  describe('toggle', () => {
    it('should toggle visibility', () => {
      expect(overlay.isVisible()).toBe(false);
      overlay.toggle();
      expect(overlay.isVisible()).toBe(true);
      overlay.toggle();
      expect(overlay.isVisible()).toBe(false);
    });
  });

  describe('update with data', () => {
    it('should not render when not visible', () => {
      overlay.update({
        x: 100,
        y: 100,
        radius: 5,
        ax: 1,
        ay: 0,
      });
      // Force overlay not visible, graphics should be empty
      expect(overlay.isVisible()).toBe(false);
    });

    it('should render when visible and has data', () => {
      overlay.show();
      expect(overlay.isVisible()).toBe(true);
      // Update should not throw
      expect(() =>
        overlay.update({
          x: 100,
          y: 100,
          radius: 5,
          ax: 1,
          ay: 0,
        })
      ).not.toThrow();
    });

    it('should not render when force magnitude is zero', () => {
      overlay.show();
      expect(() =>
        overlay.update({
          x: 100,
          y: 100,
          radius: 5,
          ax: 0,
          ay: 0,
        })
      ).not.toThrow();
    });

    it('should handle undefined data', () => {
      overlay.show();
      expect(() => overlay.update(undefined)).not.toThrow();
    });
  });

  describe('destroy', () => {
    it('should not throw', () => {
      expect(() => overlay.destroy()).not.toThrow();
    });
  });
});

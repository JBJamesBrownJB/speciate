import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { OverlayManager } from './OverlayManager';
import type { IOverlay, OverlayConfig } from './IOverlay';

class MockOverlay implements IOverlay {
  public visible = false;

  constructor(public readonly config: OverlayConfig) {}

  show(): void {
    this.visible = true;
  }

  hide(): void {
    this.visible = false;
  }

  toggle(): void {
    this.visible = !this.visible;
  }

  isVisible(): boolean {
    return this.visible;
  }

  destroy(): void {}
}

describe('OverlayManager', () => {
  let manager: OverlayManager;

  beforeEach(() => {
    manager = new OverlayManager();
  });

  afterEach(() => {
    manager.destroy();
  });

  describe('registration', () => {
    it('should register an overlay', () => {
      const overlay = new MockOverlay({ name: 'test', devToolsOnly: true });
      manager.register(overlay);
      expect(manager.getOverlay('test')).toBe(overlay);
    });

    it('should register keyboard shortcut', () => {
      const overlay = new MockOverlay({
        name: 'test',
        devToolsOnly: true,
        keyboardShortcut: 'g',
      });
      manager.register(overlay);
      // Access private map for testing
      expect(manager.getAllOverlays()).toContain(overlay);
    });

    it('should not register duplicate overlay names', () => {
      const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
      const overlay1 = new MockOverlay({ name: 'test', devToolsOnly: true });
      const overlay2 = new MockOverlay({ name: 'test', devToolsOnly: true });

      manager.register(overlay1);
      manager.register(overlay2);

      expect(manager.getOverlay('test')).toBe(overlay1);
      expect(consoleError).toHaveBeenCalledWith('Overlay "test" already registered');
      consoleError.mockRestore();
    });
  });

  describe('toggle', () => {
    it('should toggle overlay visibility', () => {
      const overlay = new MockOverlay({ name: 'test', devToolsOnly: true });
      manager.register(overlay);

      expect(overlay.visible).toBe(false);
      manager.toggleOverlay('test');
      expect(overlay.visible).toBe(true);
      manager.toggleOverlay('test');
      expect(overlay.visible).toBe(false);
    });
  });

  describe('keyboard shortcuts', () => {
    it('should toggle overlay when shortcut key is pressed', () => {
      const overlay = new MockOverlay({
        name: 'test',
        devToolsOnly: true,
        keyboardShortcut: 'g',
      });
      manager.register(overlay);
      manager.enableKeyboardShortcuts();

      expect(overlay.visible).toBe(false);

      // Simulate keydown event
      const event = new KeyboardEvent('keydown', { key: 'g' });
      window.dispatchEvent(event);

      expect(overlay.visible).toBe(true);
    });

    it('should handle uppercase key press', () => {
      const overlay = new MockOverlay({
        name: 'test',
        devToolsOnly: true,
        keyboardShortcut: 'g',
      });
      manager.register(overlay);
      manager.enableKeyboardShortcuts();

      expect(overlay.visible).toBe(false);

      const event = new KeyboardEvent('keydown', { key: 'G' });
      window.dispatchEvent(event);

      expect(overlay.visible).toBe(true);
    });

    it('should not toggle when different key is pressed', () => {
      const overlay = new MockOverlay({
        name: 'test',
        devToolsOnly: true,
        keyboardShortcut: 'g',
      });
      manager.register(overlay);
      manager.enableKeyboardShortcuts();

      const event = new KeyboardEvent('keydown', { key: 'f' });
      window.dispatchEvent(event);

      expect(overlay.visible).toBe(false);
    });

    it('should handle multiple overlays with different shortcuts', () => {
      const overlay1 = new MockOverlay({
        name: 'grid',
        devToolsOnly: true,
        keyboardShortcut: 'g',
      });
      const overlay2 = new MockOverlay({
        name: 'force',
        devToolsOnly: true,
        keyboardShortcut: 'f',
      });
      const overlay3 = new MockOverlay({
        name: 'perception',
        devToolsOnly: true,
        keyboardShortcut: 'p',
      });

      manager.register(overlay1);
      manager.register(overlay2);
      manager.register(overlay3);
      manager.enableKeyboardShortcuts();

      // Press 'g'
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'g' }));
      expect(overlay1.visible).toBe(true);
      expect(overlay2.visible).toBe(false);
      expect(overlay3.visible).toBe(false);

      // Press 'f'
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'f' }));
      expect(overlay1.visible).toBe(true);
      expect(overlay2.visible).toBe(true);
      expect(overlay3.visible).toBe(false);

      // Press 'p'
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'p' }));
      expect(overlay1.visible).toBe(true);
      expect(overlay2.visible).toBe(true);
      expect(overlay3.visible).toBe(true);
    });

    it('should not respond to shortcuts when disabled', () => {
      const overlay = new MockOverlay({
        name: 'test',
        devToolsOnly: true,
        keyboardShortcut: 'g',
      });
      manager.register(overlay);
      manager.enableKeyboardShortcuts();
      manager.disableKeyboardShortcuts();

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'g' }));
      expect(overlay.visible).toBe(false);
    });
  });
});

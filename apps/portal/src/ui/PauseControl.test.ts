import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { PauseControl } from './PauseControl';
import { mockDocumentGetElementById } from '@/test-utils';

function createMockButton(): HTMLButtonElement {
  const listeners: { [key: string]: EventListener } = {};
  return {
    textContent: '',
    addEventListener: vi.fn((event: string, handler: EventListener) => {
      listeners[event] = handler;
    }),
    removeEventListener: vi.fn((event: string) => {
      delete listeners[event];
    }),
    setAttribute: vi.fn(),
    classList: {
      add: vi.fn(),
      remove: vi.fn(),
      toggle: vi.fn(),
    },
    click: () => {
      if (listeners['click']) {
        listeners['click'](new MouseEvent('click'));
      }
    },
  } as unknown as HTMLButtonElement;
}

describe('PauseControl', () => {
  let mockButton: HTMLButtonElement;

  beforeEach(() => {
    mockButton = createMockButton();
    mockDocumentGetElementById({
      'pause-button': mockButton,
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('initial state', () => {
    it('should start in running (not paused) state', () => {
      const control = new PauseControl({ buttonId: 'pause-button' });
      expect(control.isPaused()).toBe(false);
    });

    it('should show pause icon initially (click to pause)', () => {
      new PauseControl({ buttonId: 'pause-button' });
      expect(mockButton.textContent).toBe('\u23F8');
    });

    it('should set correct aria-label initially', () => {
      new PauseControl({ buttonId: 'pause-button' });
      expect(mockButton.setAttribute).toHaveBeenCalledWith('aria-label', 'Pause simulation');
    });
  });

  describe('toggle', () => {
    it('should toggle from running to paused', () => {
      const control = new PauseControl({ buttonId: 'pause-button' });
      control.toggle();
      expect(control.isPaused()).toBe(true);
    });

    it('should toggle from paused to running', () => {
      const control = new PauseControl({ buttonId: 'pause-button' });
      control.toggle();
      control.toggle();
      expect(control.isPaused()).toBe(false);
    });

    it('should update button icon on toggle to paused', () => {
      const control = new PauseControl({ buttonId: 'pause-button' });
      control.toggle();
      expect(mockButton.textContent).toBe('\u25B6');
    });

    it('should update button icon on toggle back to running', () => {
      const control = new PauseControl({ buttonId: 'pause-button' });
      control.toggle();
      control.toggle();
      expect(mockButton.textContent).toBe('\u23F8');
    });

    it('should update aria-label on toggle', () => {
      const control = new PauseControl({ buttonId: 'pause-button' });
      control.toggle();
      expect(mockButton.setAttribute).toHaveBeenCalledWith('aria-label', 'Resume simulation');
    });
  });

  describe('setPaused', () => {
    it('should call onPauseChange callback when state changes', () => {
      const callback = vi.fn();
      const control = new PauseControl({
        buttonId: 'pause-button',
        onPauseChange: callback,
      });

      control.setPaused(true);

      expect(callback).toHaveBeenCalledWith(true);
    });

    it('should not call callback if state unchanged', () => {
      const callback = vi.fn();
      const control = new PauseControl({
        buttonId: 'pause-button',
        onPauseChange: callback,
      });

      control.setPaused(false);

      expect(callback).not.toHaveBeenCalled();
    });

    it('should update isPaused() return value', () => {
      const control = new PauseControl({ buttonId: 'pause-button' });

      control.setPaused(true);
      expect(control.isPaused()).toBe(true);

      control.setPaused(false);
      expect(control.isPaused()).toBe(false);
    });
  });

  describe('click handler', () => {
    it('should toggle on button click', () => {
      const control = new PauseControl({ buttonId: 'pause-button' });

      mockButton.click();

      expect(control.isPaused()).toBe(true);
    });

    it('should call callback on button click', () => {
      const callback = vi.fn();
      new PauseControl({
        buttonId: 'pause-button',
        onPauseChange: callback,
      });

      mockButton.click();

      expect(callback).toHaveBeenCalledWith(true);
    });
  });

  describe('keyboard shortcut', () => {
    it('should toggle on ESC key', () => {
      const control = new PauseControl({ buttonId: 'pause-button' });
      control.enableKeyboardShortcut();

      const event = new KeyboardEvent('keydown', { key: 'Escape' });
      window.dispatchEvent(event);

      expect(control.isPaused()).toBe(true);
    });

    it('should toggle back on second ESC key', () => {
      const control = new PauseControl({ buttonId: 'pause-button' });
      control.enableKeyboardShortcut();

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));

      expect(control.isPaused()).toBe(false);
    });

    it('should return cleanup function', () => {
      const control = new PauseControl({ buttonId: 'pause-button' });
      const cleanup = control.enableKeyboardShortcut();

      expect(typeof cleanup).toBe('function');
    });

    it('should stop listening after cleanup', () => {
      const control = new PauseControl({ buttonId: 'pause-button' });
      const cleanup = control.enableKeyboardShortcut();

      cleanup();

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));

      expect(control.isPaused()).toBe(false);
    });
  });

  describe('null button handling', () => {
    it('should handle missing button element', () => {
      mockDocumentGetElementById({});

      expect(() => {
        new PauseControl({ buttonId: 'missing-button' });
      }).not.toThrow();
    });

    it('should still track state with missing button', () => {
      mockDocumentGetElementById({});
      const control = new PauseControl({ buttonId: 'missing-button' });

      control.setPaused(true);

      expect(control.isPaused()).toBe(true);
    });
  });
});

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { ToolsPanel } from './ToolsPanel';
import { mockDocumentGetElementById } from '@/test-utils';

function createMockButton(): HTMLElement {
  const listeners: Record<string, EventListener[]> = {};
  let classList = new Set<string>();
  return {
    addEventListener: vi.fn((event: string, handler: EventListener) => {
      if (!listeners[event]) listeners[event] = [];
      listeners[event].push(handler);
    }),
    classList: {
      toggle: vi.fn((cls: string, force?: boolean) => {
        if (force === true) classList.add(cls);
        else if (force === false) classList.delete(cls);
        else if (classList.has(cls)) classList.delete(cls);
        else classList.add(cls);
      }),
      contains: (cls: string) => classList.has(cls),
    },
    click: () => {
      listeners['click']?.forEach(h => h(new MouseEvent('click')));
    },
  } as unknown as HTMLElement;
}

describe('ToolsPanel', () => {
  let btn: HTMLElement;

  beforeEach(() => {
    btn = createMockButton();
    mockDocumentGetElementById({ 'tool-plant': btn });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('initial state', () => {
    it('has no active tool', () => {
      const panel = new ToolsPanel();
      expect(panel.activeTool).toBeNull();
    });
  });

  describe('toggleTool', () => {
    it('activates plant on first toggle', () => {
      const panel = new ToolsPanel();
      panel.toggleTool('plant');
      expect(panel.activeTool).toBe('plant');
    });

    it('deactivates plant on second toggle', () => {
      const panel = new ToolsPanel();
      panel.toggleTool('plant');
      panel.toggleTool('plant');
      expect(panel.activeTool).toBeNull();
    });

    it('calls onToolChange with plant when activating', () => {
      const cb = vi.fn();
      const panel = new ToolsPanel();
      panel.onToolChange = cb;
      panel.toggleTool('plant');
      expect(cb).toHaveBeenCalledWith('plant');
    });

    it('calls onToolChange with null when deactivating', () => {
      const cb = vi.fn();
      const panel = new ToolsPanel();
      panel.onToolChange = cb;
      panel.toggleTool('plant');
      panel.toggleTool('plant');
      expect(cb).toHaveBeenLastCalledWith(null);
    });

    it('adds active class to button when tool activates', () => {
      const panel = new ToolsPanel();
      panel.toggleTool('plant');
      expect(btn.classList.toggle).toHaveBeenCalledWith('active', true);
    });

    it('removes active class from button when tool deactivates', () => {
      const panel = new ToolsPanel();
      panel.toggleTool('plant');
      panel.toggleTool('plant');
      expect(btn.classList.toggle).toHaveBeenLastCalledWith('active', false);
    });
  });

  describe('button click', () => {
    it('toggles tool on click', () => {
      const panel = new ToolsPanel();
      (btn as any).click();
      expect(panel.activeTool).toBe('plant');
    });

    it('deactivates on second click', () => {
      const panel = new ToolsPanel();
      (btn as any).click();
      (btn as any).click();
      expect(panel.activeTool).toBeNull();
    });
  });

  describe('missing button element', () => {
    it('does not throw when button element is absent', () => {
      mockDocumentGetElementById({});
      expect(() => new ToolsPanel()).not.toThrow();
    });

    it('still tracks state with missing button', () => {
      mockDocumentGetElementById({});
      const panel = new ToolsPanel();
      panel.toggleTool('plant');
      expect(panel.activeTool).toBe('plant');
    });
  });
});

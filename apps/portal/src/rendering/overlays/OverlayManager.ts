import type { IOverlay } from './IOverlay';

export class OverlayManager {
  private overlays: Map<string, IOverlay> = new Map();
  private keyboardShortcuts: Map<string, string> = new Map();
  private keydownHandler: ((event: KeyboardEvent) => void) | null = null;

  register(overlay: IOverlay): void {
    const { name, keyboardShortcut } = overlay.config;

    if (this.overlays.has(name)) {
      console.error(`Overlay "${name}" already registered`);
      return;
    }

    this.overlays.set(name, overlay);

    if (keyboardShortcut) {
      this.keyboardShortcuts.set(keyboardShortcut.toLowerCase(), name);
    }
  }

  unregister(name: string): void {
    const overlay = this.overlays.get(name);
    if (overlay) {
      const shortcut = overlay.config.keyboardShortcut;
      if (shortcut) {
        this.keyboardShortcuts.delete(shortcut.toLowerCase());
      }
      this.overlays.delete(name);
    }
  }

  getOverlay<T extends IOverlay>(name: string): T | undefined {
    return this.overlays.get(name) as T | undefined;
  }

  toggleOverlay(name: string): void {
    const overlay = this.overlays.get(name);
    if (overlay) {
      overlay.toggle();
    }
  }

  showOverlay(name: string): void {
    const overlay = this.overlays.get(name);
    if (overlay) {
      overlay.show();
    }
  }

  hideOverlay(name: string): void {
    const overlay = this.overlays.get(name);
    if (overlay) {
      overlay.hide();
    }
  }

  isOverlayVisible(name: string): boolean {
    const overlay = this.overlays.get(name);
    return overlay?.isVisible() ?? false;
  }

  getAllOverlays(): IOverlay[] {
    return Array.from(this.overlays.values());
  }

  getDevToolsOverlays(): IOverlay[] {
    return this.getAllOverlays().filter((o) => o.config.devToolsOnly);
  }

  getGameOverlays(): IOverlay[] {
    return this.getAllOverlays().filter((o) => !o.config.devToolsOnly);
  }

  enableKeyboardShortcuts(): void {
    if (this.keydownHandler) return;

    this.keydownHandler = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
        return;
      }

      const key = event.key.toLowerCase();
      const overlayName = this.keyboardShortcuts.get(key);

      if (overlayName) {
        this.toggleOverlay(overlayName);
      }
    };

    window.addEventListener('keydown', this.keydownHandler);
  }

  disableKeyboardShortcuts(): void {
    if (this.keydownHandler) {
      window.removeEventListener('keydown', this.keydownHandler);
      this.keydownHandler = null;
    }
  }

  destroy(): void {
    this.disableKeyboardShortcuts();

    for (const overlay of this.overlays.values()) {
      overlay.destroy();
    }

    this.overlays.clear();
    this.keyboardShortcuts.clear();
  }
}

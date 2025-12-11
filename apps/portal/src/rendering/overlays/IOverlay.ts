import type { Container } from 'pixi.js';

export interface OverlayConfig {
  name: string;
  devToolsOnly: boolean;
  keyboardShortcut?: string;
}

export interface IOverlay {
  readonly config: OverlayConfig;

  show(): void;
  hide(): void;
  toggle(): void;
  isVisible(): boolean;
  destroy(): void;
}

export abstract class BaseOverlay implements IOverlay {
  abstract readonly config: OverlayConfig;
  protected visible: boolean = false;
  protected container: Container;

  constructor(container: Container) {
    this.container = container;
  }

  show(): void {
    this.visible = true;
    this.onVisibilityChange(true);
  }

  hide(): void {
    this.visible = false;
    this.onVisibilityChange(false);
  }

  toggle(): void {
    if (this.visible) {
      this.hide();
    } else {
      this.show();
    }
  }

  isVisible(): boolean {
    return this.visible;
  }

  abstract destroy(): void;

  protected onVisibilityChange(_visible: boolean): void {
    // Override in subclasses for custom show/hide behavior
  }
}

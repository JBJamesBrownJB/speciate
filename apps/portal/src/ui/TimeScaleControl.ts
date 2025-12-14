export interface TimeScaleControlOptions {
  containerId: string;
  onTimeScaleChange?: (scale: number) => void;
}

export const TIME_SCALE_OPTIONS = [0.5, 1, 2, 4] as const;

export class TimeScaleControl {
  private container: HTMLElement | null = null;
  private buttons: Map<number, HTMLButtonElement> = new Map();
  private currentScale: number = 1;
  private onTimeScaleChange?: (scale: number) => void;

  constructor(options: TimeScaleControlOptions) {
    this.container = document.getElementById(options.containerId);
    this.onTimeScaleChange = options.onTimeScaleChange;
    this.setupButtons();
    this.updateActiveState();
  }

  private setupButtons(): void {
    if (!this.container) return;

    for (const scale of TIME_SCALE_OPTIONS) {
      const button = this.container.querySelector(`[data-scale="${scale}"]`) as HTMLButtonElement | null;
      if (button) {
        this.buttons.set(scale, button);
        button.addEventListener('click', () => this.setTimeScale(scale));
      }
    }
  }

  setTimeScale(scale: number): void {
    if (this.currentScale === scale) return;

    this.currentScale = scale;
    this.updateActiveState();
    this.onTimeScaleChange?.(scale);
  }

  getTimeScale(): number {
    return this.currentScale;
  }

  private updateActiveState(): void {
    for (const [scale, button] of this.buttons) {
      if (scale === this.currentScale) {
        button.classList.add('active');
        button.setAttribute('aria-pressed', 'true');
      } else {
        button.classList.remove('active');
        button.setAttribute('aria-pressed', 'false');
      }
    }
  }
}

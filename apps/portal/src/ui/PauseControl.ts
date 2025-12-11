export interface PauseControlOptions {
  buttonId: string;
  onPauseChange?: (paused: boolean) => void;
}

const PAUSE_ICON = '\u23F8';
const PLAY_ICON = '\u25B6';

export class PauseControl {
  private button: HTMLButtonElement | null = null;
  private paused = false;
  private onPauseChange?: (paused: boolean) => void;
  private keydownHandler: ((event: KeyboardEvent) => void) | null = null;

  constructor(options: PauseControlOptions) {
    this.button = document.getElementById(options.buttonId) as HTMLButtonElement | null;
    this.onPauseChange = options.onPauseChange;
    this.setupClickHandler();
    this.updateButtonIcon();
  }

  private setupClickHandler(): void {
    if (this.button) {
      this.button.addEventListener('click', () => this.toggle());
    }
  }

  toggle(): void {
    this.setPaused(!this.paused);
  }

  setPaused(paused: boolean): void {
    if (this.paused === paused) return;

    this.paused = paused;
    this.updateButtonIcon();
    this.onPauseChange?.(paused);
  }

  isPaused(): boolean {
    return this.paused;
  }

  private updateButtonIcon(): void {
    if (this.button) {
      this.button.textContent = this.paused ? PLAY_ICON : PAUSE_ICON;
      this.button.setAttribute(
        'aria-label',
        this.paused ? 'Resume simulation' : 'Pause simulation'
      );
    }
  }

  enableKeyboardShortcut(): () => void {
    this.keydownHandler = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        this.toggle();
      }
    };
    window.addEventListener('keydown', this.keydownHandler);

    return () => {
      if (this.keydownHandler) {
        window.removeEventListener('keydown', this.keydownHandler);
        this.keydownHandler = null;
      }
    };
  }
}

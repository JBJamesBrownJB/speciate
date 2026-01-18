export type ToolType = 'select' | 'terrain' | 'eraser';

export interface ToolbarOptions {
  onToolChange?: (tool: ToolType) => void;
}

export class Toolbar {
  private currentTool: ToolType = 'select';
  private buttons: Map<ToolType, HTMLButtonElement> = new Map();
  private onToolChange?: (tool: ToolType) => void;
  private keydownHandler: ((event: KeyboardEvent) => void) | null = null;

  constructor(options: ToolbarOptions = {}) {
    this.onToolChange = options.onToolChange;
    this.setupButtons();
  }

  private setupButtons(): void {
    const toolIds: [ToolType, string][] = [
      ['select', 'tool-select'],
      ['terrain', 'tool-terrain'],
      ['eraser', 'tool-eraser'],
    ];

    for (const [tool, id] of toolIds) {
      const button = document.getElementById(id) as HTMLButtonElement | null;
      if (button) {
        this.buttons.set(tool, button);
        button.addEventListener('click', () => this.setTool(tool));
      }
    }
  }

  setTool(tool: ToolType): void {
    if (this.currentTool === tool) return;

    this.currentTool = tool;
    this.updateButtonStates();
    this.onToolChange?.(tool);
  }

  getTool(): ToolType {
    return this.currentTool;
  }

  private updateButtonStates(): void {
    for (const [tool, button] of this.buttons) {
      if (tool === this.currentTool) {
        button.classList.add('active');
        button.setAttribute('aria-pressed', 'true');
      } else {
        button.classList.remove('active');
        button.setAttribute('aria-pressed', 'false');
      }
    }
  }

  enableKeyboardShortcuts(): () => void {
    this.keydownHandler = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
        return;
      }

      switch (event.key.toLowerCase()) {
        case 'v':
          this.setTool('select');
          break;
        case 'b':
          this.setTool('terrain');
          break;
        case 'e':
          this.setTool('eraser');
          break;
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

export type ToolId = 'plant';

export class ToolsPanel {
  activeTool: ToolId | null = null;
  onToolChange?: (tool: ToolId | null) => void;

  private plantBtn: HTMLElement | null;

  constructor() {
    this.plantBtn = document.getElementById('tool-plant');
    this.plantBtn?.addEventListener('click', () => this.toggleTool('plant'));
  }

  toggleTool(tool: ToolId): void {
    const next: ToolId | null = this.activeTool === tool ? null : tool;
    this.activeTool = next;
    this.plantBtn?.classList.toggle('active', next === 'plant');
    this.onToolChange?.(next);
  }
}

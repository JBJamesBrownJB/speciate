import type { CreatureData } from '../types/GameState';

export interface ExtendedCreatureData {
  energy?: number;
  behavior?: string;
}

export class CreatureInfoPanel {
  private container: HTMLDivElement;
  private visible: boolean = false;

  constructor(parentElement: HTMLElement) {
    this.container = document.createElement('div');
    this.container.className = 'creature-info-panel';
    this.container.style.cssText = `
      position: absolute;
      top: 80px;
      left: 10px;
      background: rgba(0, 0, 0, 0.85);
      color: white;
      padding: 12px 16px;
      border-radius: 6px;
      font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace;
      font-size: 12px;
      line-height: 1.6;
      min-width: 180px;
      display: none;
      border: 1px solid rgba(255, 255, 255, 0.1);
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
      z-index: 1000;
    `;
    parentElement.appendChild(this.container);
  }

  show(creature: CreatureData, extended?: ExtendedCreatureData): void {
    this.visible = true;
    this.render(creature, extended);
    this.container.style.display = 'block';
  }

  hide(): void {
    this.visible = false;
    this.container.style.display = 'none';
  }

  isVisible(): boolean {
    return this.visible;
  }

  update(creature: CreatureData, extended?: ExtendedCreatureData): void {
    if (!this.visible) return;
    this.render(creature, extended);
  }

  destroy(): void {
    this.container.remove();
  }

  private render(creature: CreatureData, extended?: ExtendedCreatureData): void {
    const lines: string[] = [
      `<div style="color: #ffff00; font-weight: bold; margin-bottom: 8px;">Creature #${creature.id}</div>`,
      `<div><span style="color: #888;">Position:</span> (${creature.x.toFixed(1)}, ${creature.y.toFixed(1)})</div>`,
      `<div><span style="color: #888;">Size:</span> ${creature.size.toFixed(1)}m</div>`,
    ];

    if (extended?.energy !== undefined) {
      lines.push(`<div><span style="color: #888;">Energy:</span> ${extended.energy.toFixed(1)}</div>`);
    }

    if (extended?.behavior) {
      lines.push(`<div><span style="color: #888;">Behavior:</span> ${extended.behavior}</div>`);
    }

    this.container.innerHTML = lines.join('');
  }
}

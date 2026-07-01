import type { CreatureData, PerceptionDebugData } from '../types/GameState';

export interface ExtendedCreatureData {
  energy?: number;
  behavior?: string;
}

export class CreatureInfoPanel {
  private container: HTMLDivElement;
  private visible: boolean = false;
  private lastDebugData: PerceptionDebugData | null = null;
  private lastRenderKey = '';

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
    this.lastDebugData = null;
    this.lastRenderKey = '';
    this.container.style.display = 'none';
  }

  isVisible(): boolean {
    return this.visible;
  }

  update(creature: CreatureData, extended?: ExtendedCreatureData): void {
    if (!this.visible) return;
    this.render(creature, extended);
  }

  updateDebugData(debugData: PerceptionDebugData | null): void {
    this.lastDebugData = debugData;
  }

  destroy(): void {
    this.container.remove();
  }

  /** Key over every value the panel displays, at display precision. Called every
   *  frame — rebuilding innerHTML for an unchanged display is pure DOM churn. */
  private renderKey(creature: CreatureData, extended?: ExtendedCreatureData): string {
    const debug = this.lastDebugData
      ? `${this.lastDebugData.ax.toFixed(2)},${this.lastDebugData.ay.toFixed(2)}`
      : '';
    return `${creature.id}|${creature.x.toFixed(1)}|${creature.y.toFixed(1)}|` +
      `${creature.size.toFixed(1)}|${extended?.energy?.toFixed(1) ?? ''}|` +
      `${extended?.behavior ?? ''}|${debug}`;
  }

  private render(creature: CreatureData, extended?: ExtendedCreatureData): void {
    const key = this.renderKey(creature, extended);
    if (key === this.lastRenderKey) return;
    this.lastRenderKey = key;

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

    // Show acceleration from debug data
    if (this.lastDebugData) {
      const ax = this.lastDebugData.ax;
      const ay = this.lastDebugData.ay;
      const magnitude = Math.sqrt(ax * ax + ay * ay);
      lines.push(`<div><span style="color: #888;">Accel:</span> (${ax.toFixed(2)}, ${ay.toFixed(2)}) |${magnitude.toFixed(2)}|</div>`);
    }

    // Keyboard legend
    lines.push(`<div style="margin-top: 12px; padding-top: 8px; border-top: 1px solid rgba(255,255,255,0.2);">`);
    lines.push(`<div style="color: #666; font-size: 11px; margin-bottom: 4px;">Overlays:</div>`);
    lines.push(`<div style="color: #888; font-size: 11px;"><span style="color: #4a9eff;">[G]</span> Grid &nbsp; <span style="color: #4a9eff;">[F]</span> Force &nbsp; <span style="color: #4a9eff;">[P]</span> Perception</div>`);
    lines.push(`</div>`);

    this.container.innerHTML = lines.join('');
  }
}

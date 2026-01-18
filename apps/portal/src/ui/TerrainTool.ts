import type { ToolType } from './Toolbar';

export interface TerrainToolOptions {
  getActiveTool: () => ToolType;
  worldToTerrainCell: (worldX: number, worldY: number) => [number, number];
  setTerrainCell: (cellX: number, cellY: number, blocked: boolean) => void;
}

export class TerrainTool {
  private getActiveTool: () => ToolType;
  private worldToTerrainCell: (worldX: number, worldY: number) => [number, number];
  private setTerrainCell: (cellX: number, cellY: number, blocked: boolean) => void;

  private isDragging = false;
  private modifiedCells = new Set<string>();

  constructor(options: TerrainToolOptions) {
    this.getActiveTool = options.getActiveTool;
    this.worldToTerrainCell = options.worldToTerrainCell;
    this.setTerrainCell = options.setTerrainCell;
  }

  isActive(): boolean {
    const tool = this.getActiveTool();
    return tool === 'terrain' || tool === 'eraser';
  }

  handlePointerDown(worldX: number, worldY: number): boolean {
    if (!this.isActive()) return false;

    this.isDragging = true;
    this.modifiedCells.clear();
    this.applyAtPosition(worldX, worldY);
    return true;
  }

  handlePointerMove(worldX: number, worldY: number): boolean {
    if (!this.isActive() || !this.isDragging) return false;

    this.applyAtPosition(worldX, worldY);
    return true;
  }

  handlePointerUp(): boolean {
    if (!this.isDragging) return false;

    this.isDragging = false;
    this.modifiedCells.clear();
    return true;
  }

  private applyAtPosition(worldX: number, worldY: number): void {
    const [cellX, cellY] = this.worldToTerrainCell(worldX, worldY);
    const cellKey = `${cellX},${cellY}`;

    if (this.modifiedCells.has(cellKey)) return;

    this.modifiedCells.add(cellKey);

    const tool = this.getActiveTool();
    const blocked = tool === 'terrain';

    this.setTerrainCell(cellX, cellY, blocked);
  }

  getCursor(): string {
    const tool = this.getActiveTool();
    if (tool === 'terrain') return 'crosshair';
    if (tool === 'eraser') return 'cell';
    return 'default';
  }
}

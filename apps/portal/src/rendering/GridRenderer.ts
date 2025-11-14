import { Graphics, Container } from "pixi.js";
import type { Camera } from "@/domain/Camera";
import type { Viewport } from "@/domain/Viewport";

/**
 * Renders a reference grid in the world to show scale and zoom level.
 * This is a temporary visual aid that will be replaced by terrain tiles.
 * Uses viewport culling to only render visible grid lines for performance.
 */
export class GridRenderer {
  private grid: Graphics;
  private spacing: number; // Grid spacing in meters (fixed at 1m in practice)
  private readonly color: number;
  private readonly alpha: number;
  private readonly baseLineWidth: number;
  private currentZoom: number;
  private camera: Camera | null = null;
  private viewport: Viewport | null = null;

  constructor(
    worldContainer: Container,
    spacing: number,
    color: number = 0x2a2a2a,
    alpha: number = 0.3,
    lineWidth: number = 1,
    initialZoom: number = 1
  ) {
    this.spacing = spacing;
    this.color = color;
    this.alpha = alpha;
    this.baseLineWidth = lineWidth;
    this.currentZoom = initialZoom;

    this.grid = new Graphics();
    // Add grid at index 0 so it renders behind all sprites
    worldContainer.addChildAt(this.grid, 0);

    this.render();
  }

  /**
   * Update grid when zoom changes or camera/viewport are set
   * @param zoom Current camera zoom level (pixels per meter)
   * @param spacing Optional new grid spacing (meters). Currently kept at fixed 1m value.
   * @param camera Camera instance for viewport culling
   * @param viewport Viewport instance for calculating visible bounds
   */
  update(zoom: number, spacing: number | undefined, camera: Camera, viewport: Viewport): void {
    this.currentZoom = zoom;
    if (spacing !== undefined) {
      this.spacing = spacing;
    }
    this.camera = camera;
    this.viewport = viewport;
    this.render();
  }

  /**
   * Clear all grid geometry (used when grid should not be shown)
   */
  clear(): void {
    this.grid.clear();
  }

  /**
   * Draws the grid lines
   * Stroke width is calculated in world space to maintain consistent appearance
   * across different zoom levels: strokeWidthWorldSpace = baseLineWidth / zoom
   * Uses viewport culling to only render visible grid lines
   */
  private render(): void {
    this.grid.clear();

    // Skip rendering if camera/viewport not set yet (e.g., during construction)
    if (!this.camera || !this.viewport) {
      return;
    }

    // Calculate stroke width in world space units
    // At zoom 100 with base width 1: strokeWidth = 1/100 = 0.01 world units
    // When worldContainer scales by 100x: 0.01 × 100 = 1 pixel on screen
    const strokeWidthWorldSpace = this.baseLineWidth / this.currentZoom;

    const bounds = this.viewport.getWorldBounds(this.camera);

    // Add padding to ensure grid lines just outside viewport are rendered
    const padding = this.spacing * 2;
    const minX = bounds.minX - padding;
    const maxX = bounds.maxX + padding;
    const minY = bounds.minY - padding;
    const maxY = bounds.maxY + padding;

    // Round to nearest grid line
    const startX = Math.floor(minX / this.spacing) * this.spacing;
    const endX = Math.ceil(maxX / this.spacing) * this.spacing;
    const startY = Math.floor(minY / this.spacing) * this.spacing;
    const endY = Math.ceil(maxY / this.spacing) * this.spacing;

    // Draw vertical lines (only in visible area)
    for (let x = startX; x <= endX; x += this.spacing) {
      this.grid.moveTo(x, minY).lineTo(x, maxY);
    }

    // Draw horizontal lines (only in visible area)
    for (let y = startY; y <= endY; y += this.spacing) {
      this.grid.moveTo(minX, y).lineTo(maxX, y);
    }

    // Apply stroke style once after all lines are drawn (PixiJS v8 pattern)
    this.grid.stroke({
      width: strokeWidthWorldSpace,
      color: this.color,
      alpha: this.alpha,
    });
  }

  /**
   * Clean up resources
   */
  destroy(): void {
    this.grid.destroy();
  }
}

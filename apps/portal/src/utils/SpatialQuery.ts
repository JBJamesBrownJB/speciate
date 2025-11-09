/**
 * Pure utility functions for spatial calculations.
 * These are separated from domain models to avoid adding simulation logic to the UI.
 */
export class SpatialQuery {
  /**
   * Calculate Euclidean distance between two points.
   */
  static distance(x1: number, y1: number, x2: number, y2: number): number {
    const dx = x2 - x1;
    const dy = y2 - y1;
    return Math.sqrt(dx * dx + dy * dy);
  }

  /**
   * Check if an entity intersects with viewport bounds.
   * Used for viewport culling to avoid rendering off-screen entities.
   */
  static isInViewport(
    entity: { x: number; y: number; width: number; height: number },
    viewportBounds: { minX: number; maxX: number; minY: number; maxY: number }
  ): boolean {
    const halfW = entity.width / 2;
    const halfH = entity.height / 2;

    // Check if entity is completely outside viewport bounds
    return !(
      entity.x + halfW < viewportBounds.minX ||
      entity.x - halfW > viewportBounds.maxX ||
      entity.y + halfH < viewportBounds.minY ||
      entity.y - halfH > viewportBounds.maxY
    );
  }
}

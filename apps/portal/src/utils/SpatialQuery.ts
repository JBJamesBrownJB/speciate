export class SpatialQuery {
  static distance(x1: number, y1: number, x2: number, y2: number): number {
    const dx = x2 - x1;
    const dy = y2 - y1;
    return Math.sqrt(dx * dx + dy * dy);
  }

  static isInViewport(
    entity: { x: number; y: number; size: number },
    viewportBounds: { minX: number; maxX: number; minY: number; maxY: number }
  ): boolean {
    const halfSize = entity.size / 2;

    return !(
      entity.x + halfSize < viewportBounds.minX ||
      entity.x - halfSize > viewportBounds.maxX ||
      entity.y + halfSize < viewportBounds.minY ||
      entity.y - halfSize > viewportBounds.maxY
    );
  }
}

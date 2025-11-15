export class SpatialQuery {
  static distance(x1: number, y1: number, x2: number, y2: number): number {
    const dx = x2 - x1;
    const dy = y2 - y1;
    return Math.sqrt(dx * dx + dy * dy);
  }

  static isInViewport(
    entity: { x: number; y: number; width: number; height: number },
    viewportBounds: { minX: number; maxX: number; minY: number; maxY: number }
  ): boolean {
    const halfW = entity.width / 2;
    const halfH = entity.height / 2;

    return !(
      entity.x + halfW < viewportBounds.minX ||
      entity.x - halfW > viewportBounds.maxX ||
      entity.y + halfH < viewportBounds.minY ||
      entity.y - halfH > viewportBounds.maxY
    );
  }
}

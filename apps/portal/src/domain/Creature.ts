/**
 * Creature represents an entity in the simulation world.
 *
 * This is an immutable value object - all update methods return new instances.
 *
 * Coordinates are in world space (meters).
 */
export class Creature {
  /**
   * Create a new creature
   * @param id Unique identifier
   * @param x X position in world space (meters)
   * @param y Y position in world space (meters)
   * @param rotation Rotation in radians
   * @param width Width in meters
   * @param height Height in meters
   * @param vx Velocity X (m/s) - optional, for extrapolation
   * @param vy Velocity Y (m/s) - optional, for extrapolation
   */
  constructor(
    public readonly id: number,
    public readonly x: number,
    public readonly y: number,
    public readonly rotation: number,
    public readonly width: number,
    public readonly height: number,
    public readonly vx?: number,
    public readonly vy?: number
  ) {
    // Make properties truly readonly by freezing the object
    Object.freeze(this);
  }

  /**
   * Create a new creature with updated position and rotation
   * @param x New X position (meters)
   * @param y New Y position (meters)
   * @param rotation New rotation (radians)
   * @returns New Creature instance
   */
  withTransform(x: number, y: number, rotation: number): Creature {
    return new Creature(
      this.id,
      x,
      y,
      rotation,
      this.width,
      this.height,
      this.vx,
      this.vy
    );
  }

  /**
   * Create a Creature from a network message
   * @param message Message data from WebSocket
   * @returns New Creature instance
   */
  static fromMessage(message: {
    id: number;
    x: number;
    y: number;
    rotation?: number;
    width?: number;
    height?: number;
    vx?: number;
    vy?: number;
  }): Creature {
    return new Creature(
      message.id,
      message.x,
      message.y,
      message.rotation ?? 0,
      message.width ?? 1,
      message.height ?? 1,
      message.vx,
      message.vy
    );
  }
}

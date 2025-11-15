export class Creature {
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
    Object.freeze(this);
  }

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

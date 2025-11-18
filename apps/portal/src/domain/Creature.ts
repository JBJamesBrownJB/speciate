export class Creature {
  constructor(
    public readonly id: number,
    public readonly x: number,
    public readonly y: number,
    public readonly rotation: number,
    public readonly size: number
  ) {
    Object.freeze(this);
  }

  withTransform(x: number, y: number, rotation: number): Creature {
    return new Creature(
      this.id,
      x,
      y,
      rotation,
      this.size
    );
  }

  static fromMessage(message: {
    id: number;
    x: number;
    y: number;
    rotation?: number;
    size?: number;
  }): Creature {
    return new Creature(
      message.id,
      message.x,
      message.y,
      message.rotation ?? 0,
      message.size ?? 1
    );
  }
}

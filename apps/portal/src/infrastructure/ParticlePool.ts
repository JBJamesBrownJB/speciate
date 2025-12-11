import { Particle, Texture } from 'pixi.js';

export class ParticlePool {
  private pool = new Map<number, Particle>();
  private active = new Set<number>();

  acquire(entityId: number, texture: Texture): Particle {
    let particle = this.pool.get(entityId);
    if (!particle) {
      particle = new Particle({
        texture,
        anchorX: 0.5,
        anchorY: 0.5,
      });
      this.pool.set(entityId, particle);
    }
    this.active.add(entityId);
    return particle;
  }

  release(entityId: number): void {
    this.active.delete(entityId);
  }

  isActive(entityId: number): boolean {
    return this.active.has(entityId);
  }

  getPoolSize(): number {
    return this.pool.size;
  }

  getActiveCount(): number {
    return this.active.size;
  }

  getActiveIds(): IterableIterator<number> {
    return this.active.values();
  }

  hasEntity(entityId: number): boolean {
    return this.pool.has(entityId);
  }

  private staleBuffer: number[] = [];

  beginFrame(): void {
    this.active.clear();
  }

  getStaleEntities(): number[] {
    this.staleBuffer.length = 0;
    for (const id of this.pool.keys()) {
      if (!this.active.has(id)) {
        this.staleBuffer.push(id);
      }
    }
    return this.staleBuffer;
  }

  removeEntity(entityId: number): Particle | undefined {
    const particle = this.pool.get(entityId);
    this.pool.delete(entityId);
    this.active.delete(entityId);
    return particle;
  }
}

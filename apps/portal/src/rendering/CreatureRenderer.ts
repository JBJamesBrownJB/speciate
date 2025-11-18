import type { ParticleContainer, Texture } from "pixi.js";
import type { ParticlePool } from "@/infrastructure/ParticlePool";
import type { CreatureData } from "@/types/GameState";

export class CreatureRenderer {
  private particlePool: ParticlePool;
  private particleContainer: ParticleContainer;
  private texture: Texture;

  constructor(
    particlePool: ParticlePool,
    particleContainer: ParticleContainer,
    texture: Texture
  ) {
    this.particlePool = particlePool;
    this.particleContainer = particleContainer;
    this.texture = texture;
  }

  render(creatures: CreatureData[]): void {
    const textureWidth = this.texture.width;

    const currentIds = new Set<number>();
    for (let i = 0; i < creatures.length; i++) {
      const c = creatures[i];
      currentIds.add(c.id);
      const isNew = !this.particlePool.hasEntity(c.id);
      const particle = this.particlePool.acquire(c.id, this.texture);

      particle.x = c.x;
      particle.y = c.y;
      particle.rotation = c.rotation;

      const worldScale = c.size / textureWidth;
      particle.scaleX = worldScale;
      particle.scaleY = worldScale;

      if (isNew) {
        this.particleContainer.addParticle(particle);
      }
    }

    const staleIds = this.particlePool.getStaleEntities(currentIds);
    for (const id of staleIds) {
      const particle = this.particlePool.removeEntity(id);
      if (particle) {
        this.particleContainer.removeParticle(particle);
      }
    }
  }
}

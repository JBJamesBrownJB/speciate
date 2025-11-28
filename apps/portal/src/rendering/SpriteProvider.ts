import { Assets, Texture } from 'pixi.js';

export class SpriteProvider {
  private placeholderTexture?: Texture;
  private initialized = false;

  async init(): Promise<void> {
    if (this.initialized) {
      return;
    }

    try {
      const assetUrl = 'blimp-alpha.png';
      this.placeholderTexture = await Assets.load(assetUrl);
      this.initialized = true;
    } catch (error) {
      console.error('Failed to load placeholder sprite:', error);
      throw error;
    }
  }

  getCreatureTexture(_speciesId?: number): Texture {
    if (!this.initialized || !this.placeholderTexture) {
      throw new Error('SpriteProvider not initialized. Call init() first.');
    }

    return this.placeholderTexture;
  }

  getPlaceholderDimensions(): { width: number; height: number } {
    if (!this.placeholderTexture) {
      throw new Error('SpriteProvider not initialized');
    }

    return {
      width: this.placeholderTexture.width,
      height: this.placeholderTexture.height,
    };
  }
}

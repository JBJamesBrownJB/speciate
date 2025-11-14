import { Assets, Texture } from 'pixi.js';

/**
 * Provides sprite textures for creatures
 *
 * Currently returns placeholder sprite for all creatures.
 * Future: Support multiple species with different sprites.
 */
export class SpriteProvider {
  private placeholderTexture?: Texture;
  private initialized = false;

  /**
   * Load all sprite textures
   */
  async init(): Promise<void> {
    if (this.initialized) {
      return;
    }

    try {
      // Load the placeholder sprite
      const assetUrl = 'placeholder.png';
      this.placeholderTexture = await Assets.load(assetUrl);
      this.initialized = true;
    } catch (error) {
      console.error('❌ Failed to load placeholder sprite:', error);
      throw error;
    }
  }

  /**
   * Get texture for a creature
   *
   * @param _speciesId - Species identifier (not used yet, prefixed with _ to indicate intentionally unused)
   * @returns Texture to use for rendering
   */
  getCreatureTexture(_speciesId?: number): Texture {
    if (!this.initialized || !this.placeholderTexture) {
      throw new Error('SpriteProvider not initialized. Call init() first.');
    }

    // For now, always return placeholder regardless of species
    return this.placeholderTexture;
  }

  /**
   * Get the dimensions of the placeholder sprite
   */
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

import { describe, it, expect } from 'vitest';
// @ts-ignore - plain CJS module under test; types not needed for this spec
import { validateCommand } from './commandValidation.cjs';

/**
 * Renderer → main command validation (salvaged from the retired stdio path's
 * COMMAND_VALIDATORS, which the NAPI migration dropped). The renderer is
 * sandboxed and semi-trusted; the main process must not forward garbage —
 * especially `template`, which reaches Rust file loading.
 */
describe('validateCommand', () => {
  describe('dev_spawn_creature', () => {
    it('accepts finite in-bounds coordinates', () => {
      expect(() =>
        validateCommand({ type: 'dev_spawn_creature', x: 100, y: -200 })
      ).not.toThrow();
    });

    it('accepts optional DNA genes when they are finite numbers', () => {
      expect(() =>
        validateCommand({
          type: 'dev_spawn_creature',
          x: 0,
          y: 0,
          dna: { size_gene: 0.5, fov_gene: 1.2 },
        })
      ).not.toThrow();
    });

    it('rejects missing or non-numeric coordinates', () => {
      expect(() => validateCommand({ type: 'dev_spawn_creature' })).toThrow();
      expect(() =>
        validateCommand({ type: 'dev_spawn_creature', x: '10', y: 5 })
      ).toThrow();
    });

    it('rejects NaN / Infinity coordinates', () => {
      expect(() =>
        validateCommand({ type: 'dev_spawn_creature', x: NaN, y: 0 })
      ).toThrow();
      expect(() =>
        validateCommand({ type: 'dev_spawn_creature', x: 0, y: Infinity })
      ).toThrow();
    });

    it('rejects out-of-world coordinates', () => {
      expect(() =>
        validateCommand({ type: 'dev_spawn_creature', x: 2_000_000, y: 0 })
      ).toThrow();
    });

    it('rejects non-numeric DNA genes', () => {
      expect(() =>
        validateCommand({
          type: 'dev_spawn_creature',
          x: 0,
          y: 0,
          dna: { size_gene: 'big' },
        })
      ).toThrow();
    });
  });

  describe('dev_load_trial', () => {
    it('accepts a plain template name', () => {
      expect(() =>
        validateCommand({ type: 'dev_load_trial', template: 'plains_200k' })
      ).not.toThrow();
    });

    it('accepts the category/name spec format the loader supports (e.g. behavior/opposing-seekers)', () => {
      // Real names from apps/simulation/specs/ — see dev-ui trial-templates.ts
      expect(() =>
        validateCommand({ type: 'dev_load_trial', template: 'behavior/opposing-seekers' })
      ).not.toThrow();
      expect(() =>
        validateCommand({ type: 'dev_load_trial', template: 'performance/100k_medium_sparse' })
      ).not.toThrow();
    });

    it('rejects empty or non-string templates', () => {
      expect(() => validateCommand({ type: 'dev_load_trial', template: '' })).toThrow();
      expect(() => validateCommand({ type: 'dev_load_trial', template: 42 })).toThrow();
      expect(() => validateCommand({ type: 'dev_load_trial' })).toThrow();
    });

    it('rejects anything that could escape the specs/trials directories', () => {
      expect(() =>
        validateCommand({ type: 'dev_load_trial', template: '../secrets' })
      ).toThrow();
      expect(() =>
        validateCommand({ type: 'dev_load_trial', template: 'behavior/../../secrets' })
      ).toThrow();
      expect(() =>
        validateCommand({ type: 'dev_load_trial', template: 'a\\b' })
      ).toThrow();
      expect(() =>
        validateCommand({ type: 'dev_load_trial', template: '/etc/passwd' })
      ).toThrow();
      expect(() =>
        validateCommand({ type: 'dev_load_trial', template: 'C:/windows' })
      ).toThrow();
      expect(() =>
        validateCommand({ type: 'dev_load_trial', template: 'a//b' })
      ).toThrow();
      expect(() =>
        validateCommand({ type: 'dev_load_trial', template: 'a/./b' })
      ).toThrow();
    });
  });

  describe('parameterless commands', () => {
    it('accepts dev_clear_creatures and dev_clear_plants', () => {
      expect(() => validateCommand({ type: 'dev_clear_creatures' })).not.toThrow();
      expect(() => validateCommand({ type: 'dev_clear_plants' })).not.toThrow();
    });
  });

  describe('whitelist', () => {
    it('rejects unknown command types', () => {
      expect(() => validateCommand({ type: 'rm_rf_slash' })).toThrow();
    });

    it('rejects non-object commands', () => {
      expect(() => validateCommand(null)).toThrow();
      expect(() => validateCommand('dev_spawn_creature')).toThrow();
    });
  });
});

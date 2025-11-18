import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { ScaleBarManager } from './ScaleBarManager';
import { SCALE_BAR_CONFIG } from '@/core/constants';
import { createMockElement, mockDocumentGetElementById } from '@/test-utils';

describe('ScaleBarManager', () => {
  let mockScaleLine: HTMLElement;
  let mockScaleLabel: HTMLElement;

  beforeEach(() => {
    mockScaleLine = createMockElement();
    mockScaleLabel = createMockElement();

    mockDocumentGetElementById({
      'scale-line': mockScaleLine,
      'scale-label': mockScaleLabel,
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('constructor', () => {
    it('should get DOM elements by ID', () => {
      new ScaleBarManager('scale-line', 'scale-label');

      expect(document.getElementById).toHaveBeenCalledWith('scale-line');
      expect(document.getElementById).toHaveBeenCalledWith('scale-label');
    });

    it('should handle missing elements gracefully', () => {
      mockDocumentGetElementById({});

      expect(() => new ScaleBarManager('missing', 'missing')).not.toThrow();
    });
  });

  describe('update', () => {
    it('should update line width and label for low zoom', () => {
      const manager = new ScaleBarManager('scale-line', 'scale-label');

      manager.update(0.1);

      expect(mockScaleLine.style.width).toBe('100px');
      expect(mockScaleLabel.textContent).toBe('1km');
    });

    it('should update line width and label for medium zoom', () => {
      const manager = new ScaleBarManager('scale-line', 'scale-label');

      manager.update(1);

      expect(mockScaleLine.style.width).toBe('100px');
      expect(mockScaleLabel.textContent).toBe('100m');
    });

    it('should update line width and label for high zoom', () => {
      const manager = new ScaleBarManager('scale-line', 'scale-label');

      manager.update(10);

      expect(mockScaleLine.style.width).toBe('100px');
      expect(mockScaleLabel.textContent).toBe('10m');
    });

    it('should select closest nice interval', () => {
      const manager = new ScaleBarManager('scale-line', 'scale-label');

      manager.update(2.4);

      expect(mockScaleLine.style.width).toBe('120px');
      expect(mockScaleLabel.textContent).toBe('50m');
    });

    it('should format kilometers for distances >= 1000m', () => {
      const manager = new ScaleBarManager('scale-line', 'scale-label');

      manager.update(0.024);
      expect(mockScaleLabel.textContent).toBe('5km');
    });

    it('should format meters for distances < 1000m', () => {
      const manager = new ScaleBarManager('scale-line', 'scale-label');

      manager.update(12);
      expect(mockScaleLabel.textContent).toBe('10m');
    });

    it('should handle null elements without errors', () => {
      mockDocumentGetElementById({});
      const manager = new ScaleBarManager('missing', 'missing');

      expect(() => manager.update(10)).not.toThrow();
    });

    it('should test all SCALE_BAR_CONFIG intervals', () => {
      const manager = new ScaleBarManager('scale-line', 'scale-label');

      const intervals = SCALE_BAR_CONFIG.NICE_INTERVALS;

      intervals.forEach((interval) => {
        const zoom = SCALE_BAR_CONFIG.TARGET_PIXEL_WIDTH / interval;
        manager.update(zoom);

        const expectedLabel = interval >= 1000
          ? `${interval / 1000}km`
          : `${interval}m`;

        expect(mockScaleLabel.textContent).toBe(expectedLabel);
      });
    });

    it('should handle very low zoom (MIN_ZOOM)', () => {
      const manager = new ScaleBarManager('scale-line', 'scale-label');

      manager.update(0.0005);

      expect(mockScaleLabel.textContent).toMatch(/km$/);
    });

    it('should handle very high zoom (MAX_ZOOM)', () => {
      const manager = new ScaleBarManager('scale-line', 'scale-label');

      manager.update(400);

      expect(mockScaleLabel.textContent).toMatch(/m$/);
    });

    it('should calculate correct pixel width for each interval', () => {
      const manager = new ScaleBarManager('scale-line', 'scale-label');

      manager.update(2);
      const width1 = parseFloat(mockScaleLine.style.width);

      manager.update(5);
      const width2 = parseFloat(mockScaleLine.style.width);

      expect(width1).toBeGreaterThan(0);
      expect(width2).toBeGreaterThan(0);
    });

    it('should handle zoom resulting in ideal distance between intervals', () => {
      const manager = new ScaleBarManager('scale-line', 'scale-label');

      manager.update(1.2);

      expect(mockScaleLabel.textContent).toMatch(/^[0-9]+(km|m)$/);
    });

    it('should select 1m for very high zoom values', () => {
      const manager = new ScaleBarManager('scale-line', 'scale-label');

      manager.update(200);

      expect(mockScaleLabel.textContent).toBe('1m');
    });

    it('should select large intervals for very low zoom', () => {
      const manager = new ScaleBarManager('scale-line', 'scale-label');

      manager.update(0.0001);

      expect(mockScaleLabel.textContent).toBe('1000km');
    });
  });
});

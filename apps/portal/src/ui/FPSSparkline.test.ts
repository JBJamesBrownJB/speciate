import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { FPSSparkline } from './FPSSparkline';
import { createMockCanvas, setDevicePixelRatio, resetDevicePixelRatio } from '@/test-utils';

describe('FPSSparkline', () => {
  let mockCanvas: ReturnType<typeof createMockCanvas>;

  beforeEach(() => {
    setDevicePixelRatio(1);
    mockCanvas = createMockCanvas();
    vi.spyOn(document, 'getElementById').mockReturnValue(mockCanvas.canvas);
  });

  afterEach(() => {
    vi.restoreAllMocks();
    resetDevicePixelRatio();
  });

  describe('constructor', () => {
    it('should throw if canvas not found', () => {
      vi.spyOn(document, 'getElementById').mockReturnValue(null);
      expect(() => new FPSSparkline('nonexistent')).toThrow('Canvas nonexistent not found');
    });

    it('should throw if 2d context not available', () => {
      mockCanvas.canvas.getContext = vi.fn().mockReturnValue(null);
      expect(() => new FPSSparkline('fps-sparkline')).toThrow('Could not get 2d context');
    });

    it('should set canvas dimensions based on devicePixelRatio', () => {
      setDevicePixelRatio(2);
      mockCanvas = createMockCanvas();
      vi.spyOn(document, 'getElementById').mockReturnValue(mockCanvas.canvas);

      new FPSSparkline('fps-sparkline');

      expect(mockCanvas.canvas.width).toBe(400);
      expect(mockCanvas.canvas.height).toBe(120);
      expect(mockCanvas.spies.scale).toHaveBeenCalledWith(2, 2);
    });

    it('should handle missing devicePixelRatio (default to 1)', () => {
      resetDevicePixelRatio();
      mockCanvas = createMockCanvas();
      vi.spyOn(document, 'getElementById').mockReturnValue(mockCanvas.canvas);

      new FPSSparkline('fps-sparkline');

      expect(mockCanvas.canvas.width).toBe(200);
      expect(mockCanvas.canvas.height).toBe(60);
    });

    it('should call getContext with "2d"', () => {
      new FPSSparkline('fps-sparkline');

      expect(mockCanvas.canvas.getContext).toHaveBeenCalledWith('2d');
    });
  });

  describe('update', () => {
    it('should add fps value to history', () => {
      const sparkline = new FPSSparkline('fps-sparkline');

      sparkline.update(60);
      sparkline.update(58);

      expect(mockCanvas.spies.clearRect).toHaveBeenCalledTimes(2);
    });

    it('should limit history to maxHistory entries', () => {
      const sparkline = new FPSSparkline('fps-sparkline');

      for (let i = 0; i < 100; i++) {
        sparkline.update(60);
      }

      expect(mockCanvas.spies.clearRect).toHaveBeenCalledTimes(100);
    });

    it('should not render line with less than 2 points', () => {
      const sparkline = new FPSSparkline('fps-sparkline');

      sparkline.update(60);

      expect(mockCanvas.spies.clearRect).toHaveBeenCalled();
      expect(mockCanvas.spies.lineTo).not.toHaveBeenCalled();
    });

    it('should render line with 2+ points', () => {
      const sparkline = new FPSSparkline('fps-sparkline');

      sparkline.update(60);
      sparkline.update(58);

      expect(mockCanvas.spies.moveTo).toHaveBeenCalled();
      expect(mockCanvas.spies.lineTo).toHaveBeenCalled();
    });

    it('should clear canvas before each render', () => {
      const sparkline = new FPSSparkline('fps-sparkline');

      sparkline.update(60);

      expect(mockCanvas.spies.clearRect).toHaveBeenCalledWith(0, 0, 200, 60);
    });

    it('should call beginPath for FPS line', () => {
      const sparkline = new FPSSparkline('fps-sparkline');

      sparkline.update(60);
      sparkline.update(58);

      expect(mockCanvas.spies.beginPath).toHaveBeenCalled();
    });

    it('should call stroke to draw the line', () => {
      const sparkline = new FPSSparkline('fps-sparkline');

      sparkline.update(60);
      sparkline.update(58);

      expect(mockCanvas.spies.stroke).toHaveBeenCalled();
    });

    it('should draw reference line at top', () => {
      const sparkline = new FPSSparkline('fps-sparkline');

      sparkline.update(60);
      sparkline.update(58);

      expect(mockCanvas.spies.setLineDash).toHaveBeenCalledWith([2, 2]);
      expect(mockCanvas.spies.setLineDash).toHaveBeenCalledWith([]);
    });

    it('should set correct stroke style for FPS line', () => {
      const sparkline = new FPSSparkline('fps-sparkline');
      let capturedStrokeStyle = '';

      Object.defineProperty(mockCanvas.context, 'strokeStyle', {
        get: () => capturedStrokeStyle,
        set: (value: string) => {
          capturedStrokeStyle = value;
        },
        configurable: true,
      });

      sparkline.update(60);
      sparkline.update(58);

      expect(capturedStrokeStyle).toContain('111, 184, 63');
    });

    it('should set line width for FPS line', () => {
      const sparkline = new FPSSparkline('fps-sparkline');

      sparkline.update(60);
      sparkline.update(58);

      expect(mockCanvas.context.lineWidth).toBeGreaterThan(0);
    });

    it('should handle rapid consecutive updates', () => {
      const sparkline = new FPSSparkline('fps-sparkline');

      for (let i = 0; i < 10; i++) {
        sparkline.update(60 + i);
      }

      expect(mockCanvas.spies.clearRect).toHaveBeenCalledTimes(10);
    });

    it('should handle zero FPS value', () => {
      const sparkline = new FPSSparkline('fps-sparkline');

      sparkline.update(0);
      sparkline.update(60);

      expect(mockCanvas.spies.moveTo).toHaveBeenCalled();
    });

    it('should handle very high FPS values', () => {
      const sparkline = new FPSSparkline('fps-sparkline');

      sparkline.update(200);
      sparkline.update(150);

      expect(mockCanvas.spies.moveTo).toHaveBeenCalled();
    });

    it('should handle negative FPS values', () => {
      const sparkline = new FPSSparkline('fps-sparkline');

      sparkline.update(-10);
      sparkline.update(60);

      expect(mockCanvas.spies.moveTo).toHaveBeenCalled();
    });
  });
});

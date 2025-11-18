import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { HUDManager } from './HUDManager';
import type { FPSSparkline } from './FPSSparkline';
import { createMockElement, mockDocumentGetElementById } from '@/test-utils';

describe('HUDManager', () => {
  let mockElements: {
    fpsValue: HTMLElement;
    tickRateValue: HTMLElement;
    creatureCount: HTMLElement;
    zoomValue: HTMLElement;
  };
  let mockFpsSparkline: FPSSparkline;

  beforeEach(() => {
    mockElements = {
      fpsValue: createMockElement(),
      tickRateValue: createMockElement(),
      creatureCount: createMockElement(),
      zoomValue: createMockElement(),
    };

    mockFpsSparkline = {
      update: vi.fn(),
    } as unknown as FPSSparkline;

    mockDocumentGetElementById({
      'fps-value': mockElements.fpsValue,
      'tick-rate-value': mockElements.tickRateValue,
      'creature-count': mockElements.creatureCount,
      'zoom-value': mockElements.zoomValue,
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('constructor', () => {
    it('should get all DOM elements by ID', () => {
      new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      expect(document.getElementById).toHaveBeenCalledWith('fps-value');
      expect(document.getElementById).toHaveBeenCalledWith('tick-rate-value');
      expect(document.getElementById).toHaveBeenCalledWith('creature-count');
      expect(document.getElementById).toHaveBeenCalledWith('zoom-value');
    });
  });

  describe('updateFPS', () => {
    it('should update FPS value element', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateFPS(60);

      expect(mockElements.fpsValue.textContent).toBe('60');
    });

    it('should call FPSSparkline.update', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateFPS(60);

      expect(mockFpsSparkline.update).toHaveBeenCalledWith(60);
    });

    it('should handle null FPS element', () => {
      mockDocumentGetElementById({
        'tick-rate-value': mockElements.tickRateValue,
        'creature-count': mockElements.creatureCount,
        'zoom-value': mockElements.zoomValue,
      });

      const hud = new HUDManager(
        {
          fpsValue: 'missing',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      expect(() => hud.updateFPS(60)).not.toThrow();
      expect(mockFpsSparkline.update).toHaveBeenCalledWith(60);
    });

    it('should handle various FPS values', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateFPS(0);
      expect(mockElements.fpsValue.textContent).toBe('0');

      hud.updateFPS(90);
      expect(mockElements.fpsValue.textContent).toBe('90');

      hud.updateFPS(144);
      expect(mockElements.fpsValue.textContent).toBe('144');
    });
  });

  describe('updateTickRate', () => {
    it('should display tick rate with 1 decimal place', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateTickRate(15.67);

      expect(mockElements.tickRateValue.textContent).toBe('15.7 Hz');
    });

    it('should display "..." for negative tick rate', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateTickRate(-1);

      expect(mockElements.tickRateValue.textContent).toBe('...');
    });

    it('should handle zero tick rate', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateTickRate(0);

      expect(mockElements.tickRateValue.textContent).toBe('0.0 Hz');
    });

    it('should handle high tick rates', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateTickRate(90.123);

      expect(mockElements.tickRateValue.textContent).toBe('90.1 Hz');
    });

    it('should handle null tick rate element', () => {
      mockDocumentGetElementById({
        'fps-value': mockElements.fpsValue,
        'creature-count': mockElements.creatureCount,
        'zoom-value': mockElements.zoomValue,
      });

      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'missing',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      expect(() => hud.updateTickRate(15)).not.toThrow();
    });
  });

  describe('updateCreatureCount', () => {
    it('should update creature count element', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateCreatureCount(42);

      expect(mockElements.creatureCount.textContent).toBe('42');
    });

    it('should handle zero creatures', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateCreatureCount(0);

      expect(mockElements.creatureCount.textContent).toBe('0');
    });

    it('should handle large creature counts', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateCreatureCount(10000);

      expect(mockElements.creatureCount.textContent).toBe('10000');
    });

    it('should handle null creature count element', () => {
      mockDocumentGetElementById({
        'fps-value': mockElements.fpsValue,
        'tick-rate-value': mockElements.tickRateValue,
        'zoom-value': mockElements.zoomValue,
      });

      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'missing',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      expect(() => hud.updateCreatureCount(42)).not.toThrow();
    });
  });

  describe('updateZoom', () => {
    it('should display zoom with 2 decimal places and "x" suffix', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateZoom(15.6789);

      expect(mockElements.zoomValue.textContent).toBe('15.68x');
    });

    it('should handle low zoom values', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateZoom(0.0005);

      expect(mockElements.zoomValue.textContent).toBe('0.00x');
    });

    it('should handle high zoom values', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateZoom(400);

      expect(mockElements.zoomValue.textContent).toBe('400.00x');
    });

    it('should handle null zoom element', () => {
      mockDocumentGetElementById({
        'fps-value': mockElements.fpsValue,
        'tick-rate-value': mockElements.tickRateValue,
        'creature-count': mockElements.creatureCount,
      });

      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'missing',
        },
        mockFpsSparkline
      );

      expect(() => hud.updateZoom(10)).not.toThrow();
    });
  });

  describe('integration', () => {
    it('should handle multiple updates in sequence', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureCount: 'creature-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateFPS(60);
      hud.updateTickRate(15.5);
      hud.updateCreatureCount(1000);
      hud.updateZoom(10.5);

      expect(mockElements.fpsValue.textContent).toBe('60');
      expect(mockElements.tickRateValue.textContent).toBe('15.5 Hz');
      expect(mockElements.creatureCount.textContent).toBe('1000');
      expect(mockElements.zoomValue.textContent).toBe('10.50x');
      expect(mockFpsSparkline.update).toHaveBeenCalledWith(60);
    });

    it('should handle all null elements without crashing', () => {
      mockDocumentGetElementById({});

      const hud = new HUDManager(
        {
          fpsValue: 'missing',
          tickRateValue: 'missing',
          creatureCount: 'missing',
          zoomValue: 'missing',
        },
        mockFpsSparkline
      );

      expect(() => {
        hud.updateFPS(60);
        hud.updateTickRate(15);
        hud.updateCreatureCount(100);
        hud.updateZoom(10);
      }).not.toThrow();
    });
  });
});

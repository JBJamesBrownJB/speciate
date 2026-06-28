import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { HUDManager } from './HUDManager';
import type { FPSSparkline } from './FPSSparkline';
import { createMockElement, mockDocumentGetElementById } from '@/test-utils';

describe('HUDManager', () => {
  let mockElements: {
    fpsValue: HTMLElement;
    tickRateValue: HTMLElement;
    creatureWorldCount: HTMLElement;
    creatureScreenCount: HTMLElement;
    plantWorldCount: HTMLElement;
    plantScreenCount: HTMLElement;
    zoomValue: HTMLElement;
  };
  let mockFpsSparkline: FPSSparkline;

  beforeEach(() => {
    mockElements = {
      fpsValue: createMockElement(),
      tickRateValue: createMockElement(),
      creatureWorldCount: createMockElement(),
      creatureScreenCount: createMockElement(),
      plantWorldCount: createMockElement(),
      plantScreenCount: createMockElement(),
      zoomValue: createMockElement(),
    };

    mockFpsSparkline = {
      update: vi.fn(),
    } as unknown as FPSSparkline;

    mockDocumentGetElementById({
      'fps-value': mockElements.fpsValue,
      'tick-rate-value': mockElements.tickRateValue,
      'creature-world-count': mockElements.creatureWorldCount,
      'creature-screen-count': mockElements.creatureScreenCount,
      'plant-world-count': mockElements.plantWorldCount,
      'plant-screen-count': mockElements.plantScreenCount,
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
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      expect(document.getElementById).toHaveBeenCalledWith('fps-value');
      expect(document.getElementById).toHaveBeenCalledWith('tick-rate-value');
      expect(document.getElementById).toHaveBeenCalledWith('creature-world-count');
      expect(document.getElementById).toHaveBeenCalledWith('creature-screen-count');
      expect(document.getElementById).toHaveBeenCalledWith('plant-world-count');
      expect(document.getElementById).toHaveBeenCalledWith('plant-screen-count');
      expect(document.getElementById).toHaveBeenCalledWith('zoom-value');
    });

    it('constructor resolves all four new IDs via getElementById and does not reference creature-count or plant-count', () => {
      new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      expect(document.getElementById).not.toHaveBeenCalledWith('creature-count');
      expect(document.getElementById).not.toHaveBeenCalledWith('plant-count');
    });
  });

  describe('updateFPS', () => {
    it('should update FPS value element', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
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
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
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
        'creature-world-count': mockElements.creatureWorldCount,
        'creature-screen-count': mockElements.creatureScreenCount,
        'plant-world-count': mockElements.plantWorldCount,
        'plant-screen-count': mockElements.plantScreenCount,
        'zoom-value': mockElements.zoomValue,
      });

      const hud = new HUDManager(
        {
          fpsValue: 'missing',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      expect(() => hud.updateFPS(60)).not.toThrow();
      expect(mockFpsSparkline.update).toHaveBeenCalledWith(60);
    });

    it('seeds from the first real sample, then EMA-smooths subsequent values', () => {
      // The FPS readout is intentionally smoothed (EMA, α=0.05, ~1 s time constant)
      // so the number doesn't jitter frame-to-frame. This test pins that contract.
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      // Leading zeros = "no real sample yet": shown as 0, and don't poison the seed.
      hud.updateFPS(0);
      expect(mockElements.fpsValue.textContent).toBe('0');

      // First real sample seeds the display raw (no startup lag from climbing out of 0).
      hud.updateFPS(90);
      expect(mockElements.fpsValue.textContent).toBe('90');

      // Subsequent samples ease toward the new value instead of jumping:
      // 0.05*144 + 0.95*90 = 92.7 → 93.
      hud.updateFPS(144);
      expect(mockElements.fpsValue.textContent).toBe('93');

      // A steady input converges to that value.
      for (let i = 0; i < 300; i++) hud.updateFPS(144);
      expect(mockElements.fpsValue.textContent).toBe('144');
    });
  });

  describe('updateTickRate', () => {
    it('should display tick rate with 1 decimal place', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
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
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
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
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
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
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
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
        'creature-world-count': mockElements.creatureWorldCount,
        'creature-screen-count': mockElements.creatureScreenCount,
        'plant-world-count': mockElements.plantWorldCount,
        'plant-screen-count': mockElements.plantScreenCount,
        'zoom-value': mockElements.zoomValue,
      });

      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'missing',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      expect(() => hud.updateTickRate(15)).not.toThrow();
    });
  });

  describe('updateCreatureWorldCount', () => {
    it('updateCreatureWorldCount sets textContent on the creature-world-count element', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateCreatureWorldCount(42);

      expect(mockElements.creatureWorldCount.textContent).toBe('42');
    });

    it('should handle zero creatures', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateCreatureWorldCount(0);

      expect(mockElements.creatureWorldCount.textContent).toBe('0');
    });

    it('should handle large creature counts', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateCreatureWorldCount(1000000);

      expect(mockElements.creatureWorldCount.textContent).toBe('1000000');
    });

    it('should handle null creature world count element', () => {
      mockDocumentGetElementById({
        'fps-value': mockElements.fpsValue,
        'tick-rate-value': mockElements.tickRateValue,
        'creature-screen-count': mockElements.creatureScreenCount,
        'plant-world-count': mockElements.plantWorldCount,
        'plant-screen-count': mockElements.plantScreenCount,
        'zoom-value': mockElements.zoomValue,
      });

      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'missing',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      expect(() => hud.updateCreatureWorldCount(42)).not.toThrow();
    });
  });

  describe('updateCreatureScreenCount', () => {
    it('updateCreatureScreenCount sets textContent on the creature-screen-count element', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateCreatureScreenCount(7);

      expect(mockElements.creatureScreenCount.textContent).toBe('7');
    });

    it('should handle null creature screen count element', () => {
      mockDocumentGetElementById({
        'fps-value': mockElements.fpsValue,
        'tick-rate-value': mockElements.tickRateValue,
        'creature-world-count': mockElements.creatureWorldCount,
        'plant-world-count': mockElements.plantWorldCount,
        'plant-screen-count': mockElements.plantScreenCount,
        'zoom-value': mockElements.zoomValue,
      });

      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'missing',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      expect(() => hud.updateCreatureScreenCount(7)).not.toThrow();
    });
  });

  describe('updatePlantWorldCount', () => {
    it('updatePlantWorldCount sets textContent on the plant-world-count element', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updatePlantWorldCount(500);

      expect(mockElements.plantWorldCount.textContent).toBe('500');
    });

    it('should handle null plant world count element', () => {
      mockDocumentGetElementById({
        'fps-value': mockElements.fpsValue,
        'tick-rate-value': mockElements.tickRateValue,
        'creature-world-count': mockElements.creatureWorldCount,
        'creature-screen-count': mockElements.creatureScreenCount,
        'plant-screen-count': mockElements.plantScreenCount,
        'zoom-value': mockElements.zoomValue,
      });

      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'missing',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      expect(() => hud.updatePlantWorldCount(500)).not.toThrow();
    });
  });

  describe('updatePlantScreenCount', () => {
    it('updatePlantScreenCount sets textContent on the plant-screen-count element', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updatePlantScreenCount(200);

      expect(mockElements.plantScreenCount.textContent).toBe('200');
    });

    it('should handle null plant screen count element', () => {
      mockDocumentGetElementById({
        'fps-value': mockElements.fpsValue,
        'tick-rate-value': mockElements.tickRateValue,
        'creature-world-count': mockElements.creatureWorldCount,
        'creature-screen-count': mockElements.creatureScreenCount,
        'plant-world-count': mockElements.plantWorldCount,
        'zoom-value': mockElements.zoomValue,
      });

      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'missing',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      expect(() => hud.updatePlantScreenCount(200)).not.toThrow();
    });
  });

  describe('updateZoom', () => {
    it('should display zoom with 2 decimal places and "x" suffix', () => {
      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
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
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
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
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
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
        'creature-world-count': mockElements.creatureWorldCount,
        'creature-screen-count': mockElements.creatureScreenCount,
        'plant-world-count': mockElements.plantWorldCount,
        'plant-screen-count': mockElements.plantScreenCount,
      });

      const hud = new HUDManager(
        {
          fpsValue: 'fps-value',
          tickRateValue: 'tick-rate-value',
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
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
          creatureWorldCount: 'creature-world-count',
          creatureScreenCount: 'creature-screen-count',
          plantWorldCount: 'plant-world-count',
          plantScreenCount: 'plant-screen-count',
          zoomValue: 'zoom-value',
        },
        mockFpsSparkline
      );

      hud.updateFPS(60);
      hud.updateTickRate(15.5);
      hud.updateCreatureWorldCount(1000);
      hud.updateCreatureScreenCount(50);
      hud.updatePlantWorldCount(5000);
      hud.updatePlantScreenCount(200);
      hud.updateZoom(10.5);

      expect(mockElements.fpsValue.textContent).toBe('60');
      expect(mockElements.tickRateValue.textContent).toBe('15.5 Hz');
      expect(mockElements.creatureWorldCount.textContent).toBe('1000');
      expect(mockElements.creatureScreenCount.textContent).toBe('50');
      expect(mockElements.plantWorldCount.textContent).toBe('5000');
      expect(mockElements.plantScreenCount.textContent).toBe('200');
      expect(mockElements.zoomValue.textContent).toBe('10.50x');
      expect(mockFpsSparkline.update).toHaveBeenCalledWith(60);
    });

    it('should handle all null elements without crashing', () => {
      mockDocumentGetElementById({});

      const hud = new HUDManager(
        {
          fpsValue: 'missing',
          tickRateValue: 'missing',
          creatureWorldCount: 'missing',
          creatureScreenCount: 'missing',
          plantWorldCount: 'missing',
          plantScreenCount: 'missing',
          zoomValue: 'missing',
        },
        mockFpsSparkline
      );

      expect(() => {
        hud.updateFPS(60);
        hud.updateTickRate(15);
        hud.updateCreatureWorldCount(100);
        hud.updateCreatureScreenCount(10);
        hud.updatePlantWorldCount(500);
        hud.updatePlantScreenCount(50);
        hud.updateZoom(10);
      }).not.toThrow();
    });
  });
});

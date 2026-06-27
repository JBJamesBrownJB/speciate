import type { FPSSparkline } from "./FPSSparkline";

interface HUDElements {
  fpsValue: HTMLElement | null;
  tickRateValue: HTMLElement | null;
  creatureWorldCount: HTMLElement | null;
  creatureScreenCount: HTMLElement | null;
  plantWorldCount: HTMLElement | null;
  plantScreenCount: HTMLElement | null;
  zoomValue: HTMLElement | null;
}

export class HUDManager {
  private elements: HUDElements;
  private fpsSparkline: FPSSparkline;
  private smoothedFps = 0;

  constructor(
    elementIds: {
      fpsValue: string;
      tickRateValue: string;
      creatureWorldCount: string;
      creatureScreenCount: string;
      plantWorldCount: string;
      plantScreenCount: string;
      zoomValue: string;
    },
    fpsSparkline: FPSSparkline
  ) {
    this.elements = {
      fpsValue: document.getElementById(elementIds.fpsValue),
      tickRateValue: document.getElementById(elementIds.tickRateValue),
      creatureWorldCount: document.getElementById(elementIds.creatureWorldCount),
      creatureScreenCount: document.getElementById(elementIds.creatureScreenCount),
      plantWorldCount: document.getElementById(elementIds.plantWorldCount),
      plantScreenCount: document.getElementById(elementIds.plantScreenCount),
      zoomValue: document.getElementById(elementIds.zoomValue),
    };
    this.fpsSparkline = fpsSparkline;
  }

  updateFPS(fps: number): void {
    // EMA with α=0.05 → ~1 s time constant at 90 FPS; seed from first real sample
    this.smoothedFps = this.smoothedFps === 0 ? fps : 0.05 * fps + 0.95 * this.smoothedFps;
    const display = Math.round(this.smoothedFps).toString();
    if (this.elements.fpsValue && this.elements.fpsValue.textContent !== display) {
      this.elements.fpsValue.textContent = display;
    }
    this.fpsSparkline.update(fps);
  }

  updateTickRate(tickRateHz: number): void {
    if (this.elements.tickRateValue) {
      const tickRateDisplay = tickRateHz < 0 ? "..." : `${tickRateHz.toFixed(1)} Hz`;
      this.elements.tickRateValue.textContent = tickRateDisplay;
    }
  }

  updateCreatureWorldCount(count: number): void {
    if (this.elements.creatureWorldCount) {
      this.elements.creatureWorldCount.textContent = count.toString();
    }
  }

  updateCreatureScreenCount(count: number): void {
    if (this.elements.creatureScreenCount) {
      this.elements.creatureScreenCount.textContent = count.toString();
    }
  }

  updatePlantWorldCount(count: number): void {
    if (this.elements.plantWorldCount) {
      this.elements.plantWorldCount.textContent = count.toString();
    }
  }

  updatePlantScreenCount(count: number): void {
    if (this.elements.plantScreenCount) {
      this.elements.plantScreenCount.textContent = count.toString();
    }
  }

  updateZoom(zoom: number): void {
    if (this.elements.zoomValue) {
      this.elements.zoomValue.textContent = `${zoom.toFixed(2)}x`;
    }
  }
}

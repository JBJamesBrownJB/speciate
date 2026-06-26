import type { FPSSparkline } from "./FPSSparkline";

interface HUDElements {
  fpsValue: HTMLElement | null;
  tickRateValue: HTMLElement | null;
  creatureCount: HTMLElement | null;
  plantCount: HTMLElement | null;
  zoomValue: HTMLElement | null;
}

export class HUDManager {
  private elements: HUDElements;
  private fpsSparkline: FPSSparkline;

  constructor(
    elementIds: {
      fpsValue: string;
      tickRateValue: string;
      creatureCount: string;
      plantCount: string;
      zoomValue: string;
    },
    fpsSparkline: FPSSparkline
  ) {
    this.elements = {
      fpsValue: document.getElementById(elementIds.fpsValue),
      tickRateValue: document.getElementById(elementIds.tickRateValue),
      creatureCount: document.getElementById(elementIds.creatureCount),
      plantCount: document.getElementById(elementIds.plantCount),
      zoomValue: document.getElementById(elementIds.zoomValue),
    };
    this.fpsSparkline = fpsSparkline;
  }

  updateFPS(fps: number): void {
    if (this.elements.fpsValue) {
      this.elements.fpsValue.textContent = fps.toString();
    }
    this.fpsSparkline.update(fps);
  }

  updateTickRate(tickRateHz: number): void {
    if (this.elements.tickRateValue) {
      const tickRateDisplay = tickRateHz < 0 ? "..." : `${tickRateHz.toFixed(1)} Hz`;
      this.elements.tickRateValue.textContent = tickRateDisplay;
    }
  }

  updateCreatureCount(count: number): void {
    if (this.elements.creatureCount) {
      this.elements.creatureCount.textContent = count.toString();
    }
  }

  updatePlantCount(count: number): void {
    if (this.elements.plantCount) {
      this.elements.plantCount.textContent = count.toString();
    }
  }

  updateZoom(zoom: number): void {
    if (this.elements.zoomValue) {
      this.elements.zoomValue.textContent = `${zoom.toFixed(2)}x`;
    }
  }
}

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

// The render loop calls every updater each frame; the sparkline redraws its
// whole canvas per sample, so sample it every Nth frame (~10-15 Hz) instead.
const SPARKLINE_SAMPLE_INTERVAL = 6;

export class HUDManager {
  private elements: HUDElements;
  private fpsSparkline: FPSSparkline;
  private smoothedFps = 0;
  private framesSinceSparklineSample = SPARKLINE_SAMPLE_INTERVAL - 1; // sample on first frame

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

  /** Write only when the displayed string actually changed — these run every
   *  frame, and redundant textContent writes still dirty layout. */
  private setText(el: HTMLElement | null, text: string): void {
    if (el && el.textContent !== text) {
      el.textContent = text;
    }
  }

  updateFPS(fps: number): void {
    // EMA with α=0.05 → ~1 s time constant at 90 FPS; seed from first real sample
    this.smoothedFps = this.smoothedFps === 0 ? fps : 0.05 * fps + 0.95 * this.smoothedFps;
    this.setText(this.elements.fpsValue, Math.round(this.smoothedFps).toString());

    if (++this.framesSinceSparklineSample >= SPARKLINE_SAMPLE_INTERVAL) {
      this.framesSinceSparklineSample = 0;
      this.fpsSparkline.update(this.smoothedFps);
    }
  }

  updateTickRate(tickRateHz: number): void {
    this.setText(
      this.elements.tickRateValue,
      tickRateHz < 0 ? "..." : `${tickRateHz.toFixed(1)} Hz`
    );
  }

  updateCreatureWorldCount(count: number): void {
    this.setText(this.elements.creatureWorldCount, count.toString());
  }

  updateCreatureScreenCount(count: number): void {
    this.setText(this.elements.creatureScreenCount, count.toString());
  }

  updatePlantWorldCount(count: number): void {
    this.setText(this.elements.plantWorldCount, count.toString());
  }

  updatePlantScreenCount(count: number): void {
    this.setText(this.elements.plantScreenCount, count.toString());
  }

  updateZoom(zoom: number): void {
    this.setText(this.elements.zoomValue, `${zoom.toFixed(2)}x`);
  }
}

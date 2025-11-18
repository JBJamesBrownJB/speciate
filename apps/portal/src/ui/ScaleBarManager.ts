import { SCALE_BAR_CONFIG } from "@/core/constants";

export class ScaleBarManager {
  private scaleLine: HTMLElement | null;
  private scaleLabel: HTMLElement | null;

  constructor(lineElementId: string, labelElementId: string) {
    this.scaleLine = document.getElementById(lineElementId);
    this.scaleLabel = document.getElementById(labelElementId);
  }

  update(zoom: number): void {
    const { distance, label } = this.selectScaleDistance(zoom);
    const pixelWidth = distance * zoom;

    if (this.scaleLine) {
      this.scaleLine.style.width = `${pixelWidth}px`;
    }

    if (this.scaleLabel) {
      this.scaleLabel.textContent = label;
    }
  }

  private selectScaleDistance(zoom: number): {
    distance: number;
    label: string;
  } {
    const idealDistance =
      SCALE_BAR_CONFIG.TARGET_PIXEL_WIDTH / zoom;

    let bestDistance = SCALE_BAR_CONFIG.NICE_INTERVALS[0];
    let bestDiff = Math.abs(idealDistance - bestDistance);

    for (const num of SCALE_BAR_CONFIG.NICE_INTERVALS) {
      const diff = Math.abs(idealDistance - num);
      if (diff < bestDiff) {
        bestDistance = num;
        bestDiff = diff;
      }
    }

    let label: string;
    if (bestDistance >= 1000) {
      label = `${bestDistance / 1000}km`;
    } else {
      label = `${bestDistance}m`;
    }

    return { distance: bestDistance, label };
  }
}

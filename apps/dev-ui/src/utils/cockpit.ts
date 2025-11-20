export const COLORS = {
  success: '#4ade80',
  warning: '#fbbf24',
  critical: '#f87171',
  streaming: '#06b6d4',
  fetch: '#60a5fa',
  dataWait: '#fb923c',
  neutral: '#9ca3af',
  white: '#fff',
} as const;

export const ANIMATION = {
  smoothing: 0.15,
  minAngleDiff: 0.001,
} as const;

export interface ColorZone {
  start: number;
  end: number;
  color: string;
  label: string;
}

export const getColorFromZones = (value: number, zones: ColorZone[]): string => {
  for (const zone of zones) {
    if (value >= zone.start && value < zone.end) {
      return zone.color;
    }
  }
  return zones[zones.length - 1].color;
};

export const getLabelFromZones = (value: number, zones: ColorZone[]): string => {
  for (const zone of zones) {
    if (value >= zone.start && value < zone.end) {
      return zone.label;
    }
  }
  return zones[zones.length - 1].label;
};

export const normalizeValue = (value: number, min: number, max: number): number => {
  return Math.max(0, Math.min(1, (value - min) / (max - min)));
};

export const mapToRange = (
  normalized: number,
  rangeStart: number,
  rangeEnd: number
): number => {
  return rangeStart + normalized * (rangeEnd - rangeStart);
};

export const useAnimatedValue = (
  targetValue: number,
  smoothing: number = ANIMATION.smoothing
): number => {
  const [current, setCurrent] = React.useState(targetValue);
  const [target, setTarget] = React.useState(targetValue);

  React.useEffect(() => {
    setTarget(targetValue);
  }, [targetValue]);

  React.useEffect(() => {
    const diff = target - current;
    if (Math.abs(diff) > ANIMATION.minAngleDiff) {
      const frame = requestAnimationFrame(() => {
        setCurrent((prev) => prev + diff * smoothing);
      });
      return () => cancelAnimationFrame(frame);
    }
  }, [current, target, smoothing]);

  return current;
};

import React from 'react';

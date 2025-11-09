import { describe, it, expect, beforeEach } from 'vitest';
import { Interpolator } from './Interpolator';
import { Creature } from './Creature';

describe('Interpolator', () => {
  let interpolator: Interpolator;

  beforeEach(() => {
    interpolator = new Interpolator();
  });

  describe('update', () => {
    it('should store creatures for interpolation', () => {
      const creatures = [
        new Creature(1, 100, 200, 0, 1, 1),
        new Creature(2, 300, 400, 0, 1, 1)
      ];

      interpolator.update(creatures, performance.now());

      const interpolated = interpolator.interpolate(performance.now());
      expect(interpolated).toHaveLength(2);
      expect(interpolated[0].id).toBe(1);
      expect(interpolated[1].id).toBe(2);
    });

    it('should track old and new positions for each creature', () => {
      // Provide enough updates for stable adaptive buffer (need 4 updates = 3 intervals)
      interpolator.update([new Creature(1, 75, 175, 0, 1, 1)], 950);
      interpolator.update([new Creature(1, 100, 200, 0, 1, 1)], 1000);
      interpolator.update([new Creature(1, 125, 225, 0, 1, 1)], 1050);
      interpolator.update([new Creature(1, 150, 250, Math.PI / 2, 1, 1)], 1100);

      // After 4 updates at 50ms intervals (3 intervals), buffer should be ~75ms
      // When rendering at 1175, we render at 1175 - 75 = 1100
      // This is exactly at the last snapshot
      const interpolated = interpolator.interpolate(1175);

      // Should be at the latest snapshot position
      expect(interpolated[0].x).toBe(150);
      expect(interpolated[0].y).toBe(250);
    });

    it('should handle new creatures appearing', () => {
      const creatures1 = [new Creature(1, 100, 200, 0, 1, 1)];
      interpolator.update(creatures1, 1000);

      const creatures2 = [
        new Creature(1, 150, 250, 0, 1, 1),
        new Creature(2, 300, 400, 0, 1, 1) // New creature
      ];
      interpolator.update(creatures2, 1050);

      const interpolated = interpolator.interpolate(1050);
      expect(interpolated).toHaveLength(2);
      expect(interpolated.find(c => c.id === 2)).toBeDefined();
    });

    it('should handle creatures disappearing', () => {
      const creatures1 = [
        new Creature(1, 100, 200, 0, 1, 1),
        new Creature(2, 300, 400, 0, 1, 1)
      ];
      interpolator.update(creatures1, 1000);

      const creatures2 = [
        new Creature(1, 150, 250, 0, 1, 1)
        // Creature 2 is gone
      ];
      interpolator.update(creatures2, 1050);

      const interpolated = interpolator.interpolate(1050);
      expect(interpolated).toHaveLength(1);
      expect(interpolated.find(c => c.id === 2)).toBeUndefined();
    });
  });

  describe('interpolate', () => {
    it('should return current positions when at latest update time', () => {
      const creatures = [new Creature(1, 100, 200, 0, 1, 1)];
      const updateTime = 1000;

      interpolator.update(creatures, updateTime);

      const interpolated = interpolator.interpolate(updateTime);
      expect(interpolated[0].x).toBe(100);
      expect(interpolated[0].y).toBe(200);
    });

    it('should interpolate between old and new positions', () => {
      // Provide enough updates to establish stable adaptive buffer (4 updates = 3 intervals)
      interpolator.update([new Creature(1, 0, 100, 0, 1, 1)], 900);
      interpolator.update([new Creature(1, 100, 200, 0, 1, 1)], 950);
      interpolator.update([new Creature(1, 200, 300, Math.PI, 1, 1)], 1000);
      interpolator.update([new Creature(1, 300, 400, Math.PI, 1, 1)], 1050);

      // After 4 updates at 50ms intervals (3 intervals), buffer = 75ms
      // When rendering at 1125, we render at 1125 - 75 = 1050
      // This is exactly at the last snapshot
      const interpolated = interpolator.interpolate(1125);

      // Should be at the latest position
      expect(interpolated[0].x).toBe(300);
      expect(interpolated[0].y).toBe(400);
      expect(interpolated[0].rotation).toBe(Math.PI);
    });

    it('should clamp interpolation alpha to [0, 1]', () => {
      const creatures1 = [new Creature(1, 100, 200, 0, 1, 1)];
      interpolator.update(creatures1, 1000);

      const creatures2 = [new Creature(1, 200, 300, 0, 1, 1)];
      interpolator.update(creatures2, 1050);

      // Try to interpolate in the past (before first update)
      const past = interpolator.interpolate(900);
      expect(past[0].x).toBe(100);
      expect(past[0].y).toBe(200);

      // Try to interpolate in the future (after second update)
      const future = interpolator.interpolate(2000);
      expect(future[0].x).toBe(200);
      expect(future[0].y).toBe(300);
    });

    it('should handle rotation interpolation across 2π boundary', () => {
      // Start at 0 radians
      const creatures1 = [new Creature(1, 100, 100, 0, 1, 1)];
      interpolator.update(creatures1, 1000);

      // End at 2π - 0.1 (just before full rotation)
      const creatures2 = [new Creature(1, 100, 100, 2 * Math.PI - 0.1, 1, 1)];
      interpolator.update(creatures2, 1050);

      // Interpolate at midpoint
      const interpolated = interpolator.interpolate(1025);

      // Should take shortest path (clockwise near 0, not counterclockwise all the way around)
      // This is approximately -0.05 radians (or 2π - 0.05)
      expect(Math.abs(interpolated[0].rotation)).toBeLessThan(0.2);
    });

    it('should use latest position for new creatures', () => {
      // First update with creature 1
      const creatures1 = [new Creature(1, 100, 200, 0, 1, 1)];
      interpolator.update(creatures1, 1000);

      // Second update adds creature 2
      const creatures2 = [
        new Creature(1, 150, 250, 0, 1, 1),
        new Creature(2, 300, 400, 0, 1, 1) // Just appeared
      ];
      interpolator.update(creatures2, 1050);

      // Interpolate at midpoint
      const interpolated = interpolator.interpolate(1025);

      // Creature 2 should be at its latest position (no interpolation)
      const creature2 = interpolated.find(c => c.id === 2);
      expect(creature2?.x).toBe(300);
      expect(creature2?.y).toBe(400);
    });
  });

  describe('clear', () => {
    it('should remove all creatures', () => {
      const creatures = [
        new Creature(1, 100, 200, 0, 1, 1),
        new Creature(2, 300, 400, 0, 1, 1)
      ];
      interpolator.update(creatures, 1000);

      interpolator.clear();

      const interpolated = interpolator.interpolate(1000);
      expect(interpolated).toHaveLength(0);
    });
  });

  describe('getCreatureCount', () => {
    it('should return number of tracked creatures', () => {
      expect(interpolator.getCreatureCount()).toBe(0);

      const creatures = [
        new Creature(1, 100, 200, 0, 1, 1),
        new Creature(2, 300, 400, 0, 1, 1)
      ];
      interpolator.update(creatures, 1000);

      expect(interpolator.getCreatureCount()).toBe(2);
    });
  });

  describe('velocity-aware extrapolation', () => {
    it('should extrapolate with damping when past latest snapshot', () => {
      // First update: creature at (0, 0) moving at 10 m/s in X direction
      const creatures1 = [new Creature(1, 0, 0, 0, 1, 1, 10, 0)];
      interpolator.update(creatures1, 1000);

      // Second update: creature at (0.5, 0) - moved 0.5m in 50ms (10 m/s)
      const creatures2 = [new Creature(1, 0.5, 0, 0, 1, 1, 10, 0)];
      interpolator.update(creatures2, 1050);

      // Extrapolate 100ms after second update (should use damping)
      const interpolated = interpolator.interpolate(1150);

      // Without damping: 0.5 + (10 * 0.1) = 1.5m
      // With damping (0.1): 0.5 + (10 * 0.1 * exp(-0.1 * 0.1)) ≈ 0.5 + 0.99 ≈ 1.49m
      expect(interpolated[0].x).toBeGreaterThan(0.5);
      expect(interpolated[0].x).toBeLessThan(1.5); // Damped
    });

    it('should interpolate normally when between snapshots', () => {
      // Provide enough updates for stable buffer (4 updates = 3 intervals)
      interpolator.update([new Creature(1, 0, 0, 0, 1, 1, 10, 0)], 800);
      interpolator.update([new Creature(1, 10, 0, 0, 1, 1, 10, 0)], 900);
      interpolator.update([new Creature(1, 20, 0, 0, 1, 1, 10, 0)], 1000);
      interpolator.update([new Creature(1, 30, 0, 0, 1, 1, 10, 0)], 1100);

      // After 4 updates at 100ms intervals (3 intervals), buffer = 100ms
      // When rendering at 1200, we render at 1200 - 100 = 1100
      // This is exactly at the last snapshot
      const interpolated = interpolator.interpolate(1200);

      // Should be at last snapshot (standard interpolation, not extrapolation)
      expect(interpolated[0].x).toBe(30);
    });

    it('should handle missing velocity gracefully', () => {
      // Creature without velocity data
      const creatures1 = [new Creature(1, 0, 0, 0, 1, 1)]; // No vx, vy
      interpolator.update(creatures1, 1000);

      const creatures2 = [new Creature(1, 10, 0, 0, 1, 1)]; // No vx, vy
      interpolator.update(creatures2, 1050);

      // Extrapolate past latest (should just use latest position)
      const interpolated = interpolator.interpolate(1150);

      // Without velocity, should clamp to latest position
      expect(interpolated[0].x).toBe(10);
      expect(interpolated[0].y).toBe(0);
    });

    it('should use exponential damping for realistic deceleration', () => {
      // Creature at (0, 0) with high velocity
      const creatures1 = [new Creature(1, 0, 0, 0, 1, 1, 30, 0)]; // 30 m/s
      interpolator.update(creatures1, 1000);

      const creatures2 = [new Creature(1, 1.5, 0, 0, 1, 1, 30, 0)]; // Moved 1.5m in 50ms
      interpolator.update(creatures2, 1050);

      // Extrapolate 500ms later (long time)
      const interpolated = interpolator.interpolate(1550);

      // With damping (0.1), velocity should decay
      // Distance without damping: 1.5 + (30 * 0.5) = 16.5m
      // Distance with damping (0.1): 1.5 + (30 * 0.5 * exp(-0.1 * 0.5)) ≈ 15.77m
      expect(interpolated[0].x).toBeGreaterThan(1.5); // Moved forward
      expect(interpolated[0].x).toBeLessThan(16.5); // Less than without damping
    });
  });

  describe('adaptive buffer', () => {
    it('should use fallback buffer when insufficient data', () => {
      const interpolator = new Interpolator();

      // No updates yet - should use fallback (50ms from constants)
      expect(interpolator.getBufferMs()).toBe(50);

      // One update - still not enough
      const creatures1 = [new Creature(1, 0, 0, 0, 1, 1)];
      interpolator.update(creatures1, 1000);
      expect(interpolator.getBufferMs()).toBe(50);

      // Two updates - still not enough (need 3 intervals)
      const creatures2 = [new Creature(1, 10, 0, 0, 1, 1)];
      interpolator.update(creatures2, 1050);
      expect(interpolator.getBufferMs()).toBe(50);
    });

    it('should adapt to 20 Hz updates (50ms intervals)', () => {
      const interpolator = new Interpolator();

      // Simulate 10 updates at 50ms intervals (20 Hz)
      for (let i = 0; i < 10; i++) {
        const creatures = [new Creature(1, i * 5, 0, 0, 1, 1)];
        interpolator.update(creatures, i * 50);
      }

      // Should calculate ~50ms buffer (1.0 × 50ms)
      expect(interpolator.getBufferMs()).toBeCloseTo(50, 0);
    });

    it('should adapt to 60 Hz updates (16.67ms intervals)', () => {
      const interpolator = new Interpolator();

      // Simulate 10 updates at 16.67ms intervals (60 Hz)
      for (let i = 0; i < 10; i++) {
        const creatures = [new Creature(1, i * 2, 0, 0, 1, 1)];
        interpolator.update(creatures, i * 16.67);
      }

      // Should calculate ~16.67ms buffer (1.0 × 16.67ms), clamped to minimum 20ms
      expect(interpolator.getBufferMs()).toBe(20); // Clamped to min
    });

    it('should adapt to 10 Hz updates (100ms intervals)', () => {
      const interpolator = new Interpolator();

      // Simulate 10 updates at 100ms intervals (10 Hz)
      for (let i = 0; i < 10; i++) {
        const creatures = [new Creature(1, i * 10, 0, 0, 1, 1)];
        interpolator.update(creatures, i * 100);
      }

      // Should calculate ~100ms buffer (1.0 × 100ms)
      expect(interpolator.getBufferMs()).toBeCloseTo(100, 0);
    });

    it('should handle jittery updates using median', () => {
      const interpolator = new Interpolator();

      // Simulate jittery updates: intervals vary between 40-60ms
      const intervals = [40, 60, 50, 45, 55, 50, 50, 50, 50, 50];
      let time = 0;
      for (const interval of intervals) {
        time += interval;
        const creatures = [new Creature(1, time / 10, 0, 0, 1, 1)];
        interpolator.update(creatures, time);
      }

      // Median should be 50ms, buffer should be ~50ms
      expect(interpolator.getBufferMs()).toBeCloseTo(50, 0);
    });

    it('should clamp buffer to minimum 20ms', () => {
      const interpolator = new Interpolator();

      // Simulate extremely fast updates (1ms intervals)
      for (let i = 0; i < 10; i++) {
        const creatures = [new Creature(1, i * 0.1, 0, 0, 1, 1)];
        interpolator.update(creatures, i);
      }

      // Should clamp to 20ms minimum
      expect(interpolator.getBufferMs()).toBe(20);
    });

    it('should clamp buffer to maximum 200ms', () => {
      const interpolator = new Interpolator();

      // Simulate very slow updates (200ms intervals)
      for (let i = 0; i < 10; i++) {
        const creatures = [new Creature(1, i * 20, 0, 0, 1, 1)];
        interpolator.update(creatures, i * 200);
      }

      // Should clamp to 200ms maximum (not 300ms = 1.0 × 200 with extended clamp)
      expect(interpolator.getBufferMs()).toBe(200);
    });

    it('should use adaptive buffer in interpolation', () => {
      const interpolator = new Interpolator();

      // Set up 20 Hz updates (50ms intervals)
      interpolator.update([new Creature(1, 0, 0, 0, 1, 1)], 1000);
      interpolator.update([new Creature(1, 5, 0, 0, 1, 1)], 1050);
      interpolator.update([new Creature(1, 10, 0, 0, 1, 1)], 1100);
      interpolator.update([new Creature(1, 15, 0, 0, 1, 1)], 1150);

      // Buffer should be ~50ms (1.0 × 50ms)
      // When rendering at time 1200, we should render at 1200 - 50 = 1150
      // This is exactly at the last snapshot
      const interpolated = interpolator.interpolate(1200);

      // Expected position: at snapshot time 1150 (x = 15)
      expect(interpolated[0].x).toBe(15);
    });
  });
});

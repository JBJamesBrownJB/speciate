import { describe, it, expect } from "vitest";
import { CameraSmoother } from "./CameraSmoother";

describe("CameraSmoother", () => {
  it("starts at the constructed pose", () => {
    const s = new CameraSmoother(10, 20, 2);
    expect(s.x).toBe(10);
    expect(s.y).toBe(20);
    expect(s.zoom).toBe(2);
  });

  it("eases toward the target (monotonic, never overshoots)", () => {
    const s = new CameraSmoother(0, 0, 1, 0.045);
    let prev = s.x;
    for (let i = 0; i < 5; i++) {
      s.follow(100, 0, 1, 0.016);
      expect(s.x).toBeGreaterThan(prev); // moving toward target
      expect(s.x).toBeLessThan(100); // never past it
      prev = s.x;
    }
  });

  it("converges to the target after enough time", () => {
    const s = new CameraSmoother(0, 0, 1);
    for (let i = 0; i < 100; i++) s.follow(50, -30, 4, 0.016);
    expect(s.x).toBeCloseTo(50, 3);
    expect(s.y).toBeCloseTo(-30, 3);
    expect(s.zoom).toBeCloseTo(4, 3);
  });

  it("is frame-rate independent: one big step == many small steps", () => {
    const big = new CameraSmoother(0, 0, 1);
    const small = new CameraSmoother(0, 0, 1);
    big.follow(100, 0, 1, 0.1);
    for (let i = 0; i < 10; i++) small.follow(100, 0, 1, 0.01);
    // Exact continuous solution → identical regardless of subdivision (to float precision).
    expect(small.x).toBeCloseTo(big.x, 9);
  });

  it("dt <= 0 is a no-op", () => {
    const s = new CameraSmoother(5, 5, 1);
    s.follow(100, 100, 9, 0);
    s.follow(100, 100, 9, -0.5);
    expect(s.x).toBe(5);
    expect(s.y).toBe(5);
    expect(s.zoom).toBe(1);
  });

  it("already at target stays put", () => {
    const s = new CameraSmoother(7, 7, 3);
    s.follow(7, 7, 3, 0.016);
    expect(s.x).toBe(7);
    expect(s.y).toBe(7);
    expect(s.zoom).toBe(3);
  });

  it("snap jumps instantly with no easing", () => {
    const s = new CameraSmoother(0, 0, 1);
    s.snap(123, -45, 6);
    expect(s.x).toBe(123);
    expect(s.y).toBe(-45);
    expect(s.zoom).toBe(6);
  });
});

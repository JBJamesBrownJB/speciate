import { describe, it, expect, beforeEach } from "vitest";
import { ChangeDetector } from "./ChangeDetection";
import { FLOATS_PER_CREATURE, getBufferOffsets } from "@/types/BufferLayout";

interface Spec {
  id: number;
  x: number;
  y: number;
  rotation?: number;
  size?: number;
}

/** Build the wire-format SoA buffer for a set of creatures. */
function soa(specs: Spec[]): { buffer: Float32Array; count: number } {
  const n = specs.length;
  const buffer = new Float32Array(n * FLOATS_PER_CREATURE);
  const o = getBufferOffsets(n);
  specs.forEach((c, i) => {
    buffer[o.id + i] = c.id;
    buffer[o.x + i] = c.x;
    buffer[o.y + i] = c.y;
    buffer[o.rot + i] = c.rotation ?? 0;
    buffer[o.size + i] = c.size ?? 1;
  });
  return { buffer, count: n };
}

function check(detector: ChangeDetector, tick: number, specs: Spec[]): boolean {
  const { buffer, count } = soa(specs);
  return detector.shouldUpdate(tick, buffer, count);
}

describe("ChangeDetector", () => {
  let detector: ChangeDetector;

  beforeEach(() => {
    detector = new ChangeDetector();
  });

  describe("push-on-swap path (tick > 0): tick identity is the change signal", () => {
    const one = [{ id: 1, x: 0, y: 0 }];

    it("first delivery always updates", () => {
      expect(check(detector, 1, one)).toBe(true);
    });

    it("a new tick is new data — even if positions look identical", () => {
      expect(check(detector, 1, one)).toBe(true);
      expect(check(detector, 2, one)).toBe(true);
    });

    it("the same tick delivered twice is a duplicate", () => {
      expect(check(detector, 5, one)).toBe(true);
      expect(check(detector, 5, one)).toBe(false);
    });
  });

  describe("poll-fallback path (tick 0): exact comparison, no sampling", () => {
    it("updates when creatures are spawned (count increases)", () => {
      expect(check(detector, 0, [])).toBe(true); // first call
      expect(check(detector, 0, [{ id: 1, x: 0, y: 0 }])).toBe(true);
    });

    it("updates when creatures are despawned (count decreases)", () => {
      const two = [{ id: 1, x: 0, y: 0 }, { id: 2, x: 10, y: 10 }];
      expect(check(detector, 0, two)).toBe(true);
      expect(check(detector, 0, two)).toBe(false);
      expect(check(detector, 0, [two[0]])).toBe(true);
    });

    it("updates when ONLY a middle creature moves (the old sampled hash missed this)", () => {
      // Catatonic creatures at the edges, one mover in the middle — the exact
      // failure mode of the first-3/last-3 sample: a visibly frozen mover.
      const initial = Array.from({ length: 100 }, (_, i) => ({ id: i, x: i * 2, y: i * 2 }));
      const afterMove = initial.map((c, i) => (i === 50 ? { id: 50, x: 1000, y: 100 } : c));

      expect(check(detector, 0, initial)).toBe(true);
      expect(check(detector, 0, afterMove)).toBe(true);
    });

    it("does NOT update when nothing moved (catatonic crowd)", () => {
      const catatonics = [{ id: 1, x: 10, y: 10 }, { id: 2, x: 20, y: 20 }];
      expect(check(detector, 0, catatonics)).toBe(true);
      expect(check(detector, 0, catatonics)).toBe(false);
      expect(check(detector, 0, catatonics)).toBe(false);
    });

    it("detects rotation-only changes (spinning in place)", () => {
      expect(check(detector, 0, [{ id: 1, x: 5, y: 5, rotation: 0 }])).toBe(true);
      expect(check(detector, 0, [{ id: 1, x: 5, y: 5, rotation: 1.5 }])).toBe(true);
    });

    it("detects id changes at identical positions (death + respawn)", () => {
      expect(check(detector, 0, [{ id: 1, x: 5, y: 5 }])).toBe(true);
      expect(check(detector, 0, [{ id: 2, x: 5, y: 5 }])).toBe(true);
    });

    it("detects even sub-pixel moves — exact means exact", () => {
      expect(check(detector, 0, [{ id: 1, x: 0, y: 0 }])).toBe(true);
      expect(check(detector, 0, [{ id: 1, x: 0.001, y: 0.001 }])).toBe(true);
    });

    it("handles empty buffers", () => {
      expect(check(detector, 0, [])).toBe(true);
      expect(check(detector, 0, [])).toBe(false);
    });

    it("ignores stale floats past `count` in an oversized buffer", () => {
      // Real deliveries reuse a large backing buffer; only [0, count*5) is live.
      const { buffer } = soa([{ id: 1, x: 0, y: 0 }, { id: 99, x: 123, y: 456 }]);
      expect(detector.shouldUpdate(0, buffer, 1)).toBe(true);
      buffer[7] = 777; // mutate a float beyond creature 0's slice...
      // careful: SoA layout interleaves columns by count, so with count=1 only
      // indices {0..4} are live; index 7 is stale garbage.
      expect(detector.shouldUpdate(0, buffer, 1)).toBe(false);
    });
  });

  describe("reset", () => {
    it("resets both paths", () => {
      const one = [{ id: 1, x: 0, y: 0 }];
      expect(check(detector, 3, one)).toBe(true);
      detector.reset();
      expect(check(detector, 3, one)).toBe(true);

      expect(check(detector, 0, one)).toBe(true);
      expect(check(detector, 0, one)).toBe(false);
      detector.reset();
      expect(check(detector, 0, one)).toBe(true);
    });
  });
});

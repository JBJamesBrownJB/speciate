/**
 * Performance Metrics Tracker
 *
 * Tracks timing data for the entire IPC chain:
 * - Electron IPC latency
 * - MessagePack deserialization time
 * - PixiJS sprite update time
 * - DOM update time
 * - Overall frame time
 *
 * Maintains rolling averages for smooth metrics display.
 */

export interface PerformanceSnapshot {
  // IPC metrics
  ipcLatency: number;
  ipcLatencyAvg: number;
  ipcLatencyMin: number;
  ipcLatencyMax: number;

  // Deserialization metrics
  decodeTime: number;
  decodeTimeAvg: number;
  payloadSize: number; // bytes

  // Rendering metrics
  spriteUpdateTime: number;
  domUpdateTime: number;
  totalRenderTime: number;

  // Frame metrics
  frameTime: number; // Actual time taken for this frame
  frameBudget: number; // Target frame time (e.g., 11.1ms for 90 FPS)
  frameOverhead: number; // frameTime - frameBudget (negative = good!)

  // Freshness metrics
  backendTick: number; // Tick number from backend
  tickAge: number; // How many frames we've been showing this tick
}

export class PerformanceMetrics {
  private readonly windowSize = 30; // Rolling window size (samples)

  // IPC latency tracking
  private ipcLatencies: number[] = [];
  private ipcLatencyMin = Infinity;
  private ipcLatencyMax = 0;

  // Decode time tracking
  private decodeTimes: number[] = [];

  // Current measurements
  private currentIpcLatency = 0;
  private currentDecodeTime = 0;
  private currentPayloadSize = 0;
  private currentSpriteUpdateTime = 0;
  private currentDomUpdateTime = 0;
  private currentTotalRenderTime = 0;

  // Frame tracking
  private currentFrameTime = 0;
  private frameBudget: number;

  // Tick freshness tracking
  private lastSeenTick = 0;
  private tickAge = 0;

  constructor(targetFPS: number) {
    this.frameBudget = 1000 / targetFPS; // e.g., 90 FPS = 11.1ms
  }

  /**
   * Record IPC call latency
   */
  recordIpcLatency(latencyMs: number): void {
    this.currentIpcLatency = latencyMs;
    this.ipcLatencies.push(latencyMs);

    // Update min/max
    this.ipcLatencyMin = Math.min(this.ipcLatencyMin, latencyMs);
    this.ipcLatencyMax = Math.max(this.ipcLatencyMax, latencyMs);

    // Trim window
    if (this.ipcLatencies.length > this.windowSize) {
      this.ipcLatencies.shift();
    }
  }

  /**
   * Record MessagePack decode time and payload size
   */
  recordDecodeTime(decodeMs: number, payloadBytes: number): void {
    this.currentDecodeTime = decodeMs;
    this.currentPayloadSize = payloadBytes;
    this.decodeTimes.push(decodeMs);

    // Trim window
    if (this.decodeTimes.length > this.windowSize) {
      this.decodeTimes.shift();
    }
  }

  /**
   * Record sprite update time
   */
  recordSpriteUpdateTime(timeMs: number): void {
    this.currentSpriteUpdateTime = timeMs;
  }

  /**
   * Record DOM update time
   */
  recordDomUpdateTime(timeMs: number): void {
    this.currentDomUpdateTime = timeMs;
  }

  /**
   * Record total render time (sprite + DOM + other)
   */
  recordTotalRenderTime(timeMs: number): void {
    this.currentTotalRenderTime = timeMs;
  }

  /**
   * Record frame time
   */
  recordFrameTime(frameTimeMs: number): void {
    this.currentFrameTime = frameTimeMs;
  }

  /**
   * Update backend tick freshness
   */
  updateBackendTick(tick: number): void {
    if (tick === this.lastSeenTick) {
      this.tickAge++;
    } else {
      this.lastSeenTick = tick;
      this.tickAge = 0;
    }
  }

  /**
   * Get current performance snapshot
   */
  getSnapshot(): PerformanceSnapshot {
    return {
      // IPC metrics
      ipcLatency: this.currentIpcLatency,
      ipcLatencyAvg: this.average(this.ipcLatencies),
      ipcLatencyMin: this.ipcLatencyMin === Infinity ? 0 : this.ipcLatencyMin,
      ipcLatencyMax: this.ipcLatencyMax,

      // Decode metrics
      decodeTime: this.currentDecodeTime,
      decodeTimeAvg: this.average(this.decodeTimes),
      payloadSize: this.currentPayloadSize,

      // Render metrics
      spriteUpdateTime: this.currentSpriteUpdateTime,
      domUpdateTime: this.currentDomUpdateTime,
      totalRenderTime: this.currentTotalRenderTime,

      // Frame metrics
      frameTime: this.currentFrameTime,
      frameBudget: this.frameBudget,
      frameOverhead: this.currentFrameTime - this.frameBudget,

      // Freshness metrics
      backendTick: this.lastSeenTick,
      tickAge: this.tickAge,
    };
  }

  /**
   * Get detailed performance log string
   */
  getDetailedLog(frameNumber: number, fps: number): string {
    const snapshot = this.getSnapshot();

    const lines = [
      `[PERF] Frame ${frameNumber}:`,
      `  IPC call:    ${snapshot.ipcLatency.toFixed(2)}ms (avg: ${snapshot.ipcLatencyAvg.toFixed(2)}ms, min: ${snapshot.ipcLatencyMin.toFixed(2)}ms, max: ${snapshot.ipcLatencyMax.toFixed(2)}ms)`,
      `  Decode:      ${snapshot.decodeTime.toFixed(2)}ms (avg: ${snapshot.decodeTimeAvg.toFixed(2)}ms, payload: ${(snapshot.payloadSize / 1024).toFixed(2)}KB)`,
      `  Sprite upd:  ${snapshot.spriteUpdateTime.toFixed(2)}ms`,
      `  DOM upd:     ${snapshot.domUpdateTime.toFixed(2)}ms`,
      `  Render:      ${snapshot.totalRenderTime.toFixed(2)}ms`,
      `  Frame time:  ${snapshot.frameTime.toFixed(2)}ms (budget: ${snapshot.frameBudget.toFixed(2)}ms, overhead: ${snapshot.frameOverhead >= 0 ? '+' : ''}${snapshot.frameOverhead.toFixed(2)}ms)`,
      `  FPS:         ${fps} (target: ${Math.round(1000 / this.frameBudget)})`,
      `  Tick:        #${snapshot.backendTick} (stale for ${snapshot.tickAge} frames)`,
    ];

    return lines.join('\n');
  }

  /**
   * Calculate average of array
   */
  private average(values: number[]): number {
    if (values.length === 0) return 0;
    return values.reduce((sum, val) => sum + val, 0) / values.length;
  }

  /**
   * Reset all metrics (useful for testing)
   */
  reset(): void {
    this.ipcLatencies = [];
    this.ipcLatencyMin = Infinity;
    this.ipcLatencyMax = 0;
    this.decodeTimes = [];
    this.currentIpcLatency = 0;
    this.currentDecodeTime = 0;
    this.currentPayloadSize = 0;
    this.currentSpriteUpdateTime = 0;
    this.currentDomUpdateTime = 0;
    this.currentTotalRenderTime = 0;
    this.currentFrameTime = 0;
    this.lastSeenTick = 0;
    this.tickAge = 0;
  }
}

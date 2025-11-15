export interface PerformanceSnapshot {
  ipcLatency: number;
  ipcLatencyAvg: number;
  ipcLatencyMin: number;
  ipcLatencyMax: number;

  decodeTime: number;
  decodeTimeAvg: number;
  payloadSize: number;

  spriteUpdateTime: number;
  domUpdateTime: number;
  totalRenderTime: number;

  frameTime: number;
  frameBudget: number;
  frameOverhead: number;

  backendTick: number;
  tickAge: number;
}

export class PerformanceMetrics {
  private readonly windowSize = 30;

  private ipcLatencies: number[] = [];
  private ipcLatencyMin = Infinity;
  private ipcLatencyMax = 0;

  private decodeTimes: number[] = [];

  private currentIpcLatency = 0;
  private currentDecodeTime = 0;
  private currentPayloadSize = 0;
  private currentSpriteUpdateTime = 0;
  private currentDomUpdateTime = 0;
  private currentTotalRenderTime = 0;

  private currentFrameTime = 0;
  private frameBudget: number;

  private lastSeenTick = 0;
  private tickAge = 0;

  constructor(targetFPS: number) {
    this.frameBudget = 1000 / targetFPS;
  }

  recordIpcLatency(latencyMs: number): void {
    this.currentIpcLatency = latencyMs;
    this.ipcLatencies.push(latencyMs);

    this.ipcLatencyMin = Math.min(this.ipcLatencyMin, latencyMs);
    this.ipcLatencyMax = Math.max(this.ipcLatencyMax, latencyMs);

    if (this.ipcLatencies.length > this.windowSize) {
      this.ipcLatencies.shift();
    }
  }

  recordDecodeTime(decodeMs: number, payloadBytes: number): void {
    this.currentDecodeTime = decodeMs;
    this.currentPayloadSize = payloadBytes;
    this.decodeTimes.push(decodeMs);

    if (this.decodeTimes.length > this.windowSize) {
      this.decodeTimes.shift();
    }
  }

  recordSpriteUpdateTime(timeMs: number): void {
    this.currentSpriteUpdateTime = timeMs;
  }

  recordDomUpdateTime(timeMs: number): void {
    this.currentDomUpdateTime = timeMs;
  }

  recordTotalRenderTime(timeMs: number): void {
    this.currentTotalRenderTime = timeMs;
  }

  recordFrameTime(frameTimeMs: number): void {
    this.currentFrameTime = frameTimeMs;
  }

  updateBackendTick(tick: number): void {
    if (tick === this.lastSeenTick) {
      this.tickAge++;
    } else {
      this.lastSeenTick = tick;
      this.tickAge = 0;
    }
  }

  getSnapshot(): PerformanceSnapshot {
    return {
      ipcLatency: this.currentIpcLatency,
      ipcLatencyAvg: this.average(this.ipcLatencies),
      ipcLatencyMin: this.ipcLatencyMin === Infinity ? 0 : this.ipcLatencyMin,
      ipcLatencyMax: this.ipcLatencyMax,

      decodeTime: this.currentDecodeTime,
      decodeTimeAvg: this.average(this.decodeTimes),
      payloadSize: this.currentPayloadSize,

      spriteUpdateTime: this.currentSpriteUpdateTime,
      domUpdateTime: this.currentDomUpdateTime,
      totalRenderTime: this.currentTotalRenderTime,

      frameTime: this.currentFrameTime,
      frameBudget: this.frameBudget,
      frameOverhead: this.currentFrameTime - this.frameBudget,

      backendTick: this.lastSeenTick,
      tickAge: this.tickAge,
    };
  }

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

  private average(values: number[]): number {
    if (values.length === 0) return 0;
    return values.reduce((sum, val) => sum + val, 0) / values.length;
  }

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

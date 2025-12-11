/**
 * Memory Profiling Type Extensions
 *
 * Add these types to types.ts when using memory profiling mode
 */

export interface MemorySnapshot {
  timestamp: number;
  rss: number;
  heapTotal: number;
  heapUsed: number;
  external: number;
  arrayBuffers: number;
}

export interface HeapSnapshotResult {
  success: boolean;
  path?: string;
  error?: string;
}

declare global {
  interface Window {
    electron?: {
      // Memory profiling extensions
      onMemoryUpdate?: (callback: (snapshot: MemorySnapshot) => void) => void;
      removeMemoryUpdateListener?: (callback?: (snapshot: MemorySnapshot) => void) => void;
      triggerGC?: () => void;
      takeHeapSnapshot?: () => Promise<HeapSnapshotResult>;
    };
  }
}

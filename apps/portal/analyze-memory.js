#!/usr/bin/env node

/**
 * Memory Profile Analyzer
 *
 * Parses memory-profile.jsonl and prints statistics
 *
 * Usage: node analyze-memory.js
 */

const fs = require('fs');
const path = require('path');

const MEMORY_LOG_FILE = path.join(__dirname, '../../docs/performance/memory-profile.jsonl');

if (!fs.existsSync(MEMORY_LOG_FILE)) {
  console.error('Memory log file not found:', MEMORY_LOG_FILE);
  console.error('Run the app with memory profiling first:');
  console.error('  ./memory-profile.sh');
  process.exit(1);
}

const formatBytes = (bytes) => {
  return (bytes / 1024 / 1024).toFixed(2) + ' MB';
};

const formatDelta = (bytes) => {
  const sign = bytes >= 0 ? '+' : '';
  return sign + formatBytes(bytes);
};

const lines = fs.readFileSync(MEMORY_LOG_FILE, 'utf8')
  .split('\n')
  .filter(line => line.trim() !== '')
  .map(line => JSON.parse(line));

if (lines.length === 0) {
  console.error('Memory log file is empty');
  process.exit(1);
}

const first = lines[0];
const last = lines[lines.length - 1];
const duration = (last.timestamp - first.timestamp) / 1000;

console.log('=== MEMORY PROFILE ANALYSIS ===\n');
console.log(`Duration: ${duration.toFixed(1)}s (${lines.length} samples)\n`);

console.log('BASELINE (start):');
console.log(`  RSS:          ${formatBytes(first.rss)}`);
console.log(`  Heap Total:   ${formatBytes(first.heapTotal)}`);
console.log(`  Heap Used:    ${formatBytes(first.heapUsed)}`);
console.log(`  External:     ${formatBytes(first.external)}`);
console.log(`  ArrayBuffers: ${formatBytes(first.arrayBuffers)}\n`);

console.log('CURRENT (end):');
console.log(`  RSS:          ${formatBytes(last.rss)}`);
console.log(`  Heap Total:   ${formatBytes(last.heapTotal)}`);
console.log(`  Heap Used:    ${formatBytes(last.heapUsed)}`);
console.log(`  External:     ${formatBytes(last.external)}`);
console.log(`  ArrayBuffers: ${formatBytes(last.arrayBuffers)}\n`);

console.log('DELTA (growth):');
console.log(`  RSS:          ${formatDelta(last.rss - first.rss)}`);
console.log(`  Heap Total:   ${formatDelta(last.heapTotal - first.heapTotal)}`);
console.log(`  Heap Used:    ${formatDelta(last.heapUsed - first.heapUsed)}`);
console.log(`  External:     ${formatDelta(last.external - first.external)}`);
console.log(`  ArrayBuffers: ${formatDelta(last.arrayBuffers - first.arrayBuffers)}\n`);

console.log('GROWTH RATE (per second):');
console.log(`  RSS:          ${formatDelta((last.rss - first.rss) / duration)}/s`);
console.log(`  Heap Total:   ${formatDelta((last.heapTotal - first.heapTotal) / duration)}/s`);
console.log(`  Heap Used:    ${formatDelta((last.heapUsed - first.heapUsed) / duration)}/s`);
console.log(`  External:     ${formatDelta((last.external - first.external) / duration)}/s`);
console.log(`  ArrayBuffers: ${formatDelta((last.arrayBuffers - first.arrayBuffers) / duration)}/s\n`);

const heapGrowth = (last.heapUsed - first.heapUsed) / duration;
const externalGrowth = (last.external - first.external) / duration;
const arrayBufferGrowth = (last.arrayBuffers - first.arrayBuffers) / duration;

console.log('=== DIAGNOSIS ===\n');

if (heapGrowth > 0.5 * 1024 * 1024) {
  console.log('V8 Heap Leak Detected:');
  console.log(`  Heap growing at ${formatDelta(heapGrowth)}/s`);
  console.log('  Likely cause: JavaScript objects not being garbage collected');
  console.log('  Action: Take heap snapshot and analyze with Chrome DevTools\n');
}

if (externalGrowth > 0.5 * 1024 * 1024) {
  console.log('External Memory Leak Detected:');
  console.log(`  External memory growing at ${formatDelta(externalGrowth)}/s`);
  console.log('  Likely cause: C++ objects (NAPI) or native resources');
  console.log('  Action: Check NAPI addon for unreleased resources\n');
}

if (arrayBufferGrowth > 0.5 * 1024 * 1024) {
  console.log('ArrayBuffer Leak Detected:');
  console.log(`  ArrayBuffer memory growing at ${formatDelta(arrayBufferGrowth)}/s`);
  console.log('  Likely cause: Typed arrays (Float32Array) not being released');
  console.log('  Action: Check buffer.subarray() calls in polling loop\n');
}

if (heapGrowth < 0.1 * 1024 * 1024 && externalGrowth < 0.1 * 1024 * 1024 && arrayBufferGrowth < 0.1 * 1024 * 1024) {
  console.log('No significant memory growth detected.');
  console.log('Memory usage appears stable.\n');
}

console.log('=== NEXT STEPS ===\n');
console.log('1. Trigger manual GC from dev-ui and check if memory drops');
console.log('2. Take heap snapshot: dev-ui will call IPC handler');
console.log('3. Open .heapsnapshot file in Chrome DevTools Memory tab');
console.log('4. Look for retained objects and compare snapshots\n');

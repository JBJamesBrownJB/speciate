import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

/**
 * Contract test for the preload ↔ main IPC seam: every channel the preload
 * bridge uses must have a matching registration in napi-main.cjs. Guards
 * against the getLatestState class of bug — a bridge method invoking a channel
 * nothing handles, rejecting for every caller.
 *
 * Static source check on purpose: napi-main.cjs cannot be require()d outside
 * Electron, and the channel names are literals on both sides.
 */

const preloadSrc = readFileSync(resolve(__dirname, 'preload.cjs'), 'utf8');
// The main-process side spans napi-main plus the delivery modules it wires up.
const mainSrc = [
  readFileSync(resolve(__dirname, 'napi-main.cjs'), 'utf8'),
  readFileSync(resolve(__dirname, 'frameDelivery.cjs'), 'utf8'),
  readFileSync(resolve(__dirname, 'plantDelivery.cjs'), 'utf8'),
].join('\n');

const matchAll = (src: string, re: RegExp): string[] =>
  [...src.matchAll(re)].map((m) => m[1]);

describe('preload ↔ napi-main IPC channel contract', () => {
  it('every ipcRenderer.invoke channel has an ipcMain.handle registration', () => {
    const invoked = matchAll(preloadSrc, /ipcRenderer\.invoke\(\s*'([^']+)'/g);
    const handled = matchAll(mainSrc, /ipcMain\.handle\(\s*'([^']+)'/g);

    expect(invoked.length).toBeGreaterThan(0);
    for (const channel of invoked) {
      expect(handled, `invoke('${channel}') has no ipcMain.handle`).toContain(channel);
    }
  });

  it('every ipcRenderer.send channel has an ipcMain.on registration', () => {
    const sent = matchAll(preloadSrc, /ipcRenderer\.send\(\s*'([^']+)'/g);
    const listened = matchAll(mainSrc, /ipcMain\.on\(\s*'([^']+)'/g);

    expect(sent.length).toBeGreaterThan(0);
    for (const channel of sent) {
      expect(listened, `send('${channel}') has no ipcMain.on`).toContain(channel);
    }
  });

  it('every preload subscription channel is actually emitted by napi-main', () => {
    const subscribed = matchAll(preloadSrc, /ipcRenderer\.on\(\s*'([^']+)'/g);
    // webContents.send / removeAllListeners cleanup lines don't count as emitters.
    const emitted = matchAll(mainSrc, /webContents\.send\(\s*'([^']+)'/g);

    expect(subscribed.length).toBeGreaterThan(0);
    for (const channel of subscribed) {
      expect(emitted, `on('${channel}') is never emitted by napi-main`).toContain(channel);
    }
  });
});

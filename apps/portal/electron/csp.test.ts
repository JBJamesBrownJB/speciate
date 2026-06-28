import { describe, it, expect } from 'vitest';
// @ts-ignore - plain CJS module under test; types not needed for this spec
import { PROD_CSP, shouldApplyCsp, applyCspHeaders } from './csp.cjs';

describe('production Content-Security-Policy', () => {
  it('forbids the XSS vectors (unsafe-eval / unsafe-inline) in script-src', () => {
    // The whole point: the packaged bundle has no eval/inline scripts, so the
    // dev-only Electron "insecure CSP" warning becomes a real, locked-down policy.
    expect(PROD_CSP).not.toMatch(/script-src[^;]*'unsafe-eval'/);
    expect(PROD_CSP).not.toMatch(/script-src[^;]*'unsafe-inline'/);
  });

  it('defaults to self and locks down object/base/frame ancestors', () => {
    expect(PROD_CSP).toContain("default-src 'self'");
    expect(PROD_CSP).toContain("object-src 'none'");
    expect(PROD_CSP).toContain("base-uri 'self'");
  });

  it('keeps Pixi working: blob workers + data/blob images allowed', () => {
    // Pixi v8 spins up web workers (vite emits a webworker chunk) and uploads
    // textures from data:/blob: URLs — a too-strict policy would break rendering.
    expect(PROD_CSP).toMatch(/worker-src[^;]*blob:/);
    expect(PROD_CSP).toMatch(/img-src[^;]*blob:/);
    expect(PROD_CSP).toMatch(/img-src[^;]*data:/);
  });

  it('applies ONLY in packaged builds — dev (Vite HMR needs eval) is untouched', () => {
    expect(shouldApplyCsp({ isPackaged: true })).toBe(true);
    expect(shouldApplyCsp({ isPackaged: false })).toBe(false);
    expect(shouldApplyCsp(undefined)).toBe(false);
    expect(shouldApplyCsp(null)).toBe(false);
  });

  it('injects the CSP header while preserving existing response headers', () => {
    let captured: any;
    const fakeSession = {
      webRequest: {
        onHeadersReceived: (cb: (d: any, done: (r: any) => void) => void) => {
          cb({ responseHeaders: { 'X-Existing': ['keep-me'] } }, (r) => {
            captured = r;
          });
        },
      },
    };
    applyCspHeaders(fakeSession, "default-src 'self'");
    expect(captured.responseHeaders['Content-Security-Policy']).toEqual([
      "default-src 'self'",
    ]);
    // Must not clobber headers Electron/Chromium already set.
    expect(captured.responseHeaders['X-Existing']).toEqual(['keep-me']);
  });

  it('defaults applyCspHeaders to the production policy when none is passed', () => {
    let captured: any;
    const fakeSession = {
      webRequest: {
        onHeadersReceived: (cb: (d: any, done: (r: any) => void) => void) => {
          cb({ responseHeaders: {} }, (r) => {
            captured = r;
          });
        },
      },
    };
    applyCspHeaders(fakeSession);
    expect(captured.responseHeaders['Content-Security-Policy']).toEqual([PROD_CSP]);
  });
});

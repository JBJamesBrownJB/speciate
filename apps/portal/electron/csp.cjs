'use strict';

/**
 * Content-Security-Policy for the PACKAGED portal build.
 *
 * In dev, Vite's HMR runtime needs 'unsafe-eval', which is why Electron prints
 * its "insecure CSP" warning (dev-only). The production `vite build` bundle has
 * no eval/inline scripts, so the packaged app can run under a strict policy —
 * this is that policy. WHY each non-'self' source is present:
 *   - style-src 'unsafe-inline' : React/Pixi inject inline styles (low risk; not a script vector)
 *   - img-src data: blob:       : Pixi uploads textures from data:/blob: URLs
 *   - worker-src blob:          : Pixi v8 spawns web workers (vite emits a webworker chunk)
 *   - font-src data:            : bundled/embedded fonts
 * script-src is locked to 'self' (no eval, no inline) — that's the real hardening.
 * See https://electronjs.org/docs/tutorial/security
 */
const PROD_CSP = [
  "default-src 'self'",
  "script-src 'self'",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data: blob:",
  "font-src 'self' data:",
  "connect-src 'self' data: blob:",
  "worker-src 'self' blob:",
  "object-src 'none'",
  "base-uri 'self'",
  "frame-src 'none'",
].join('; ');

/**
 * Apply the CSP only in packaged/production builds. Dev (run from source) is left
 * permissive so Vite HMR keeps working. `app.isPackaged` is the canonical signal —
 * true only in a packaged artifact, independent of NODE_ENV being set.
 * @param {{ isPackaged?: boolean } | null | undefined} app the Electron `app` (or a stub in tests)
 */
function shouldApplyCsp(app) {
  return Boolean(app && app.isPackaged);
}

/**
 * Register a CSP header on all responses for the given Electron session, merging
 * with (not replacing) headers Chromium/Electron already set.
 * @param {{ webRequest: { onHeadersReceived: Function } }} session e.g. session.defaultSession
 * @param {string} [csp] policy string; defaults to PROD_CSP
 */
function applyCspHeaders(session, csp = PROD_CSP) {
  session.webRequest.onHeadersReceived((details, callback) => {
    callback({
      responseHeaders: {
        ...details.responseHeaders,
        'Content-Security-Policy': [csp],
      },
    });
  });
}

module.exports = { PROD_CSP, shouldApplyCsp, applyCspHeaders };

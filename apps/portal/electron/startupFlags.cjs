/**
 * Chromium command-line switches that exist ONLY to work around Linux sandbox
 * crashes (SUID/ESRCH). They weaken process isolation, so they must never be
 * applied on Windows/macOS where the sandbox works fine.
 */
function sandboxWorkaroundSwitches(platform) {
  if (platform !== 'linux') return [];
  return [
    ['no-sandbox'],
    ['disable-gpu-sandbox'],
    ['disable-dev-shm-usage'],
  ];
}

/**
 * The dev-tools window is opt-in: dev mode AND an explicit --dev-tools flag
 * (`npm run dev:tools`). Plain `npm run dev` is just the game window.
 */
function shouldOpenDevTools(isDev, argv) {
  return Boolean(isDev) && argv.includes('--dev-tools');
}

module.exports = { sandboxWorkaroundSwitches, shouldOpenDevTools };

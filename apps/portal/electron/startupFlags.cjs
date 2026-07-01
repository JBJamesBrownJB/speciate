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

module.exports = { sandboxWorkaroundSwitches };

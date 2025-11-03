/**
 * Debug utility for development logging
 *
 * Usage:
 * - DEBUG flag: Use to check if debug mode is enabled
 * - debugLog(): Use instead of console.log for development-only logs
 *
 * Debug mode is enabled when:
 * - Running in development (import.meta.env.DEV)
 * - localStorage.DEBUG is set to 'true'
 */

export const DEBUG = (import.meta as any).env?.DEV && localStorage.getItem('DEBUG') === 'true';

/**
 * Logs only when DEBUG flag is enabled
 * Use this for development-only logging that should never run in production
 */
export function debugLog(...args: any[]): void {
  if (DEBUG) {
    console.log('[DEBUG]', ...args);
  }
}

/**
 * ErrorBoundary
 *
 * Converts any render-time throw in the dev-tools subtree into a visible panel
 * instead of an unmounted (blank white) window. Pairs with the forwarded
 * renderer console (Electron main) so the stack trace reaches the terminal.
 */

import React from 'react';

interface Props {
  children: React.ReactNode;
}

interface State {
  error: Error | null;
}

export class ErrorBoundary extends React.Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo): void {
    console.error('[dev-ui] render error:', error, info.componentStack);
  }

  render(): React.ReactNode {
    if (this.state.error) {
      return (
        <div className="panel" role="alert">
          <div className="panel-header">Dev UI crashed</div>
          <p className="no-data">{this.state.error.message}</p>
          <p className="hint">
            Open this window's console (Ctrl+Shift+I) or the launching terminal for the
            full stack trace.
          </p>
        </div>
      );
    }
    return this.props.children;
  }
}

export default ErrorBoundary;

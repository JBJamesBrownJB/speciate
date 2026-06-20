/**
 * Dev Tools Entry Point
 *
 * This is the entry point for the developer tools window.
 * Launched when simulation is started with --dev-tools flag.
 */

import React from 'react';
import ReactDOM from 'react-dom/client';
import { DevToolsApp } from './components/DevToolsApp';
import { ErrorBoundary } from './components/ErrorBoundary';
import './index.css';

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);

root.render(
  <React.StrictMode>
    <ErrorBoundary>
      <DevToolsApp />
    </ErrorBoundary>
  </React.StrictMode>
);

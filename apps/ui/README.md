# Frontend Application

The client-side web application for the Speciate AI life simulation, built with TypeScript and Pixi.js.

## Overview

This frontend provides:
- **Real-time simulation rendering** - High-performance 2D graphics using Pixi.js
- **Player interaction** - UI for actions, inventory, and economy management
- **Client-side prediction** - Smooth rendering ahead of server state
- **WebSocket connection** - Live updates from simulation server
- **Economy UI** - Resource tracking and trading interface

## Technology Stack

- **Vanilla TypeScript + Vite** - Currently using vanilla TypeScript for simplicity (planned migration to React in future sprint)
- **TypeScript** - Type-safe JavaScript
- **Pixi.js** - High-performance 2D rendering library
- **Vite** - Modern build tool with fast HMR
- **WebSocket** - Real-time bidirectional communication with Rust simulation server
- **Vitest/Jest** - Testing framework

## Prerequisites

- **Node.js 18+** - Install from [nodejs.org](https://nodejs.org/)

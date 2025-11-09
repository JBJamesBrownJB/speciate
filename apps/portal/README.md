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
- **Pixi.js v8.14** - High-performance 2D rendering library
- **Vite** - Modern build tool with fast HMR
- **WebSocket** - Real-time bidirectional communication with Rust simulation server
- **Vitest** - Testing framework with 172 tests passing

## Prerequisites

- **Node.js 18+** - Install from [nodejs.org](https://nodejs.org/)

## Architecture

### Clean Architecture Layers

```
src/
├── domain/          # Pure business logic (no dependencies)
│   ├── Camera.ts       # World-to-screen coordinate transforms
│   ├── Viewport.ts     # Viewport culling and bounds
│   └── Creature.ts     # Immutable entity data model
├── infrastructure/  # External dependencies & optimizations
│   └── SpritePool.ts   # Object pooling for 1000+ entities
├── rendering/       # Pixi.js rendering layer
│   └── SpriteProvider.ts
├── utils/          # Pure utility functions
│   └── SpatialQuery.ts # Distance & viewport intersection
└── main.ts         # Application orchestration
```

### Rendering Architecture

**World Container Pattern:**
- All game objects positioned in **world coordinates** (meters)
- Camera transforms the **entire world container** (not individual sprites)
- Sprites use **uniform scaling** to preserve aspect ratio
- Zero simulation logic in the frontend

```typescript
// World container holds all sprites at world coordinates
const worldContainer = new Container();

// Camera applies zoom and position to entire container
camera.applyTransform(worldContainer, screenWidth, screenHeight);

// Sprites positioned in meters (NOT pixels)
sprite.position.set(creature.x, creature.y);

// Uniform scaling preserves aspect ratio
const worldScale = Math.min(
  creature.width / texture.width,
  creature.height / texture.height
);
sprite.scale.set(worldScale);
```

### Performance Optimizations

1. **Object Pooling** - `SpritePool` reuses sprites to avoid GC pressure
2. **Viewport Culling** - Only renders visible entities
3. **Batch Rendering** - Single container enables Pixi.js batching
4. **Efficient Updates** - Position/scale updates (no sprite recreation)

### Key Design Principles

- **Zero Simulation Logic**: All game logic runs in Rust backend
- **Unidirectional Data Flow**: Frontend only renders server state
- **Domain Boundaries**: Camera uses `ITransformable` interface (Dependency Inversion)
- **Separation of Concerns**: `SpatialQuery` utility for rendering calculations
- **Immutability**: Domain models return new instances (functional style)

## Test Coverage

- **172 tests passing** across all layers
- **TDD approach** - tests written before implementation
- **Unit tests** - All domain, infrastructure, and utility layers
- **Integration tests** - WebSocket client and state management

Test suites:
- `Camera.test.ts` - 29 tests (coordinate transforms, zoom, applyTransform)
- `Creature.test.ts` - 11 tests (immutability, data model)
- `Viewport.test.ts` - 16 tests (culling, world bounds)
- `SpritePool.test.ts` - 24 tests (pooling, acquire/release)
- `SpatialQuery.test.ts` - 16 tests (distance, viewport intersection)
- `StateManager.test.ts` - 13 tests (state updates)
- `WebSocketClient.test.ts` - 27 tests (connection lifecycle)
- `Interpolator.test.ts` - 11 tests (smooth movement)
- `messages.test.ts` - 25 tests (message validation)

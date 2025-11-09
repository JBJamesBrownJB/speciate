# Portal Frontend Architecture

**Last Updated:** 2025-11-06
**Sprint:** 5 - Sprite Rendering Refactor
**Status:** ✅ Architecturally Compliant (Approved by Architect & Frontend Specialist)

## Overview

The portal frontend is a high-performance, clean-architecture TypeScript application built with Pixi.js. It renders real-time simulation state from the Rust backend with **zero simulation logic** on the client.

## Core Principles

### 1. Zero Simulation Logic
- **All game logic runs in Rust backend** (ECS, A-Life, physics)
- Frontend **only renders** state received via WebSocket
- No client-side prediction of game mechanics
- Unidirectional data flow: Server → Frontend

### 2. Clean Architecture
- **Domain Layer**: Pure business logic (no external dependencies)
- **Infrastructure Layer**: External dependencies (Pixi.js, WebSocket)
- **Utility Layer**: Pure functions (spatial calculations)
- **Rendering Layer**: Pixi.js-specific rendering code

### 3. Performance First
- Object pooling for sprites (1000+ entities)
- Viewport culling (only render visible entities)
- Batch rendering (single world container)
- Efficient updates (no sprite recreation)

## Directory Structure

```
src/
├── domain/                 # Pure business logic
│   ├── Camera.ts              # Coordinate transforms, zoom
│   ├── Viewport.ts            # Viewport bounds, culling
│   ├── Creature.ts            # Immutable entity data
│   └── Interpolator.ts        # Smooth movement between updates
│
├── infrastructure/        # External dependencies
│   └── SpritePool.ts          # Object pooling optimization
│
├── rendering/            # Pixi.js rendering layer
│   └── SpriteProvider.ts      # Texture management
│
├── core/                 # Application core
│   ├── WebSocketClient.ts     # Server connection
│   ├── StateManager.ts        # State updates
│   └── constants.ts           # Configuration constants
│
├── utils/                # Pure utility functions
│   └── SpatialQuery.ts        # Distance, viewport intersection
│
├── types/                # TypeScript types
│   └── messages.ts            # WebSocket message schemas
│
└── main.ts               # Application entry point
```

## Rendering Architecture

### World Container Pattern

The frontend uses the **industry-standard world container pattern** for camera/viewport management:

```typescript
// Create world container
const worldContainer = new Container();
app.stage.addChild(worldContainer);

// Apply camera transform to entire container (NOT individual sprites)
camera.applyTransform(worldContainer, screenWidth, screenHeight);

// Add sprites at WORLD coordinates (meters)
sprite.position.set(creature.x, creature.y);
worldContainer.addChild(sprite);
```

**Benefits:**
- Single transform point (camera) instead of per-sprite transforms
- Efficient matrix multiplication
- Sprites use world coordinates directly
- Clean separation: camera handles view, sprites handle position

### Coordinate Systems

**Two coordinate systems:**

1. **World Space** (meters)
   - Used by simulation backend (Rust ECS)
   - Range: -1,000,000 to +1,000,000 meters
   - Creature positions: `creature.x`, `creature.y`

2. **Screen Space** (pixels)
   - Used by Pixi.js rendering
   - Range: 0 to viewport dimensions
   - Converted via `camera.worldToScreen()` / `camera.screenToWorld()`

**Camera zoom:**
- Measured in **pixels per meter**
- Min: 1 px/m (zoomed out)
- Max: 100 px/m (zoomed in)

### Sprite Scaling

**Uniform scaling to preserve aspect ratio:**

```typescript
// ✅ CORRECT: Preserves aspect ratio
const worldScale = Math.min(
  creature.width / texture.width,
  creature.height / texture.height
);
sprite.scale.set(worldScale); // Uniform scale (x and y equal)

// ❌ WRONG: Causes squishing/warping
sprite.scale.x = creature.width / texture.width;
sprite.scale.y = creature.height / texture.height;
```

**Why this works:**
- `Math.min()` ensures sprite fits within creature bounds
- Uniform scaling (`scale.set(value)`) prevents distortion
- Texture aspect ratio is preserved
- Works for all shapes: square, wide, tall

## Domain Layer

### Camera (`src/domain/Camera.ts`)

**Responsibilities:**
- Store camera position (x, y) in world space
- Store zoom level (pixels per meter)
- Convert between world and screen coordinates
- Apply transform to rendering containers

**Key Methods:**
```typescript
class Camera {
  move(x: number, y: number): void
  setZoom(zoom: number): void
  worldToScreen(worldX: number, worldY: number): { x: number; y: number }
  screenToWorld(screenX: number, screenY: number): { x: number; y: number }
  applyTransform(container: ITransformable, screenWidth: number, screenHeight: number): void
}
```

**Interface Boundary:**
```typescript
export interface ITransformable {
  scale: { set(x: number, y?: number): void };
  position: { set(x: number, y: number): void };
}
```

This interface provides a **domain boundary** - Camera stays in domain layer but can work with rendering layer objects (Dependency Inversion Principle).

### Viewport (`src/domain/Viewport.ts`)

**Responsibilities:**
- Calculate visible world bounds
- Perform viewport culling
- Filter creatures to only visible ones

**Key Methods:**
```typescript
class Viewport {
  resize(width: number, height: number): void
  getWorldBounds(camera: Camera): WorldBounds
  isCreatureVisible(creature: Creature, camera: Camera): boolean
  cullCreatures(creatures: Creature[], camera: Camera): Creature[]
}
```

**Viewport culling** is critical for performance at scale (10k+ entities).

### Creature (`src/domain/Creature.ts`)

**Responsibilities:**
- Immutable value object for entity data
- No simulation logic (distance calculations moved to `SpatialQuery`)
- Provides helper methods for creating new instances

**Properties:**
```typescript
class Creature {
  readonly id: number
  readonly x: number        // World X (meters)
  readonly y: number        // World Y (meters)
  readonly rotation: number // Radians
  readonly width: number    // Meters
  readonly height: number   // Meters
}
```

**Immutability pattern:**
```typescript
// Creates NEW instance (functional style)
const updated = creature.withPosition(10, 20);
```

## Infrastructure Layer

### SpritePool (`src/infrastructure/SpritePool.ts`)

**Responsibilities:**
- Object pooling for sprite reuse
- Prevent garbage collection pressure
- MANDATORY for 1000+ entities

**Pattern:**
```typescript
class SpritePool {
  acquire(entityId: number, texture: Texture): Sprite
  release(entityId: number): void
  releaseAll(): void
  isActive(entityId: number): boolean
  getPoolSize(): number
  getActiveCount(): number
}
```

**Usage:**
```typescript
// Get sprite (creates new or reuses existing)
const sprite = spritePool.acquire(creature.id, texture);
worldContainer.addChild(sprite);

// Release when entity dies (keeps in pool for reuse)
spritePool.release(creature.id);
```

## Utility Layer

### SpatialQuery (`src/utils/SpatialQuery.ts`)

**Responsibilities:**
- Pure functions for spatial calculations
- Separated from domain models (no simulation logic in UI)

**Methods:**
```typescript
class SpatialQuery {
  static distance(x1: number, y1: number, x2: number, y2: number): number
  static isInViewport(
    entity: { x: number; y: number; width: number; height: number },
    viewportBounds: { minX: number; maxX: number; minY: number; maxY: number }
  ): boolean
}
```

**Why this exists:**
- Removed `distanceTo()`, `distanceToPoint()`, `getBounds()` from `Creature`
- Keeps domain models as pure data
- Allows rendering calculations without simulation logic

## Performance Optimizations

### Current Optimizations (Implemented)

1. **Object Pooling** (`SpritePool`)
   - Reuses sprites instead of create/destroy
   - Prevents GC pressure
   - Tested with 1000+ sprites

2. **Batch Rendering**
   - All sprites in single `worldContainer`
   - Pixi.js automatic batching enabled
   - Reduces draw calls

3. **Efficient Updates**
   - Update position/scale (don't recreate sprites)
   - Only update what changed
   - No unnecessary re-renders

4. **Viewport Culling** (Basic)
   - `Viewport.cullCreatures()` filters visible entities
   - Uses efficient AABB intersection tests

### Future Optimizations (Recommended)

1. **Advanced Viewport Culling** (High Priority for 10k+ entities)
   ```typescript
   // Hide sprites outside viewport
   sprite.visible = camera.isInViewport(creature.x, creature.y);
   ```

2. **Interpolation** (High Priority for smooth 60 FPS)
   ```typescript
   // Smooth movement between server updates (10-20Hz)
   sprite.position.set(
     lerp(creature.x, creature.targetX, alpha),
     lerp(creature.y, creature.targetY, alpha)
   );
   ```

3. **Level of Detail (LOD)** (Medium Priority for 100k+ entities)
   - Simplified rendering for distant entities
   - Texture downsampling based on zoom
   - Placeholder sprites at extreme distances

4. **Spatial Partitioning** (Medium Priority for large worlds)
   - Quadtree for efficient spatial queries
   - Only check nearby entities for culling
   - Faster than checking all entities

## Testing Strategy

### Test-Driven Development (TDD)

All new features follow **RED → GREEN → REFACTOR**:

1. **RED**: Write failing test
2. **GREEN**: Implement minimal code to pass
3. **REFACTOR**: Clean up implementation

### Test Coverage: 172 Tests Passing

| Suite | Tests | Coverage |
|-------|-------|----------|
| `Camera.test.ts` | 29 | Transforms, zoom, applyTransform |
| `Creature.test.ts` | 11 | Immutability, data model |
| `Viewport.test.ts` | 16 | Culling, world bounds |
| `SpritePool.test.ts` | 24 | Pooling, acquire/release |
| `SpatialQuery.test.ts` | 16 | Distance, viewport intersection |
| `StateManager.test.ts` | 13 | State updates |
| `WebSocketClient.test.ts` | 27 | Connection lifecycle, reconnection |
| `Interpolator.test.ts` | 11 | Smooth movement |
| `messages.test.ts` | 25 | Message validation, schemas |

### Testing Best Practices

- **Unit tests** for all domain and utility layers
- **Integration tests** for WebSocket and state management
- **No mocking** of domain models (use real instances)
- **Mock external dependencies** (Pixi.js, WebSocket)
- **Test edge cases** (negative coordinates, large numbers, zero values)

## WebSocket Communication

### Message Format

Frontend receives updates via WebSocket from Rust backend:

```typescript
// SimulationFrame (new format from broadcaster)
{
  tick: number,
  agents: Array<{
    id: number,
    x: number,
    y: number,
    rotation?: number,
    width?: number,
    height?: number
  }>
}

// SimulationStateMessage (legacy format)
{
  tick: number,
  creatures: Array<{
    id: number,
    x: number,
    y: number,
    rotation?: number,
    width?: number,
    height?: number
  }>
}
```

**Message Validation:**
- `isSimulationFrame()` - Validates new format
- `isSimulationStateMessage()` - Validates legacy format
- `adaptSimulationFrame()` - Converts new → legacy for compatibility

### Connection Management

**Features:**
- Automatic reconnection with exponential backoff
- Connection state tracking (`Connecting`, `Connected`, `Disconnected`, `Reconnecting`)
- Ping tracking (measures latency)
- Graceful disconnect handling

**Usage:**
```typescript
const client = new WebSocketClient('ws://localhost:8080/stream');

client.onMessage(message => {
  // Handle simulation update
});

client.onConnectionStateChange(state => {
  // Update UI connection indicator
});

client.connect();
```

## Configuration

### Constants (`src/core/constants.ts`)

```typescript
export const NETWORK_CONFIG = {
  UPDATE_RATE_HZ: 20,              // Server sends updates at 20Hz
  RECONNECT_DELAY_MS: 3000,        // Wait 3s before reconnecting
  MAX_RECONNECT_ATTEMPTS: 5,       // Give up after 5 attempts
} as const;

export const RENDERING_CONFIG = {
  TARGET_FPS: 60,                  // Target 60 FPS rendering
  INTERPOLATION_BUFFER_MS: 100,    // Buffer for smooth interpolation
} as const;
```

## Architectural Reviews

### Architect Review (2025-11-06)
✅ **APPROVED** - Exemplary adherence to clean architecture principles

**Key Strengths:**
- Proper domain boundaries maintained
- No simulation logic leaked into frontend
- `ITransformable` interface demonstrates Dependency Inversion Principle
- Well-designed for target scale (1000+ entities)

**Recommendations:**
- Define asset strategy for texture atlases
- Document API contracts with Rust backend
- Align with ECS standards for component serialization

### Frontend Review (2025-11-06)
✅ **APPROVED FOR PRODUCTION**

**Key Strengths:**
- World container pattern correctly implemented
- Sprite scaling preserves aspect ratio (no squishing)
- Performance optimizations in place (pooling, batching)
- Clean architecture with SOLID principles

**Recommendations:**
- Implement view culling for 10k+ entities
- Add interpolation for smooth 60 FPS rendering
- Extract configuration constants to dedicated files

## Future Enhancements

### Next Sprint Priorities

1. **View Culling** (High Priority)
   - Set `sprite.visible = false` for off-screen entities
   - Target: 10k+ entities at 60 FPS

2. **Interpolation System** (High Priority)
   - Smooth movement between server updates (20Hz → 60 FPS)
   - Use existing `Interpolator` domain model

3. **Texture Atlas Loading** (Medium Priority)
   - CDN strategy for efficient texture delivery
   - Species-based atlases for variety

4. **API Contract Documentation** (Medium Priority)
   - Formal WebSocket message schemas
   - Versioning strategy for breaking changes

### Long-Term Vision

- **100k+ entities** with LOD system and spatial partitioning
- **Multiplayer UI** with player avatars and interactions
- **Economy interface** for resource management and trading
- **React migration** for complex UI components
- **Mobile support** with touch controls and responsive design

## References

- [Pixi.js Documentation](https://pixijs.com/)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [World Container Pattern](https://www.redblobgames.com/x/2024-camera-techniques/)
- [Object Pooling](https://gameprogrammingpatterns.com/object-pool.html)

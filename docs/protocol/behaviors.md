# Behavior Enum Contract

**Protocol Version:** 1
**Last Updated:** 2025-11-17

## Behavior Discriminants

| Discriminant | Rust Enum | TypeScript Name | Description |
|--------------|-----------|-----------------|-------------|
| 0 | `BehaviorMode::Catatonic` | "Catatonic" | No active behavior, idle state |
| 1 | `BehaviorMode::Seeking` | "Seeking" | Moving toward target |
| 2 | `BehaviorMode::Wandering` | "Wandering" | Random exploration |

## Rules

1. **New behaviors MUST be added at end** with next sequential discriminant
2. **Never reorder** existing discriminants
3. **Never remove** discriminants (deprecate instead)
4. **Both Rust and TypeScript must stay in sync**

## Implementation

### Rust (apps/simulation/src/simulation/creatures/components/state.rs)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize, Reflect)]
#[repr(u8)]
pub enum BehaviorMode {
    #[default]
    Catatonic = 0,
    Seeking = 1,
    Wandering = 2,
}
```

### TypeScript (apps/portal/src/types/behaviors.ts)

```typescript
export const BehaviorDiscriminant = {
  Catatonic: 0,
  Seeking: 1,
  Wandering: 2,
} as const;

export const BehaviorNames: Record<number, string> = {
  0: 'Catatonic',
  1: 'Seeking',
  2: 'Wandering',
};
```

## Adding New Behaviors

1. Add to Rust enum with explicit discriminant:
   ```rust
   Fleeing = 3,  // Next available
   ```

2. Update TypeScript mapping:
   ```typescript
   export const BehaviorDiscriminant = {
     // ... existing ...
     Fleeing: 3,
   } as const;

   export const BehaviorNames: Record<number, string> = {
     // ... existing ...
     3: 'Fleeing',
   };
   ```

3. Update this document

4. Bump protocol version if breaking change

## Why Explicit Discriminants?

Using `#[repr(u8)]` with explicit values prevents accidental reordering:

```rust
// SAFE: Explicit values
pub enum BehaviorMode {
    Catatonic = 0,
    Seeking = 1,
    Wandering = 2,
}

// DANGEROUS: Implicit values (adding Fleeing breaks TypeScript)
pub enum BehaviorMode {
    Catatonic,  // 0
    Seeking,    // 1
    Fleeing,    // 2 ← Breaks TypeScript expecting Wandering=2
    Wandering,  // 3
}
```

# Documentation Index

Welcome to the Biosphere project documentation. This index provides quick navigation to all project documents organized by topic.

## Quick Links

- [Project Specification](./project-spec.md) - High-level project vision and core concepts

## Architecture

System design, infrastructure, and technical architecture.

- [Electron Architecture](./architecture/electron-architecture.md) - NAPI-RS zero-copy IPC, viewport culling
- [Architecture Diagram](./architecture/diagrams/architecture-high.png) - Visual system overview

## Biology & Simulation

A-Life mechanics, genetics, creature behavior, and ecological systems.

- [DNA-Driven Design](./biology/dna-driven-design.md) - Core principle: all traits encoded in DNA
- [Biology Notes](./biology/biology-notes.md) - Zoologist consultations log with scientific formulas
- [Creature Behaviors](./biology/creature-behaviors.md) - Influence maps, steering behaviors, attraction rating
- [Stigmergy](./biology/stigmergy.md) - Environmental modification as communication (path trampling, emergence)
- [Neural Network Social Learning](./biology/nn-social-learning.md) - Imitation learning, cultural transmission, memetics
- [A-Life Features](./biology/alife-features.md) - Genetic algorithms, pheromones, cellular automata

## Performance

Optimization strategies for the Electron desktop game.

- [Optimization Catalog](./performance/optimization-catalog.md) - Viewport culling, ECS optimization, sprite pooling, save/load
- [GPU Compute Idea](./performance/gpu-compute-idea.md) - WGSL kernel strategy (deferred)

## Research

Technical decisions and technology evaluations.

- [Agent ID Strategy](./research/agent-id-nanoid.md) - NanoID vs UUID vs Snowflake analysis

## Gameplay

Player-facing features, UI patterns, and game mechanics.

- [Backlight UI Pattern](./gameplay/backlight-ui-pattern.md) - Ambient state indicator design
- [High Altitude Drone](./gameplay/high-altitude-drone.md) - Strategic view concept (future)

## Document Status Legend

- **Implemented** - Feature is live in the codebase
- **Planned** - Design complete, implementation scheduled
- **Draft** - Work in progress, not finalized
- **Deferred** - Good idea, but postponed for later sprints
- **Research** - Exploratory analysis or technology evaluation

## Contributing

When creating or updating documentation:

1. Use **kebab-case** for file names
2. Add status headers (Status, Last Updated, Related)
3. Use simple markdown (readable in both preview and plain text)
4. Update this index when adding new documents
5. Add cross-references to related documents

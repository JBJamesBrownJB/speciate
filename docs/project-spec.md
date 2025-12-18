# Project Spec: "Speciate"

**Status**: Foundational Document
**Last Updated**: 2025-11-10 (Updated for Steam Early Access pivot)
**Related**: [DNA-Driven Design](./biology/dna-driven-design.md), [Business Strategy](./strategy/biz-strategy.md), [Game Goal](./strategy/goal.md)

---

## 🚀 Current Development Phase

**Phase 1: Steam Early Access (Target: Q2 2026)**

**Speciate** is a **single-player desktop game** featuring DNA-driven artificial life simulation set on an alien planet. Players explore, survive, and interact with an evolving ecosystem of autonomous creatures.

**Platform:** Windows, Mac, Linux (via Electron)
**Price:** $20-30 one-time purchase
**Distribution:** Steam Early Access

**Phase Strategy:**
- **Phase 1 (6-9 months):** Standalone sandbox game → Build community, validate concept
- **Phase 1.5 (post-launch):** Narrative campaign (daughter rescue story) → Retention & reviews
- **Phase 2 (future):** Web MMO expansion → If Phase 1 succeeds (see [Business Strategy](./strategy/biz-strategy.md))

---

## 1. 🎯 Core Concept

A **DNA-driven artificial life simulation** where hundreds of autonomous creatures evolve, compete, and adapt in a procedurally generated ecosystem.

**Phase 1 (Current) - Single-Player Sandbox:**
* **Observation & Exploration:** Navigate a vast alien world with fog of war
* **Survival:** Gather biomass, craft tools, manage resources
* **Ecosystem Interaction:** Tame creatures, breed selective traits, manipulate environment
* **DNA-Driven Emergence:** All creature behavior flows from genetic code (no scripted NPCs)

**Phase 1.5 (Post-Launch) - Narrative Campaign:**
* **Story Goal:** Find and rescue daughter across dangerous alien planet
* **Systemic Challenge:** Navigate predator territories using tamed creatures and ecosystem knowledge
* **Emotional Climax:** Player-driven victory through mastery of A-Life systems

**Phase 2 (Future Vision) - Web MMO:**
* **Multiplayer:** Shared world with thousands of players
* **Player Economy:** DNA ownership, biomass trading, speciation events
* **Conservation:** Compete to protect/exploit endangered species
* **Status:** Deferred pending Phase 1 success

---

## 2. 🏛️ Architectural Model

### Phase 1: Electron Desktop (Current)

**Architecture:** Local simulation bundled with rendering frontend into desktop executable

**Core Components:**
* **Simulation Backend (Rust/Bevy ECS):** Runs AI, physics, and state logic locally on player's machine
  - **22.2Hz Unified Tick:** Physics, perception, and behavior in single schedule
  - **Rayon Parallelization:** Multi-core execution (16 cores, 6.3x speedup)
  - **NAPI-RS Zero-Copy:** Direct buffer sharing with frontend (<1ms overhead)
  - **Target:** 150,000-200,000 creatures via time-slicing + viewport culling

* **Frontend (PixiJS):** Renders at 90 FPS with interpolation, receives state via NAPI buffer
  - **Interpolated smoothing:** 90Hz visuals from 22Hz physics snapshots
  - **Full f32 precision:** No quantization (local IPC, no network limitations)

* **Wrapper (Electron):** Bundles Rust + TypeScript into single `.exe`/`.app`/`.AppImage` for Steam distribution

**See:** [Electron Architecture Documentation](./architecture/electron-architecture.md)

**Benefits:**
- Zero server costs ($228k/year eliminated)
- Simpler development (no network synchronization)
- Faster iteration (local testing, no deployment)
- Compiled Rust (harder to pirate than web)

---

### Phase 2: Headless Server / Microservices (Future Vision)

**Architecture:** Server-authoritative MMO with distributed microservices

**Core Components:**
* **Simulation Server (Rust - Authoritative):** Headless Bevy ECS, sole source of truth for real-time simulation
* **NATS Message Broker:** Pub/sub streaming (8-11M msg/sec capacity)
* **Broadcaster (Node.js):** WebSocket distribution to thousands of clients
* **Client (Web Browser):** PixiJS rendering with client-side interpolation (10-20 Hz → 60-90 FPS)
* **Economy Ledger (Node.js/PostgreSQL):** ACID-compliant asset management
* **Player Commander:** Authentication and command validation

**Status:** Deferred pending Phase 1 success.

---

## 3. 🧬 Emergent Agent and Player Archetypes

All non-player entities exhibit **infinite procedural variation** driven by their genetic data. Roles **MUST** emerge from DNA-defined parameters, not hardcoded labels.

### A. Non-Player Organisms (NPCs)

The core food chain is implemented through DNA-defined behaviors:

* **Plants (Flora):** Possess a **simpler DNA model** defining growth rate, environmental tolerance, and **Biomass** yield. They form the **absolute bottom** of the food chain.
* **Creatures (Fauna):** Possess complex DNA defining both phenotype (visuals) and behavior. Trophic roles must emerge dynamically:
    * **Herbivores:** Behavior is defined by DNA parameters that prioritize eating plants and avoiding high-risk entities.
    * **Predators:** Behavior is defined by DNA parameters that prioritize hunting entities with specific traits (e.g., size, speed) and avoiding larger, stronger entities. This includes **intraspecies predation** (larger on smaller).
    * **Omnivores/Scavengers:** Exhibit flexible behavior prioritizing both plant matter, smaller creatures, and consuming dead biomass.

### B. Player Avatar

**Phase 1 (Current) - Survivor:**
* **Role:** Crash survivor exploring alien planet to find daughter
* **Gameplay:** Survival (hunger, crafting), exploration (fog of war), creature taming/interaction
* **Vulnerability:** Not exempt from simulation (can be hunted by predators)
* **Camera:** Top-down 2D view with zoom (0.0005 - 200 px/m)

**Phase 1.5 (Post-Launch) - Daughter Rescue:**
* **Narrative Goal:** Navigate to daughter's pod across dangerous terrain
* **Mechanics:** Taming (beacon zones, harpoon capture, DNA cloning), creature commands (Thumper, herding)
* **Endgame:** Gauntlet challenge through Karg territory using tamed creatures

**Phase 2 (Future) - Influencer:**
* **Role:** Player as "Influencer" in shared MMO world
* **Economy:** DNA ownership, biomass trading, speciation events
* **Competition:** Leaderboards, conservation mechanics, ecosystem management

---

## 4. 🏦 Economy & Persistence

### Phase 1 (Current) - Local Save/Load

**Architecture:** Single-player local save files
* **Save System:** Serialize Bevy World to disk (bincode/MessagePack)
* **Load System:** Deserialize World state on game launch
* **Steam Cloud:** Optional cloud save sync via Steam API
* **No Economy:** Phase 1 is sandbox, no trading/ownership

### Phase 2 (Future) - Economy Ledger Microservice 💰

**Architecture:** Dedicated, secure service for player asset management

| Component | Technologies | Key Role/Purpose |
| :--- | :--- | :--- |
| **Service Language** | **Node.js + TypeScript** | Runtime for secure, transactional API layer |
| **Persistence Engine** | **PostgreSQL** | ACID properties, concurrency, data integrity for player assets |
| **Interface / API** | **REST** | All external components (Simulation Server) interact via this API |
| **Assets** | DNA ownership, biomass, resources | Player-owned assets with scarcity and trading |

**Status:** Phase 2 feature, pending Early Access success

---

## 5. 🛠️ General Technology Stack

### Phase 1 (Current) - Electron Desktop Application

**Backend:**
| Component | Technologies | Key Role/Purpose |
| :--- | :--- | :--- |
| **Core Logic (ECS)** | `bevy_ecs` + `bevy_app` | 22.2Hz unified tick, Rayon parallelization |
| **Spatial Grid** | `FxHashMap` + 50m cells | O(N) queries, viewport culling |
| **IPC Protocol** | `NAPI-RS` | Zero-copy double-buffered position data |
| **Serialization** | `serde` + `bincode` | Save/load persistence |

**Frontend:**
| Component | Technologies | Key Role/Purpose |
| :--- | :--- | :--- |
| **Rendering** | `Pixi.js` | 2D WebGL renderer at 90 FPS (interpolated) |
| **UI Framework** | HTML/DOM | Menus, inventory, HUD (standard web UI) |
| **IPC Client** | `window.electron` (preload) | Receive state updates from main process |
| **Interpolation** | Custom lerp system | 30 Hz snapshots → 90 FPS smooth visuals |

**Deployment:**
| Component | Technologies | Key Role/Purpose |
| :--- | :--- | :--- |
| **Distribution** | **Steam** | Early Access release, cloud saves, achievements |
| **CI/CD** | **GitHub Actions** | Build Windows/Mac/Linux binaries, automated testing |

---

### Phase 2 (Future) - Web MMO Infrastructure

**Backend:**
| Component | Technologies | Key Role/Purpose |
| :--- | :--- | :--- |
| **Core Logic (ECS)** | `bevy_ecs` + `bevy_app` | Server-authoritative simulation at 20 Hz |
| **Networking** | `tokio` + `axum` | WebSocket state sync, REST APIs |
| **Message Broker** | **NATS** | Pub/sub streaming (8-11M msg/sec capacity) |
| **Broadcaster** | **Node.js** | WebSocket distribution to thousands of clients |
| **Deployment** | **Terraform** + **GCP** | Cloud infrastructure, Kubernetes orchestration |

**Frontend:**
| Component | Technologies | Key Role/Purpose |
| :--- | :--- | :--- |
| **Rendering** | `Pixi.js` | Browser-based 2D WebGL renderer |
| **Interpolation** | Custom lerp system | 10-20 Hz server updates → 60-90 FPS smooth visuals |
| **State Management** | Delta encoding + quantization | Bandwidth optimization for networked gameplay |

**Status:** Archived in `archive/mmo-streaming-v1` branch, pending Phase 1 success
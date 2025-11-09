# Project Spec: "Speciate" (Finalized)

**Status**: Foundational Document
**Last Updated**: 2025-11-09
**Related**: [DNA-Driven Design](./biology/dna-driven-design.md), [Architectural Patterns](./architecture/architectural-patterns.md)

## Architecture & Technology Specification

---

## 1. 🎯 Core Concept

A persistent, **server-authoritative artificial life simulation** with a player-driven economy. 
Players are "Influencers" in a shared world populated by hundreds of thousands of autonomous, evolving agents.

The core gameplay loop involves:
* **Indirect Control** (influencing the environment).
* **Survival** (resource management, crafting).
* **Economic & Technological Progression**.

Players compete to harvest resources and "**biomass**" (accrued from cultivating life forms). The ultimate goal is to trigger "**speciation events**," which grant the player ownership of unique "**DNA**" assets, forming the basis of the game's economy.

---

## 2. 🏛️ Architectural Model (Non-Negotiable)

We use a **Headless Server / Thin Client / Microservice** model to secure the economy and prevent cheating.

### Simulation Server (Rust - Authoritative)
* **Stateful:** A single, headless **Rust** application (`bevy_ecs`). It is the **sole source of truth** for all **real-time simulation logic**.
* **Decoupling:** **Does not** directly access the database. It communicates with the **Economy Ledger Microservice** via API calls for all asset changes.

### Client (TypeScript - Thin Client)
* A **TypeScript** web application (`Pixi.js`). Responsible only for rendering, input, and **client-side prediction**.
* **Performance:** Must utilize **interpolation** to translate the server's low-frequency updates (10-20 Hz) into a fluid **60-90 FPS** visual experience.

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

### B. Player Avatar (The Influencer)

* **Role:** The player controls a simple **2D top-down human avatar**.
* **Gameplay:** The avatar's primary interaction is survival, resource gathering, and influencing the environment.
* **Vulnerability:** The player avatar is **not exempt from the simulation** and is considered a target/prey for larger, DNA-driven predatory agents.

---

## 4. 🏦 Economy Ledger Microservice (Decoupled, Secure, ACID) 💰

This is a dedicated, secure service that is the final authority on all player assets.

| Component | Technologies | Key Role/Purpose |
| :--- | :--- | :--- |
| **Service Language** | **Node.js + TypeScript** | The runtime for the secure, transactional API layer. |
| **Persistence Engine** | **`PostgreSQL`** | **Mandatory.** The database guaranteeing **ACID properties**, concurrency, and data integrity for all player assets. |
| **Interface / API** | **REST** | All external components (including the Simulation Server) interact with the ledger through this API. |

---

## 5. 🛠️ General Technology Stack

### Backend / Infrastructure

| Component | Technologies | Key Role/Purpose |
| :--- | :--- | :--- |
| **Core Logic (ECS)** | `bevy_ecs` + `bevy_app` | High-performance, multi-threaded simulation, running at a **fixed tick rate (e.g., 20Hz)**. |
| **Networking** | `tokio` + `axum` | `axum` manages persistent **WebSocket** state sync with clients and makes API calls to the Economy Ledger. |
| **IAC / Deployment** | **Terraform** + **GitHub Actions** | Mandatory for managing all cloud resources and continuous delivery pipelines on **Google Cloud Platform (GCP)**. |

### Frontend (Presentation Layer)

| Component | Technologies | Key Role/Purpose |
| :--- | :--- | :--- |
| **Rendering** | `Pixi.js` | Mature, high-performance **2D WebGL/WebGPU renderer**. |
| **Core Visuals** | **Procedural Art** | Agent appearance and movement must be procedurally generated from their **DNA** to ensure unique, lifelike, and emergent animation. |
| **UI** | HTML/DOM | Standard HTML/DOM for all supplemental "chrome" (menus, inventory, HUD) for rapid UX development. |
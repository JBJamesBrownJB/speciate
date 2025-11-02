# Project Spec: "Speciate"
---

## 1. 🎯 Core Concept

A persistent, **server-authoritative artificial life simulation** with a player-driven economy. 
Players are "Influencers" in a shared world populated by hundreds of thousands of autonomous, evolving agents.

The core gameplay loop involves:
* **Indirect Control** (influencing the environment).
* **Survival** (resource management, crafting).
* **Economic Progression**.
* **Technological Progression**.

Players compete to harvest natural resources and "**biomass**" (accrued from cultivating life forms). The ultimate goal is to trigger "**speciation events**," which grant the player ownership of unique "**DNA**" assets, forming the basis of the game's economy.

---

## 2. 🏛️ Architectural Model

We will use a **Headless Server / Thin Client** model. This is **non-negotiable** to secure the game's economy and prevent cheating.

### Server (Authoritative)

* A single, headless **Rust** application.
* It is the **sole source of truth** for all simulation logic, resource ledgers, and player-owned assets.
* Statefull, has its own state to ensure that the simulation can be resumed after server outages, migrations or system downtime.

### Client (Thin Client)

* A **TypeScript** web application responsible **only for rendering and input**.
* It runs **zero game logic**, instead creating a fluid 60fps UX by interpolating server state and predicting player input.

### Economy Ledger (Secure, ACID)

* A database record of all player owned assets fronted by an API.

---

## 3. 🛠️ Technology Stack

### Backend (Headless Server)

| Component | Technologies | Key Role/Purpose |
| :--- | :--- | :--- |
| **Core Logic (ECS)** | `bevy_ecs` + `bevy_app` | High-performance, multi-threaded simulation for agents, running at a **fixed tick rate (e.g., 20Hz)**. |
| **Networking** | `tokio` + `axum` | `tokio` for async I/O; `axum` for serving the client and persistent **WebSocket** state sync. |
| **Database (Persistence)** | `sqlx` + `SQLite` | A single `game.db` file acting as the persistent ledger and ultimate source of truth for player assets. |

### Frontend (Presentation Layer)

| Component | Technologies | Key Role/Purpose |
| :--- | :--- | :--- |
| **Rendering** | `Pixi.js` | Mature, high-performance **2D WebGL renderer**. |
| **UI** | HTML/DOM | Standard HTML/DOM for all UI (inventory, menus, etc.) for rapid development. |
| **Build Tool** | `Vite` | Standard build tool. |

## Economy Ledger Microservice (Decoupled, Secure, ACID) 💰

This is a dedicated microservice solely responsible for all asset management. It is the only component that can directly access the persistence layer.

| Component | Technologies | Key Role/Purpose |
| :--- | :--- | :--- |
| **Service Language** | **Node.js + TypeScript** | The sole language and runtime for the microservice API layer. |
| **Persistence Engine** | **`PostgreSQL`** | The database guaranteeing **ACID properties**, high concurrency, and data integrity for all player assets (Biomass, DNA, etc.). |
| **Interface / API** | **REST** | All external components (including the Simulation Server) interact with the ledger **ONLY** through this API abstraction. |
| **Logic** | **Transactional Logic** | Encapsulates all ACID transactions, resource validation, and integrity checks (Foreign Keys, Constraints) before committing to PostgreSQL. |
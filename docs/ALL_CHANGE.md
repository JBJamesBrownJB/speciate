
> **✅ PROCESSED - 2025-11-10**
>
> This document's content has been dispersed into the project documentation structure:
> - **Business Strategy** → [docs/strategy/biz-strategy.md](./strategy/biz-strategy.md)
> - **Technical Architecture** → [docs/architecture/tauri-architecture.md](./architecture/tauri-architecture.md)
> - **Game Goal (Narrative)** → [docs/strategy/goal.md](./strategy/goal.md)
> - **Taming System** → [docs/gameplay/taming-system.md](./gameplay/taming-system.md)
> - **Drongo Species** → [docs/biology/drongo-species.md](./biology/drongo-species.md)
> - **MMO Architecture** → Marked "Future Vision" in [docs/architecture/streaming-architecture.md](./architecture/streaming-architecture.md)
> - **Archive Documentation** → [docs/architecture/archived/MMO_STREAMING.md](./architecture/archived/MMO_STREAMING.md)
>
> **Project Updates:**
> - [docs/project-spec.md](./project-spec.md) - Updated to Steam EA focus with phase gates
> - [README.md](../README.md) - Rewritten for standalone desktop game
> - [docs/biology/biology-notes.md](./biology/biology-notes.md) - Drongo consultation logged
>
> **Team Feedback Incorporated:**
> - Architect Andy: Lock-free ring buffer, dual-tick architecture
> - PM Pam: Phase-gated approach, scope control (defer story to Phase 1.5)
> - Gamification Garry: Gauntlet challenge (not scripted boss), systemic gameplay
> - Zoologist Tom: Drongo DNA traits validated, biological trade-offs
>
> **Original content preserved below for reference.**

---

The Business Strategy: Steam Early Access
Goal: Prove your core A-Life simulation is fun and engaging before spending time or money on massive server infrastructure. Model: A "pay-once" game sold on Steam. This is your "Phase 1" to build a fanbase and fund "Phase 2" (the web version). Monetization: A one-time purchase (e.g., $20-$30). You are profitable on nearly every sale, as there are no recurring server costs. Risk: You've eliminated the $19,000/month financial risk of a live server and exchanged it for the much smaller, solvable technical risks of local performance and piracy.

The Technical Architecture: The Tauri Hybrid
This architecture uses "best-in-class" tools for each job, leveraging your existing work.
The "Brain" (Rust/Bevy ECS):
This is your core simulation. It runs all the AI, physics, and state logic for your 1,000+ creatures.
It's compiled, fast, and (because it's not on a server) your core logic is safe from being easily stolen.
The "Renderer" (PixiJS):
This is your "face." It's what you already know, it has a massive community, and you've already proven it's fast enough (handling 1,000 sprites at 90 FPS).
It's responsible for all drawing and zooming.
The "Wrapper" (Tauri):
This is the free, open-source "glue" that bundles your Rust backend and your PixiJS frontend into a single desktop app (.exe, .app) that you upload to Steam.

The Data Flow: "Pull" not "Push"
This is the key to making the Tauri bundle efficient and simple. You are not using a network.
The Sim (20 Hz "Brain"): Your Rust sim runs its expensive AI logic (seek_system, etc.) in Bevy's FixedUpdate schedule, running at a fixed 20 Hz. This updates the velocity and state of creatures.
The Renderer (90 Hz "Body"): Your PixiJS app runs at 90 FPS. On every single render frame (app.ticker), it "pulls" the latest state by calling a Rust function: await invoke('get_game_state').
The Movement (90 Hz "Body"): The cheap physics (position += velocity * dt) should also be run in Bevy's main Update loop, so it runs in lockstep with the renderer.
The Benefit: This is massively faster and simpler. You can send full f32 coordinates because bandwidth is irrelevant. All the complex quantization and network interpolation logic is deleted.

Immediate Next Steps
Refactor Bevy Sim (Dual-Tick):
Move expensive "Brain" AI systems (seek_system, flee_system) into the FixedUpdate schedule (set to 20 Hz).
Move cheap "Body" physics (position += velocity * dt) into the main Update loop.
Refactor PixiJS (Remove Interpolation):
Rip out all the lerp() and old_state/new_state logic.
In the 90 FPS app.ticker loop, just get the one true state from Rust and set the sprite positions directly (sprite.x = state.x).
Implement Data Flow:
Set up the Tauri #[tauri::command] (get_game_state) on the Rust side.
Implement the invoke('get_game_state') call inside the PixiJS app.ticker.
Wrap the Bevy World (or a GameState struct) in a Mutex so the Tauri command can safely read the state.
Package & Test:
Wrap the entire project in Tauri and run it as a local .exe to confirm the new lockstep model works and performance is high.

Extra random ideas:
Migrations, herds of crits migrate across your world. Maybe even from other players worlds!
Per player tree of life online, community artifact.
Start game as a person, but once can craft special tech, control your tamed crits. 
Smart species? With hands? That will learn from you!? They are called ‘Drongo’s’, they are sort of humanoid or Australopithecus, they have hands so can do some things the player can do, like hunt, capture, gather resources, even craft rudimentary items. They have a larger tendency to learn from others, including you. The emergence means that taming and growing them around you they will start to appear to ‘help out’. But, they are weak and usually don’t survive long, get predated easy, starve easy. So, when you find them, you should protect them, because they become very useful.

Various ways to tame, individual harpoon capture, tame zone beacons tech, biomass DNA research with cloning high tech.

There is fog of war, so hard to find your daughter. Various ways to uncover fog of war. Harpoon tracking tech to a crit, let it explore with you.
You can and must command your grown creature hordes, example “Tech Thumper” call and launch stream of guardian crits against the danger. Picture your avatar raising the spear like totem and plunging into the ground sending a wave of crits against an enemy.

The goal!!!
You crash landed on an alien planet, your cryo pod opened on one side of the planet, your daughter's on landed on the other, protected by her cryopod, you must get to her. But distance, harsh terrain, even harsher ecosystem and her landing site has progressively hard predators, sustained by old tech which feeds them without them needing to survive and of course, the Karg! A giant monster, the boss!!! 

But first u must survive yourself, even find her location 
The final scene of entering the lair should feel emotional and exciting, listen to Contact - Daft Punk for inspiration. Picture your player riding in on an armoured steed, surrounded by a herd of tamed protectors, firing off tech bangs and force Fields, you can't defeat the Karg but you can save your daughter!!!!


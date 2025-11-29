## Tech Upgrades
- Only for paid customers ultimatley

### High Altitute Drone
This is a premium upgrade (available to paid customers) unlocked via crafting. The Drone item permanently enhances the player's camera controls, allowing them to access a new high-altitude "Strategic View." This mode allows players to zoom out far beyond the standard camera limits, providing a comprehensive, real-time overview of the entire map and the large-scale movements of entities within it.

#### Player Experience & Visuals

    Extended Zoom: After crafting the Drone, using the mouse wheel to zoom out will seamlessly transition the camera past its normal limit, ascending to a "super high altitude."

    Symbolic Data Display: To maintain performance while displaying thousands of entities, all creatures ("crits") in this view are represented by simple, color-coded circles (or icons).

    Color: Indicates the entity type (e.g., Red = Carnivore, Green = Herbivore, Yellow = Scavenger).

    Size: Indicates the relative size or power of the entity.

    UI Mode Indicator: When Strategic View is active, the glowing border of the viewport (the UI vignette) will change to a distinct color (e.g., a "digital" blue) to provide clear visual feedback that the player is in 'Drone Mode'.

#### Technical Architecture & Optimization

    Dynamic Data Broadcast: To ensure smooth performance at high altitude, the game client switches its data subscription model.

    Optimized Payload: Instead of receiving full entity data, the UI will only broadcast an optimized, low-bandwidth payload for entities in this view. This payload is limited to:

        X, Y Coordinates

        Entity Type (Carnivore, Herbivore, etc.)

        Size

    High-Performance Rendering: This dramatically shorter payload allows the client to render a massive number of entities as simple, colored shapes without the performance cost of full models, animations, or detailed stats.
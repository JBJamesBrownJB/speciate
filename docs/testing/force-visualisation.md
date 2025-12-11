2. Force Vector Visualization (The Data Plane)

Objective: Visualize the invisible steering forces driving a specific creature's behavior to verify AI logic (e.g., "Why is it turning left?").

    Rust Backend (Instrumentation):

        Component: Introduce a ForceDebug component containing vector fields for specific behaviors: seek, flee, separation, wander, and net_force.

        System Update: Modify the steering systems (running at 20Hz) to populate this component with the raw calculated vectors during the decision-making process.

        Targeted Serialization: To conserve bandwidth, only serialize and send the ForceDebug data for the Selected Entity (the creature currently clicked/inspected). Do not send this for the whole swarm.

    IPC Pipeline:

        The ForceDebug data travels attached to the EntitySnapshot via the existing sim:state channel.

    Frontend Rendering (Visualization):

        Vector Layer: Create a specialized ForceRenderer in PixiJS that sits above the creature sprite layer.

        Drawing Logic:

            Read the vectors from the snapshot.

            Draw lines originating from the creature's center point.

            Color Coding: Use semantic colors (e.g., Green for Seek/Desire, Red for Flee/Separation, Blue for the final Velocity).

            Visual Scaling: Multiply the raw vector magnitude by a visualization constant (e.g., x10) so small steering forces are visible as discernable lines on screen.
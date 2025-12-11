The Debug Toggle Architecture (The Control Plane)

Objective: Create a unified, responsive interface in the Frontend (Dev UI) to manage which debug layers are currently active, without overloading the IPC channel or the rendering loop.

    State Management (Frontend):

        Implement a centralized Debug Store (using Zustand or React Context) to hold boolean flags for visual features (e.g., showFOV, showForces, showSpatialGrid).

        This store acts as the single source of truth for the visibility of all debug layers in the PixiJS canvas.

    The "Cockpit" UI:

        Add a visual control bar to the Dev UI containing toggle switches or icon buttons for each debug feature.

        Interaction: Clicking a toggle updates the Debug Store immediately.

    Performance Optimization (Lazy Rendering):

        The PixiJS rendering loop checks these flags before attempting to draw any debug geometry.

        If showSpatialGrid is false, the rendering system for the grid returns early, saving CPU/GPU cycles on the frontend.

    Backend Synchronization (Optional but Recommended):

        If a specific debug feature requires expensive calculation on the Rust side (e.g., raycasting for detailed vision), the Frontend sends a DevConfigUpdate IPC message to the Backend to enable/disable that computation logic to save simulation ticks.
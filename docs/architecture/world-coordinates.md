## World Coordinates Strategy

## Core Decisions
- Type: `f32` (a standard float). It's fast, efficient, and the industry standard.
- Backend Rate: `20 Hz`. The server is the "truth" and calculates physics 20 times per second.
- Frontend Rate: `60+ FPS`. The browser portal renders as fast as possible to look smooth.
- Scale: `1.0 unit = 1 meter`. This keeps the math simple and human-readable.

## World Size and Precision
- The Limit: An `f32` starts to lose precision as numbers get huge (it only tracks about 7 significant digits).
- Our Limit: We will cap the world at `+/- 1,000,000 meters` from the center (0, 0).
- Total Size: This gives us a massive `2,000 km x 2,000 km` map.
- Precision: At this max range, we are still accurate to about `0.1 meters` (10 cm), which is perfect and won't cause any visual jitter.
- Future: If we ever need to go bigger, we'd use a technique called "Origin Rebasing" (shifting the world), but we don't need it for this plan.

## Backend (The Server)
- Job: Runs the simulation, calculates all creature physics, and acts as the single source of truth.
- Physics: Must use `f32`. Using integers for position would break the physics, as fractional movement (e.g., 0.5 meters) would be lost.
- Broadcast: At 20 Hz, it sends out an array of all creatures, with all detail needed for the portal to render postion, size, orientation...

## Frontend (The PixiJS Portal)
- Job: Renders the world at 60+ FPS, handles all user input (camera/zoom), and smooths out the 20 Hz backend updates.

## Frontend: Camera and Zoom
- Camera: When the user presses WASD/arrows, they are moving a "camera coordinate" `(Camera_X, Camera_Y)` around the world in meters.
- Zoom: When the user scrolls, they change a zoom ratio `Z` (pixels per meter). The zoom value represents how many pixels represent one meter.
- Rendering: PixiJS will move and scale one main "world" container. Its position and scale are set based on the user's camera coordinates and zoom level.
- Zoom Limits:
  - Min Zoom: 0.0005 px/m (1 pixel = 2000 meters, entire 2000km world fits on screen)
  - Max Zoom: 200 px/m (1 meter = 200 pixels, extreme close-up view)

## Frontend: Smoothing (Interpolation)
- The Problem: The backend only sends updates every 50ms (20 Hz), which would look very "jerky" on its own.
- The Solution: We must smooth this out. The frontend will store the *last two* positions (`old` and `new`) for every creature and use engine features like lerp or something better to smooth it.
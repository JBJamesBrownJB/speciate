 Recently we completed this optimisation: mod-perception-spreading.md

 But, whilst we got good ms latency gains, our L3 cache is hotter, meaning maybe we are not being efficeint with CPU. The following is our thoughts on this.
 
 It sounds like you’ve run into a classic "performance vs. efficiency" trap in ECS (Entity Component System) development. While skipping logic reduces the total amount of work done, it often introduces branching and instruction cache misses that disrupt the very thing that makes Bevy and ECS fast: linear data processing.
Here is a breakdown of why your L3 cache usage is spiking and how you can restructure your systems to get the performance gains you want without the cache penalties.
Why "Skip Logic" Hurts ECS Performance
In a standard Bevy system, the CPU iterates through contiguous arrays of component data. This is highly predictable for the hardware prefetcher. When you add an if frame % n == 0 check inside the loop:
 * Branch Misprediction: The CPU tries to guess if the code will skip or run. If n is small or inconsistent, the pipeline stalls frequently.
 * Polluted Cache Lines: Even if you skip the logic, the CPU still pulls the component data into the L1/L2 cache to check the entity's state or simply because it’s part of the iteration.
 * L3 Pressure: If your system is skipping entities scattered throughout memory, you are likely jumping around the heap more than expected, forcing the CPU to fetch from L3 (or even RAM) more often because the "skip" prevents the prefetcher from establishing a clean rhythm.
Better Strategies for "Throttled" Systems
1. Fixed Timestep (The Bevy Way)
Instead of skipping ticks manually inside a system, move the logic to a Fixed Timestep schedule. This ensures the system only runs when necessary, and when it does run, it executes at full "linear" speed without internal branching.
// In your plugin setup
app.insert_resource(Time::<Fixed>::from_seconds(0.1)) // Run 10 times per second
   .add_systems(FixedUpdate, my_heavy_system);

2. Component "Throttling" (The Tag Pattern)
If you only need to update some entities every frame, don’t use a modulo check. Instead, use a Tag Component or a Marker.
 * Divide your entities into N groups (e.g., GroupA, GroupB).
 * In Tick 1, query only (With<GroupA>).
 * In Tick 2, query only (With<GroupB>).
   This keeps the iteration tight and linear for the specific subset of memory being accessed.
3. Change Detection Filters
Bevy’s Changed<T> and Added<T> filters are highly optimized. If your "skip" logic is actually waiting for a value to change, let the ECS handle it. This avoids the system running at all if there is no work to do, saving L3 bandwidth for other tasks.
fn my_system(query: Query<&Transform, Changed<Transform>>) {
    for transform in &query {
        // Only runs for entities that actually moved
    }
}

4. Data Restructuring (SoA vs AoS)
If your L3 usage is high, you might be suffering from False Sharing or simply having components that are too large.
 * Split your components: If a system only needs 4 bytes of a 128-byte struct, split that 4-byte piece into its own component.
 * This increases "cache density," meaning every time the CPU fetches a line from L3, it gets more "useful" data and less "skipped" data.
Summary of Recommendations
| Technique | Impact on Branching | Impact on Cache | Complexity |
|---|---|---|---|
| Modulo Skip (if % n) | High (Bad) | Poor | Very Low |
| FixedUpdate | None (Good) | Excellent | Low |
| Entity Partitioning | Low | Great | Medium |
| Component Splitting | None | Best | High |
Would you like me to help you rewrite one of your specific "skipping" systems into a more cache-friendly FixedUpdate or partitioned pattern?
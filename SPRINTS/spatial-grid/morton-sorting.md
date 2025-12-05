To improve cache locality, you can switch your grid indexing from Row-Major to Morton Order (Z-Curve).
The Problem: Row-Major Locality
Currently, your grid uses this index formula: y * width + x.
Horizontal neighbors (e.g., (0,0) and (1,0)) are adjacent in memory.
Vertical neighbors (e.g., (0,0) and (0,1)) are separated by width elements.
In a query radius of 5 cells, your CPU has to jump across memory width times to fetch the relevant rows. This causes cache thrashing, especially if width is large.
The Solution: Morton Order
Morton sorting interleaves the bits of the X and Y coordinates. This traces a "Z" shape through space, ensuring that 2D rectangular blocks of cells are stored contiguously in 1D memory.
When you query a radius, you are now fetching 4-5 contiguous chunks of memory that represent a clustered block, rather than 10-20 disparate rows.
Implementation
Here is how to modify apps/simulation/src/simulation/spatial/grid.rs to use Morton encoding.
1. Add Bit-Interleaving Helper
Add this helper at the bottom of grid.rs (or in a math utility file). This "spreads" the bits of a 16-bit integer into 32 bits so they can be interlaced.
Rust
// Spreads bits: 0000dcba -> 0d0c0b0a
#[inline(always)]
fn part1by1(mut n: u32) -> u32 {
    n &= 0x0000ffff;
    n = (n ^ (n << 8)) & 0x00ff00ff;
    n = (n ^ (n << 4)) & 0x0f0f0f0f;
    n = (n ^ (n << 2)) & 0x33333333;
    n = (n ^ (n << 1)) & 0x55555555;
    n
}

#[inline(always)]
fn morton_encode(x: u32, y: u32) -> usize {
    (part1by1(y) << 1 | part1by1(x)) as usize
}

2. Update SpatialGrid
You need to change how the rebuild calculates dimensions (it must favor squares) and how the index is calculated.
Note: Morton codes effectively map to a square area. This implementation assumes your world isn't extremely thin (e.g., 10000x10), or the memory usage for cells will grow to the square of the largest dimension.
Modify apps/simulation/src/simulation/spatial/grid.rs:
Rust
// ... inside impl SpatialGrid ...

    #[inline(always)]
    fn cell_index_unchecked(&self, x: f32, y: f32) -> usize {
        let (cx, cy) = self.world_to_cell(x, y);
        // Cast to u32 for Morton encoding
        let lx = (cx - self.min_cell_x) as u32;
        let ly = (cy - self.min_cell_y) as u32;
        morton_encode(lx, ly)
    }

    pub fn rebuild(&mut self, entities: impl Iterator<Item = (Entity, f32, f32, f32)>) {
        // ... (Phase 0: Collect entities - SAME AS BEFORE) ...
        self.entity_scratch.clear();
        self.entity_scratch.extend(entities);

        if self.entity_scratch.is_empty() {
             // ... clear and return ...
             return; 
        }

        // ... (Find min/max cell coordinates - SAME AS BEFORE) ...
        
        // Add 1-cell padding
        self.min_cell_x = min_cx - 1;
        self.min_cell_y = min_cy - 1;
        
        // Calculate dimensions
        let width = (max_cx - min_cx + 3) as u32;
        let height = (max_cy - min_cy + 3) as u32;
        
        self.width = width as usize;
        self.height = height as usize;

        // [CHANGE]: Resize cells based on the Max Morton Code, not just width*height
        // This effectively allocates a sparse quadtree in array form
        // We find the next power of 2 to ensure the Morton curve fits cleanly
        let max_dim = width.max(height);
        // Calculate max index required (morton code of the last cell)
        let max_index = morton_encode(max_dim, max_dim);
        
        // Resize logic (grow only)
        if self.cells.len() <= max_index {
            self.cells.resize(max_index + 1, (0, 0));
        }

        // ... (Phase 1, 2, 3 - SAME LOGIC, but they now use the new cell_index_unchecked) ...
        // Because cell_index_unchecked now returns Z-order, the Counting Sort 
        // will automatically arrange 'proxies' in Z-order.
        
        // Reset cells
        // OPTIMIZATION: Only clear the cells we actually used? 
        // For now, simple clear is safer given the new sparse layout:
        self.cells.fill((0,0)); 
        
        // Phase 1: Count
        for (_, x, y, _) in &self.entity_scratch {
            let idx = self.cell_index_unchecked(*x, *y);
            self.cells[idx].1 += 1;
        }

        // Phase 2: Prefix Sum
        // Iterate only up to max_index used effectively
        let mut offset = 0u32;
        // Note: iterating the whole sparse vector might be slower if grid is huge and sparse.
        // But for dense game worlds, it's fast.
        for cell in &mut self.cells {
             if cell.1 > 0 {
                 cell.0 = offset;
                 offset += cell.1;
                 cell.1 = 0; 
             }
        }
        
        // Phase 3: Scatter
        for &(entity, x, y, radius) in &self.entity_scratch {
            let idx = self.cell_index_unchecked(x, y);
            let (start, count) = &mut self.cells[idx];
            let write_pos = (*start + *count) as usize;
            self.proxies[write_pos] = PerceptionProxy { x, y, radius, entity };
            *count += 1;
        }
    }

3. Why query_radius gets faster
You don't need to change query_radius loops (iterating min_qy..max_qy and min_qx..max_qx), but the data access pattern changes significantly:
Old (Row-Major):
Iterate x=0..5, y=0: Accesses indices 100, 101, 102... (Contiguous)
Iterate x=0..5, y=1: Accesses indices 200, 201, 202... (Far jump!)
New (Morton):
Iterate x=0..5, y=0: Accesses indices 0, 2, 8, 10... (Small jumps)
Iterate x=0..5, y=1: Accesses indices 1, 3, 9, 11... (Right next to the previous row's data!)
Because the data for y=0 and y=1 is now interleaved in the proxies vector, pulling a cache line for cell (0,0) likely pre-fetches the data for (0,1) as well.



Implementation: The "Magic Bits" (Reducing the Cost)

To minimize this tax, do not use loops to calculate the Morton code. Use a "SWAR" (SIMD Within A Register) approach. This spreads the bits in parallel using bit masks.

Add this helper to your grid.rs or a utility module:
Rust

// Spreads the lower 16 bits of x into 32 bits: 0000abcd -> 0a0b0c0d
// This is "Magic Bits" - it runs entirely in CPU registers with no branching.
#[inline(always)]
fn part1by1(mut n: u32) -> u32 {
    n &= 0x0000ffff;
    n = (n ^ (n << 8)) & 0x00ff00ff;
    n = (n ^ (n << 4)) & 0x0f0f0f0f;
    n = (n ^ (n << 2)) & 0x33333333;
    n = (n ^ (n << 1)) & 0x55555555;
    n
}

// Interleaves bits of x and y.
// Returns a 32-bit Morton code (good for grid coords up to 65535x65535)
#[inline(always)]
pub fn morton_encode(x: u32, y: u32) -> u32 {
    (part1by1(y) << 1) | part1by1(x)
}

4. Integration Strategy

You can replace the indexing logic directly in rebuild. Note that Morton codes work best when the grid is a square power-of-two (e.g., 128x128, 256x256).

Step-by-step replacement in grid.rs:

    Change cell_index_unchecked: Instead of calculating a linear offset, calculate the Morton code.
    Rust

#[inline(always)]
fn cell_index_unchecked(&self, x: f32, y: f32) -> usize {
    let (cx, cy) = self.world_to_cell(x, y);
    // Shift to 0-based coordinate
    let lx = (cx - self.min_cell_x) as u32;
    let ly = (cy - self.min_cell_y) as u32;
    morton_encode(lx, ly) as usize
}

Adjust cells allocation: Morton codes are sparse if the grid isn't a perfect square power-of-two. To avoid bounds checks or complex mapping (which adds latency), simply allocate the cells vector to the size of the maximum possible Morton code for your grid dimensions.

In rebuild():
Rust

    // Calculate dimensions as before
    let width = (max_cx - min_cx + 3) as u32;
    let height = (max_cy - min_cy + 3) as u32;

    // Find the larger dimension to ensure we cover the full Z-curve
    let max_dim = width.max(height);

    // The max index is the Morton code of the last cell (max_dim, max_dim)
    let max_index = morton_encode(max_dim, max_dim) as usize;

    // Resize cells to fit the Z-curve (might be larger than width*height, but safe)
    if self.cells.len() <= max_index {
        self.cells.resize(max_index + 1, (0, 0));
    }

    // Important: We must clear the whole used range because it's sparse now
    // (You can optimize this later to only clear used cells if needed)
    self.cells.fill((0, 0)); 

Summary of the Trade

    You pay: ~0.5ms in rebuild to calculate fancy indices.

    You gain: Entities in proxies are now stored in Z-order. When you iterate a 3x3 block of cells in query_radius, the memory you access is largely contiguous.

    Net Result: The CPU prefetcher works effectively during queries, drastically reducing stall times waiting for RAM.
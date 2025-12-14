#!/bin/bash
# Analyze component memory layout and identify optimization opportunities

cat << 'EOF' > /tmp/size_check.rs
use std::mem::size_of;

// Simulate component sizes
#[repr(C)]
struct Entity(u64);

#[repr(C)]
struct Position { x: f32, y: f32 }

#[repr(C)]
struct Velocity { vx: f32, vy: f32 }

#[repr(C)]
struct Acceleration { ax: f32, ay: f32 }

#[repr(C)]
struct BodySize { length: f32, inv_sqrt_length: f32, mass: f32 }

#[repr(C)]
struct Rotation { radians: f32 }

#[repr(C)]
struct NeighborData {
    entity: Entity,
    x: f32,
    y: f32,
    radius: f32,
}

#[repr(C)]
struct Perception {
    fov_angle: f32,
    range: f32,
    cos_half_fov_sq: f32,
    neighbor_count: u8,
    neighbors: [NeighborData; 7],
}

fn main() {
    println!("========================================");
    println!("Component Size Analysis");
    println!("========================================");
    println!();
    println!("HOT Components (read every tick):");
    println!("  Position:      {:3} bytes", size_of::<Position>());
    println!("  Velocity:      {:3} bytes", size_of::<Velocity>());
    println!("  Acceleration:  {:3} bytes", size_of::<Acceleration>());
    println!("  BodySize:      {:3} bytes", size_of::<BodySize>());
    println!("  Rotation:      {:3} bytes", size_of::<Rotation>());
    println!("  Perception:    {:3} bytes ⚠️  BLOATED!", size_of::<Perception>());
    println!();
    println!("COLD Data (nested in Perception):");
    println!("  NeighborData:  {:3} bytes", size_of::<NeighborData>());
    println!("  Array[7]:      {:3} bytes", size_of::<[NeighborData; 7]>());
    println!();
    println!("Cache Analysis:");
    let cacheline_size = 64;
    let perception_cachelines = (size_of::<Perception>() + cacheline_size - 1) / cacheline_size;
    println!("  Cacheline size: {} bytes", cacheline_size);
    println!("  Perception spans: {} cachelines", perception_cachelines);
    println!();
    println!("Impact @ 200K creatures:");
    let creature_count = 200_000;
    let total_mb = (creature_count * size_of::<Perception>()) / (1024 * 1024);
    println!("  Total Perception memory: {} MB", total_mb);
    println!("  L1 cache (32 KB) holds: {} components", 32 * 1024 / size_of::<Perception>());
    println!("  L3 cache (36 MB) holds: {} components", 36 * 1024 * 1024 / size_of::<Perception>());
    println!();
    println!("Proposed Split:");
    let perception_hot = 16; // fov_angle + range + cos_half_fov_sq + neighbor_count + padding
    let neighbor_cache_cold = size_of::<[NeighborData; 7]>();
    println!("  Perception (hot):     {:3} bytes (1 cacheline)", perception_hot);
    println!("  NeighborCache (cold): {:3} bytes (3 cachelines)", neighbor_cache_cold);
    println!("  Savings in hot path:  {:3} bytes per creature", size_of::<Perception>() - perception_hot);
    println!("  Total savings:        {} MB @ 200K creatures",
             ((size_of::<Perception>() - perception_hot) * creature_count) / (1024 * 1024));
}
EOF

rustc /tmp/size_check.rs -o /tmp/size_check && /tmp/size_check
rm -f /tmp/size_check.rs /tmp/size_check

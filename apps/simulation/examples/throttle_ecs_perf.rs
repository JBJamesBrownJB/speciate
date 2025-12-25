use bevy_ecs::prelude::*;
use std::time::Instant;

const ENTITY_COUNT: usize = 200_000;
const DIVISOR: usize = 8;
const TICKS: usize = 100;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct UpdateTicket(u8);

fn approach_a_bitwise(
    tick: u64,
    divisor: usize,
    query: &mut Query<(Entity, &mut Position)>,
) -> usize {
    let bucket_mask = divisor - 1;
    let current_bucket = (tick as usize) & bucket_mask;

    let mut processed = 0;
    for (entity, mut pos) in query.iter_mut() {
        if (entity.index() as usize) & bucket_mask != current_bucket {
            continue;
        }
        pos.x += 1.0;
        processed += 1;
    }
    processed
}

fn approach_b_ticket(
    tick: u64,
    divisor: usize,
    query: &mut Query<(&UpdateTicket, &mut Position)>,
) -> usize {
    let current_bucket = (tick as usize) % divisor;

    let mut processed = 0;
    for (ticket, mut pos) in query.iter_mut() {
        if ticket.0 as usize != current_bucket {
            continue;
        }
        pos.x += 1.0;
        processed += 1;
    }
    processed
}

fn approach_c_modulo(
    tick: u64,
    divisor: usize,
    query: &mut Query<(Entity, &mut Position)>,
) -> usize {
    let current_bucket = (tick as usize) % divisor;

    let mut processed = 0;
    for (entity, mut pos) in query.iter_mut() {
        if (entity.index() as usize) % divisor != current_bucket {
            continue;
        }
        pos.x += 1.0;
        processed += 1;
    }
    processed
}

fn main() {
    println!("=== ECS Throttle Performance Comparison ===");
    println!("Entity Count: {}", ENTITY_COUNT);
    println!("Divisor: {}", DIVISOR);
    println!("Ticks: {}", TICKS);
    println!();

    let mut world_a = World::new();
    let mut world_b = World::new();
    let mut world_c = World::new();

    println!("Spawning {} entities in 3 separate worlds...", ENTITY_COUNT);

    for i in 0..ENTITY_COUNT {
        world_a.spawn(Position { x: 0.0, y: 0.0 });

        world_b.spawn((
            Position { x: 0.0, y: 0.0 },
            UpdateTicket((i % DIVISOR) as u8),
        ));

        world_c.spawn(Position { x: 0.0, y: 0.0 });
    }

    println!("Warmup...");
    let mut query_a = world_a.query::<(Entity, &mut Position)>();
    let mut query_b = world_b.query::<(&UpdateTicket, &mut Position)>();
    let mut query_c = world_c.query::<(Entity, &mut Position)>();

    for tick in 0..10 {
        approach_a_bitwise(tick, DIVISOR, &mut query_a);
        approach_b_ticket(tick, DIVISOR, &mut query_b);
        approach_c_modulo(tick, DIVISOR, &mut query_c);
    }

    println!();
    println!("--- Approach A: Bitwise AND (entity.index() & mask) ---");
    let start = Instant::now();
    let mut total_processed = 0;
    for tick in 0..TICKS {
        total_processed += approach_a_bitwise(tick as u64, DIVISOR, &mut query_a);
    }
    let time_a = start.elapsed();
    println!("Total time: {:?}", time_a);
    println!("Per tick: {:?}", time_a / TICKS as u32);
    println!("Entities/tick: {}", total_processed / TICKS);

    println!();
    println!("--- Approach B: Ticket Component (memory load) ---");
    let start = Instant::now();
    let mut total_processed = 0;
    for tick in 0..TICKS {
        total_processed += approach_b_ticket(tick as u64, DIVISOR, &mut query_b);
    }
    let time_b = start.elapsed();
    println!("Total time: {:?}", time_b);
    println!("Per tick: {:?}", time_b / TICKS as u32);
    println!("Entities/tick: {}", total_processed / TICKS);

    println!();
    println!("--- Approach C: Modulo (entity.index() % divisor) ---");
    let start = Instant::now();
    let mut total_processed = 0;
    for tick in 0..TICKS {
        total_processed += approach_c_modulo(tick as u64, DIVISOR, &mut query_c);
    }
    let time_c = start.elapsed();
    println!("Total time: {:?}", time_c);
    println!("Per tick: {:?}", time_c / TICKS as u32);
    println!("Entities/tick: {}", total_processed / TICKS);

    println!();
    println!("=== Results ===");
    println!("A (Bitwise):  {:?}/tick", time_a / TICKS as u32);
    println!("B (Ticket):   {:?}/tick", time_b / TICKS as u32);
    println!("C (Modulo):   {:?}/tick", time_c / TICKS as u32);

    let times = vec![
        ("Bitwise AND", time_a),
        ("Ticket Component", time_b),
        ("Modulo", time_c),
    ];

    let (winner_name, winner_time) = times.iter().min_by_key(|(_, t)| t).unwrap();

    println!();
    println!("Winner: {}", winner_name);
    for (name, time) in &times {
        if name != winner_name {
            let ratio = time.as_nanos() as f64 / winner_time.as_nanos() as f64;
            println!("  {:.2}x faster than {}", ratio, name);
        }
    }

    println!();
    println!("Memory overhead:");
    println!("  Bitwise:  0 bytes (uses Entity struct already in query)");
    println!("  Ticket:   {} KB (1 byte × {} entities)", ENTITY_COUNT / 1024, ENTITY_COUNT);
    println!("  Modulo:   0 bytes (uses Entity struct already in query)");
}

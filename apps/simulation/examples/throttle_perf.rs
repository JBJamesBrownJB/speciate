use std::time::Instant;

const ENTITY_COUNT: usize = 200_000;
const DIVISOR: usize = 8;
const ITERATIONS: usize = 100;

fn bitwise_throttle(entity_ids: &[u32], tick: u64, divisor: usize) -> usize {
    let bucket_mask = divisor - 1;
    let current_bucket = (tick as usize) & bucket_mask;

    let mut processed = 0;
    for &entity_id in entity_ids {
        if (entity_id as usize) & bucket_mask != current_bucket {
            continue;
        }
        processed += 1;
    }
    processed
}

fn modulo_throttle(entity_ids: &[u32], tick: u64, divisor: usize) -> usize {
    let current_bucket = (tick as usize) % divisor;

    let mut processed = 0;
    for &entity_id in entity_ids {
        if (entity_id as usize) % divisor != current_bucket {
            continue;
        }
        processed += 1;
    }
    processed
}

fn ticket_throttle(tickets: &[u8], tick: u64, divisor: usize) -> usize {
    let current_bucket = (tick as usize) % divisor;

    let mut processed = 0;
    for &ticket in tickets {
        if ticket as usize != current_bucket {
            continue;
        }
        processed += 1;
    }
    processed
}

fn main() {
    println!("Throttle Performance Comparison");
    println!("================================");
    println!("Entity Count: {}", ENTITY_COUNT);
    println!("Divisor: {}", DIVISOR);
    println!("Iterations: {}", ITERATIONS);
    println!();

    let entity_ids: Vec<u32> = (0..ENTITY_COUNT as u32).collect();
    let tickets: Vec<u8> = (0..ENTITY_COUNT).map(|i| (i % DIVISOR) as u8).collect();

    let tick = 42;

    println!("Warming up...");
    for _ in 0..10 {
        bitwise_throttle(&entity_ids, tick, DIVISOR);
        modulo_throttle(&entity_ids, tick, DIVISOR);
        ticket_throttle(&tickets, tick, DIVISOR);
    }

    println!("\n--- Bitwise AND (power-of-2 only) ---");
    let start = Instant::now();
    let mut total = 0;
    for i in 0..ITERATIONS {
        total += bitwise_throttle(&entity_ids, tick + i as u64, DIVISOR);
    }
    let bitwise_time = start.elapsed();
    println!("Total time: {:?}", bitwise_time);
    println!("Per iteration: {:?}", bitwise_time / ITERATIONS as u32);
    println!("Processed: {} entities/iter", total / ITERATIONS);

    println!("\n--- Modulo (current implementation) ---");
    let start = Instant::now();
    let mut total = 0;
    for i in 0..ITERATIONS {
        total += modulo_throttle(&entity_ids, tick + i as u64, DIVISOR);
    }
    let modulo_time = start.elapsed();
    println!("Total time: {:?}", modulo_time);
    println!("Per iteration: {:?}", modulo_time / ITERATIONS as u32);
    println!("Processed: {} entities/iter", total / ITERATIONS);

    println!("\n--- Ticket Component (memory load) ---");
    let start = Instant::now();
    let mut total = 0;
    for i in 0..ITERATIONS {
        total += ticket_throttle(&tickets, tick + i as u64, DIVISOR);
    }
    let ticket_time = start.elapsed();
    println!("Total time: {:?}", ticket_time);
    println!("Per iteration: {:?}", ticket_time / ITERATIONS as u32);
    println!("Processed: {} entities/iter", total / ITERATIONS);

    println!("\n=== Summary ===");
    println!("Bitwise AND:      {:?}/iter", bitwise_time / ITERATIONS as u32);
    println!("Modulo:           {:?}/iter", modulo_time / ITERATIONS as u32);
    println!("Ticket Component: {:?}/iter", ticket_time / ITERATIONS as u32);

    if bitwise_time < modulo_time && bitwise_time < ticket_time {
        println!("\nWinner: Bitwise AND");
        println!("  vs Modulo:  {:.2}x faster", modulo_time.as_nanos() as f64 / bitwise_time.as_nanos() as f64);
        println!("  vs Ticket:  {:.2}x faster", ticket_time.as_nanos() as f64 / bitwise_time.as_nanos() as f64);
    } else if modulo_time < bitwise_time && modulo_time < ticket_time {
        println!("\nWinner: Modulo");
        println!("  vs Bitwise: {:.2}x faster", bitwise_time.as_nanos() as f64 / modulo_time.as_nanos() as f64);
        println!("  vs Ticket:  {:.2}x faster", ticket_time.as_nanos() as f64 / modulo_time.as_nanos() as f64);
    } else {
        println!("\nWinner: Ticket Component");
        println!("  vs Bitwise: {:.2}x faster", bitwise_time.as_nanos() as f64 / ticket_time.as_nanos() as f64);
        println!("  vs Modulo:  {:.2}x faster", modulo_time.as_nanos() as f64 / ticket_time.as_nanos() as f64);
    }
}

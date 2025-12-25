use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Simulates the bitwise AND approach (power-of-2 divisor)
fn bitwise_throttle(entity_ids: &[u32], tick: u64, divisor: usize) -> usize {
    let bucket_mask = divisor - 1;
    let current_bucket = (tick as usize) & bucket_mask;

    let mut processed = 0;
    for &entity_id in entity_ids {
        if (entity_id as usize) & bucket_mask != current_bucket {
            continue;
        }
        processed += 1;
        black_box(processed); // Simulate work
    }
    processed
}

/// Simulates the modulo approach (current implementation)
fn modulo_throttle(entity_ids: &[u32], tick: u64, divisor: usize) -> usize {
    let current_bucket = (tick as usize) % divisor;

    let mut processed = 0;
    for &entity_id in entity_ids {
        if (entity_id as usize) % divisor != current_bucket {
            continue;
        }
        processed += 1;
        black_box(processed); // Simulate work
    }
    processed
}

/// Simulates the ticket component approach (memory load)
fn ticket_throttle(tickets: &[u8], tick: u64, divisor: usize) -> usize {
    let current_bucket = (tick as usize) % divisor;

    let mut processed = 0;
    for &ticket in tickets {
        if ticket as usize != current_bucket {
            continue;
        }
        processed += 1;
        black_box(processed); // Simulate work
    }
    processed
}

fn bench_throttle_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("throttle_comparison");

    // Test with realistic entity counts and divisors
    let entity_counts = vec![10_000, 50_000, 200_000];
    let divisors = vec![2, 4, 8, 16];

    for entity_count in entity_counts {
        // Generate entity IDs (sequential, like Bevy allocates them)
        let entity_ids: Vec<u32> = (0..entity_count).collect();

        // Pre-assign tickets (assigned during spawn)
        let mut tickets = Vec::with_capacity(entity_count);
        for i in 0..entity_count {
            tickets.push((i % 16) as u8); // Max divisor of 16
        }

        for &divisor in &divisors {
            let tick = 42; // Arbitrary tick

            // Benchmark bitwise AND (power-of-2 only)
            if divisor.is_power_of_two() {
                group.bench_with_input(
                    BenchmarkId::new("bitwise_and", format!("{}e/{}", entity_count/1000, divisor)),
                    &(entity_ids.as_slice(), tick, divisor),
                    |b, &(ids, tick, div)| {
                        b.iter(|| bitwise_throttle(ids, tick, div))
                    },
                );
            }

            // Benchmark modulo (works for any divisor)
            group.bench_with_input(
                BenchmarkId::new("modulo", format!("{}e/{}", entity_count/1000, divisor)),
                &(entity_ids.as_slice(), tick, divisor),
                |b, &(ids, tick, div)| {
                    b.iter(|| modulo_throttle(ids, tick, div))
                },
            );

            // Benchmark ticket (memory load)
            group.bench_with_input(
                BenchmarkId::new("ticket", format!("{}e/{}", entity_count/1000, divisor)),
                &(tickets.as_slice(), tick, divisor),
                |b, &(tix, tick, div)| {
                    b.iter(|| ticket_throttle(tix, tick, div))
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_throttle_methods);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use crate::bench_lab::world::{build_world, Distribution, WorldSpec};
    use crate::simulation::spatial::constants::CELL_SIZE;
    use std::collections::HashMap;

    fn cell(x: f32, y: f32) -> (i32, i32) {
        ((x / CELL_SIZE).floor() as i32, (y / CELL_SIZE).floor() as i32)
    }

    fn measure(population: usize) {
        let spec = WorldSpec {
            population,
            seed: 1,
            half_extent_x: 5000.0,
            half_extent_y: 5000.0,
            distribution: Distribution::Uniform,
        };
        let mut sim = build_world(&spec);
        for _ in 0..15 {
            sim.update(0.05);
        }
        let mut prev: HashMap<u32, (i32, i32)> = sim
            .snapshot_creatures()
            .iter()
            .map(|c| (c.0, cell(c.1, c.2)))
            .collect();

        let mut total_crossed: u64 = 0;
        let mut total: u64 = 0;
        let ticks = 20;
        for t in 0..ticks {
            sim.update(0.05);
            let cur = sim.snapshot_creatures();
            let mut crossed: u64 = 0;
            for c in &cur {
                let cl = cell(c.1, c.2);
                if prev.get(&c.0) != Some(&cl) {
                    crossed += 1;
                }
                prev.insert(c.0, cl);
            }
            total_crossed += crossed;
            total += cur.len() as u64;
            eprintln!(
                "pop={population} tick {t}: crossed {crossed}/{} = {:.1}%",
                cur.len(),
                100.0 * crossed as f64 / cur.len() as f64
            );
        }
        eprintln!(
            "=== pop={population}: AVG cell-crossing rate = {:.2}% per tick (over {ticks} ticks) ===",
            100.0 * total_crossed as f64 / total as f64
        );
    }

    #[test]
    #[ignore]
    fn measure_cell_crossing_200k() {
        measure(200_000);
    }

    #[test]
    #[ignore]
    fn measure_cell_crossing_900k() {
        measure(900_000);
    }
}

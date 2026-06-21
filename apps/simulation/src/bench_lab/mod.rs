pub mod budget;
pub mod ramp;
pub mod sampler;
pub mod stats;
pub mod world;

pub use budget::{within_budget, BudgetMetric, TICK_BUDGET_US};
pub use ramp::{find_max_pop, MaxPopResult, RampConfig};
pub use sampler::{sample_ticks, PhaseSamples};
pub use stats::{summarize, TickStats};
pub use world::{build_world, Distribution, WorldSpec};

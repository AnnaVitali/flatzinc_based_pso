// rust
// edit: examples/flatzinc_based_pso_test.rs
use std::env;
use std::path::Path;
use std::sync::Arc;
use constraint_evaluator::heuristics::flatzinc_based_pso::FlatzincBasedPSO;
use constraint_evaluator::heuristics::pso::PSO;

const MODEL: &str = "g01";

fn parse_seed() -> i64 {
    env::args()
        .nth(1)
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(42) // default deterministic seed
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let swarm_size: i64 = 100;
    let max_iteration: i64 = 500;
    let w: f64 = 0.669;
    let c1: f64 = 2.385;
    let c2: f64 = 0.558;

    let fzn_path = Path::new(".\\minizinc\\json_flatzinc").join(MODEL.to_string() + ".json");
    let ozn_path = Path::new(".\\minizinc").join(MODEL.to_string() + ".ozn");

    let seed: i64 = parse_seed();
    println!("Using seed = {}", seed);

    let mut corr_pso = FlatzincBasedPSO::new(
        seed,
        swarm_size,
        max_iteration,
        w,
        c1,
        c2,
        fzn_path.to_path_buf(),
        ozn_path.to_path_buf(),
    );

    corr_pso.search();

    Ok(())
}
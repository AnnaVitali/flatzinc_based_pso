use constraint_evaluator::heuristics::flatzinc_based_pso::FlatzincBasedPSO;
use std::path::Path;

const MODEL: &str = "queens16";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    let fzn_path = Path::new(".\\minizinc\\json_flatzinc").join(MODEL.to_string() + ".json");

    let swarm_size: i64 = 100;
    let max_iteration: i64 = 500;
    let w: f64 = 0.669;
    let c1: f64 = 2.385;
    let c2: f64 = 1.558;
    let seed = rand::random_range(0..100);

    let mut flatzinc_pso = FlatzincBasedPSO::new(
        seed,
        swarm_size,
        max_iteration,
        w,
        c1,
        c2,
        fzn_path.to_path_buf(),
    );

    flatzinc_pso.search();

    let elapsed_time = start_time.elapsed();
    println!("Elapsed time: {:.2?}", elapsed_time);

    Ok(())
}
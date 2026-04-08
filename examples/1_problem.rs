use constraint_evaluator::heuristics::flatzinc_based_pso::FlatzincBasedPSO;
use constraint_evaluator::heuristics::pso::PSO;
use std::path::Path;
use std::sync::Arc;

const MODEL: &str = "problem1";
const VIOLATION_THRESHOLD: f64 = 1e-3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    let fzn_path = Path::new(".\\minizinc\\json_flatzinc").join(MODEL.to_string() + ".json");
    let ozn_path = Path::new(".\\minizinc").join(MODEL.to_string() + ".ozn");

    let swarm_size: i64 = 100;
    let max_iteration: i64 = 500;
    let w: f64 = 0.669;
    let c1: f64 = 2.385;
    let c2: f64 = 0.558;
    let seed = 10;//rand::random_range(0..100);

    let eval_fn = |solution: &[f64]| {
        let x2 = solution[0];
        let x1 = 2.0 * x2 - 1.0;

        let constraint = x1.powf(2.0) / 4.0 + x2.powf(2.0) - 1.0;
        let violation = constraint.max(0.0);

        let objective = (x1 - 2.0).powf(2.0) + (x2 - 1.0).powf(2.0);

        (objective, violation)
    };

    let bounds = vec![
        (0.0, 0.8),// x2
    ];

    let mut pso = PSO::new(
        seed,
        swarm_size,
        max_iteration,
        w,
        c1,
        c2,
        Arc::new(eval_fn),
        bounds,
    );

    let (obj_pso, viol_pso) = pso.search();

    let mut flatzinc_pso = FlatzincBasedPSO::new(
        seed,
        swarm_size,
        max_iteration,
        w,
        c1,
        c2,
        fzn_path.to_path_buf(),
        ozn_path.to_path_buf(),
    );

    let (obj_fltzinc, viol_fltzinc) = flatzinc_pso.search();

    let elapsed_time = start_time.elapsed();
    println!("Elapsed time: {:.2?}", elapsed_time);

    let best_known_obj = -1.0;
    println!("Best known objective: {}", best_known_obj);

    println!(
        "{{\"algorithm\":\"pso\", \"model\":\"{}\", \"objective\": {}, \"violation\": {}}}",
        MODEL, obj_pso, viol_pso
    );
    println!(
        "{{\"algorithm\":\"flatzinc_pso\", \"model\":\"{}\", \"objective\": {}, \"violation\": {}}}",
        MODEL,
        obj_fltzinc.unwrap_or(f64::NAN),
        viol_fltzinc
    );

    Ok(())
}
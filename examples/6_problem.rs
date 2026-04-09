use constraint_evaluator::heuristics::flatzinc_based_pso::FlatzincBasedPSO;
use constraint_evaluator::heuristics::pso::PSO;
use std::path::Path;
use std::sync::Arc;

const MODEL: &str = "problem6";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    let fzn_path = Path::new(".\\minizinc\\json_flatzinc").join(MODEL.to_string() + ".json");

    let swarm_size: i64 = 100;
    let max_iteration: i64 = 500;
    let w: f64 = 0.669;
    let c1: f64 = 2.385;
    let c2: f64 = 1.558;
    let seed = rand::random_range(0..100);

    let eval_fn = |solution: &[f64]| {
        let x1 = solution[0];
        let x2 = solution[1];
        let x3 = solution[2];
        let x4 = solution[3];
        let x5 = solution[4];
        let y  = solution[5];

        let c1 = 6.0 * x1 + 3.0 * x2 + 2.0 * x4 + x5 - 6.5;
        let c2 = 10.0 * x1 + 10.0 * x3 + y - 20.0;
        let violation = c1.max(0.0) + c2.max(0.0);

        let objective = -10.5 * x1 - 7.5 * x2 - 3.5 * x3 - 2.5 * x4 - 1.5 * x5 - 10.0 * y;

        (objective, violation)
    };

    let bounds = vec![
        (0.0, 1.0), // x1
        (0.0, 1.0), // x2
        (0.0, 1.0), // x3
        (0.0, 1.0), // x4
        (0.0, 1.0), // x5
        (0.0, 20.0), // y
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
    );

    let (obj_fltzinc, viol_fltzinc) = flatzinc_pso.search();

    let best_known_obj = -1.0;
    println!("Best known objective: {}", best_known_obj);

    let elapsed_time = start_time.elapsed();
    println!("Elapsed time: {:.2?}", elapsed_time);

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
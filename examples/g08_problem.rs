use constraint_evaluator::heuristics::flatzinc_based_pso::FlatzincBasedPSO;
use constraint_evaluator::heuristics::pso::PSO;
use std::path::Path;
use std::sync::Arc;

const MODEL: &str = "g08";

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
        let pi = std::f64::consts::PI;

        let numerator = (2.0 * pi * x1).sin().powf(3.0) * (2.0 * pi * x2).sin();
        let denominator = x1.powf(3.0) * (x1 + x2);
        let objective = - (numerator / denominator);

        let constraints = [
            x1.powf(2.0) - x2 + 1.0,
            1.0 - x1 + (x2 - 4.0).powf(2.0),
        ];

        let violation: f64 = constraints.iter().map(|&c| c.max(0.0)).sum();

        (objective, violation)
    };

    let bounds = vec![
        (0.0, 10.0), // x1
        (0.0, 10.0), // x2
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

    let best_known_obj = -0.0958250414180359;
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

use constraint_evaluator::heuristics::flatzinc_based_pso::FlatzincBasedPSO;
use constraint_evaluator::heuristics::pso::PSO;
use std::path::Path;
use std::sync::Arc;

const MODEL: &str = "g05";

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
        let (x1, x2, x3, x4) = (solution[0], solution[1], solution[2], solution[3]);

        let constraints = [
            -x4 + x3 - 0.55,
            -x3 + x4 - 0.55,
        ];

        let eq_constraints = [
            1000.0 * (-x3 - 0.25).sin() + 1000.0 * (-x4 - 0.25).sin() + 894.8 - x1,
            1000.0 * (x3 - 0.25).sin() + 1000.0 * (x3 - x4 - 0.25).sin() + 894.8 - x2,
            1000.0 * (x4 - 0.25).sin() + 1000.0 * (x4 - x3 - 0.25).sin() + 1294.8,
        ];

        let violation: f64 = constraints.iter().map(|&c| c.max(0.0)).sum::<f64>()
            + eq_constraints.iter().map(|&c| c.abs()).sum::<f64>();

        let objective = 3.0 * x1
            + 0.000001 * x1.powi(3)
            + 2.0 * x2
            + (0.000002 / 3.0) * x2.powi(3);

        (objective, violation)
    };

    let bounds = vec![
        (0.0, 1200.0), // x1
        (0.0, 1200.0), // x2
        (-0.55, 0.55), // x3
        (-0.55, 0.55), // x4
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

    let best_known_obj = 5126.4967140071;
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
use constraint_evaluator::heuristics::flatzinc_based_pso::FlatzincBasedPSO;
use constraint_evaluator::heuristics::pso::PSO;
use std::path::Path;
use std::sync::Arc;

const MODEL: &str = "g01";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    let fzn_path = Path::new(".\\minizinc\\json_flatzinc").join(MODEL.to_string() + ".json");
    let ozn_path = Path::new(".\\minizinc").join(MODEL.to_string() + ".ozn");
    let swarm_size: i64 = 100;
    let max_iteration: i64 = 500;
    let w: f64 = 0.669;
    let c1: f64 = 2.385;
    let c2: f64 = 0.558;

    let seed = rand::random_range(0..100);

    let eval_fn = |solution: &[f64]| {
        let (x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13) = (
            solution[0],
            solution[1],
            solution[2],
            solution[3],
            solution[4],
            solution[5],
            solution[6],
            solution[7],
            solution[8],
            solution[9],
            solution[10],
            solution[11],
            solution[12],
        );

        let constraints = [
            2.0 * x1 + 2.0 * x2 + x10 + x11 - 10.0,
            2.0 * x1 + 2.0 * x3 + x10 + x12 - 10.0,
            2.0 * x2 + 2.0 * x3 + x11 + x12 - 10.0,
            -8.0 * x1 + x10,
            -8.0 * x2 + x11,
            -8.0 * x3 + x12,
            -2.0 * x4 - x5 + x10,
            -2.0 * x6 - x7 + x11,
            -2.0 * x8 - x9 + x12,
        ];

        let violation: f64 = constraints.iter().map(|&c| c.max(0.0)).sum();

        let objective = 5.0 * (x1 + x2 + x3 + x4)
            - 5.0 * (x1.powi(2) + x2.powi(2) + x3.powi(2) + x4.powi(2))
            - (x5 + x6 + x7 + x8 + x9 + x10 + x11 + x12 + x13);

        (objective, violation)
    };

    let mut pso = PSO::new(
        seed,
        swarm_size,
        max_iteration,
        w,
        c1,
        c2,
        Arc::new(eval_fn),
        vec![
            (0.0, 1.0),   // x1
            (0.0, 1.0),   // x2
            (0.0, 1.0),   // x3
            (0.0, 1.0),   // x4
            (0.0, 1.0),   // x5
            (0.0, 1.0),   // x6
            (0.0, 1.0),   // x7
            (0.0, 1.0),   // x8
            (0.0, 1.0),   // x9
            (0.0, 100.0), // x10
            (0.0, 100.0), // x11
            (0.0, 100.0), // x12
            (0.0, 1.0),   // x13
        ],
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

    let best_known_obj = -15.0;
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

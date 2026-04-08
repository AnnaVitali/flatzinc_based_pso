use constraint_evaluator::heuristics::flatzinc_based_pso::FlatzincBasedPSO;
use constraint_evaluator::heuristics::pso::PSO;
use std::path::Path;
use std::sync::Arc;

const MODEL: &str = "g12";
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
    let seed = 10; //rand::random_range(0..100);

    let eval_fn = |solution: &[f64]| {
        let (x1, x2, x3, p, q, r) = (
            solution[0], solution[1], solution[2],
            solution[3], solution[4], solution[5]
        );
        let p_i = p.round();
        let q_i = q.round();
        let r_i = r.round();

        let constraint = (x1 - p_i).powf(2.0) + (x2 - q_i).powf(2.0) + (x3 - r_i).powf(2.0) - 0.0625;
        let violation = constraint.max(0.0);

        let objective = - (100.0 - (x1 - 5.0).powf(2.0) - (x2 - 5.0).powf(2.0) - (x3 - 5.0).powf(2.0)) / 100.0;

        (objective, violation)
    };

    let bounds = vec![
        (0.0, 10.0), // x1
        (0.0, 10.0), // x2
        (0.0, 10.0), // x3
        (1.0, 9.0),  // p (relaxed to float, rounded in eval)
        (1.0, 9.0),  // q (relaxed to float, rounded in eval)
        (1.0, 9.0),  // r (relaxed to float, rounded in eval)
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
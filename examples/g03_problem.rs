use std::path::Path;
use std::sync::Arc;
use constraint_evaluator::heuristics::flatzinc_based_pso::FlatzincBasedPSO;
use constraint_evaluator::heuristics::pso::PSO;

const N: usize = 10;
const MODEL: &str = "g03";

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
        let n = solution.len();
        let mut prefix_prod_x = vec![0.0; n];
        prefix_prod_x[0] = solution[0];
        for i in 1..n {
            prefix_prod_x[i] = prefix_prod_x[i - 1] * solution[i];
        }

        let product_x = prefix_prod_x[n - 1];
        let sqrt = (n as f64).sqrt();
        let scale = sqrt.powi(n as i32);
        let objective = -scale * product_x;
        let sum_sq: f64 = solution.iter().map(|xi| xi.powi(2)).sum();
        let violation = (sum_sq - 1.0).abs();

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
        vec![(0.0, 1.0); N],
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

    let best_known_obj = -1.00050010001000;
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
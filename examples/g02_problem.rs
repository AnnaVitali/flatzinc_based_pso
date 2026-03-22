use std::path::Path;
use constraint_evaluator::heuristics::flatzinc_based_pso::FlatzincBasedPSO;
use std::sync::Arc;
use constraint_evaluator::heuristics::pso::PSO;

const N: usize = 20;
const MODEL: &str = "g02";
const VIOLATION_THRESHOLD: f64 = 1e-3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fzn_path = Path::new(".\\minizinc\\json_flatzinc").join(MODEL.to_string() + ".json");
    let ozn_path = Path::new(".\\minizinc").join(MODEL.to_string() + ".ozn");
    let swarm_size: i64 = 100;
    let max_iteration: i64 = 500;
    let w: f64 = 0.669;
    let c1: f64 = 2.385;
    let c2: f64 = 1.558;

    let seed = rand::random_range(0..100);

    let eval_fn = |solution: &[f64]| {
        let x = solution;
        let n = N as f64;

        let sum_cos4: f64 = x.iter().map(|&xi| xi.cos().powf(4.0)).sum();
        let prod_cos2: f64 = x.iter().map(|&xi| xi.cos().powf(2.0)).product();
        let numerator = sum_cos4 - 2.0 * prod_cos2;
        let denominator = (x.iter().enumerate().map(|(i, &xi)| (i as f64 + 1.0) * xi.powf(2.0)).sum::<f64>()).sqrt();
        let objective = -((numerator.abs()) / denominator);


        let g1 = 0.75 - x.iter().product::<f64>();
        let g2 = x.iter().sum::<f64>() - 7.5 * n;

        let violation = g1.max(0.0) + g2.max(0.0);

        (objective, violation)
    };

    let bounds = vec![(0.0, 10.0); N];

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

    let (obj_pso, viol_pso)= pso.search();

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

    let mut score_viol_pso = 0;
    let mut score_viol_fltzinc = 0;

    let best_known_obj = -0.80361910412559;

    if viol_pso < viol_fltzinc {
        println!(
            "PSO violation < Flatzinc PSO violation: {} < {}",
            viol_pso, viol_fltzinc
        );
        println!("difference in violation: {}", viol_fltzinc - viol_pso);
        score_viol_pso += 1;
    } else if viol_pso > viol_fltzinc {
        println!(
            "Flatzinc PSO violation < PSO violation: {} < {}",
            viol_fltzinc, viol_pso
        );
        println!("difference in violation: {}", viol_pso - viol_fltzinc);
        score_viol_fltzinc += 1;
    }

    if viol_pso <= VIOLATION_THRESHOLD || viol_fltzinc <= VIOLATION_THRESHOLD {
        let distance_to_best = |obj: f64| (best_known_obj - obj).abs();
        let distance_pso = distance_to_best(obj_pso);
        let distance_flatzinc = distance_to_best(obj_fltzinc.unwrap());

        if distance_pso < distance_flatzinc {
            println!(
                "PSO objective closer to best known objective: {} < {}",
                obj_pso,
                obj_fltzinc.unwrap()
            );
            println!("difference in objective: {}", obj_fltzinc.unwrap() - obj_pso);
            score_viol_pso += 1;
        } else if distance_pso > distance_flatzinc {
            println!(
                "Flatzinc PSO objective closer to best known objective: {} < {}",
                obj_fltzinc.unwrap(),
                obj_pso
            );
            println!("difference in objective: {}", obj_pso - obj_fltzinc.unwrap());
            score_viol_fltzinc += 1;
        }
    }

    // Output machine-readable summaries for external parsing (one per algorithm)
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

    println!("\nFinal Scores:");
    println!("PSO: {}", score_viol_pso);
    println!("Flatzinc PSO: {}", score_viol_fltzinc);

    Ok(())
}

use constraint_evaluator::heuristics::flatzinc_based_pso::FlatzincBasedPSO;
use constraint_evaluator::heuristics::pso::PSO;
use std::path::Path;
use std::sync::Arc;

const MODEL: &str = "g10";
const VIOLATION_THRESHOLD: f64 = 1e-3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fzn_path = Path::new(".\\minizinc\\json_flatzinc").join(MODEL.to_string() + ".json");
    let ozn_path = Path::new(".\\minizinc").join(MODEL.to_string() + ".ozn");

    let swarm_size: i64 = 100;
    let max_iteration: i64 = 1000;
    let w: f64 = 0.669;
    let c1: f64 = 2.385;
    let c2: f64 = 0.558;
    let seed = rand::random_range(0..100);

    let eval_fn = |solution: &[f64]| {
        let (x1, x2, x3, x4, x5, x6, x7, x8) = (
            solution[0], solution[1], solution[2], solution[3],
            solution[4], solution[5], solution[6], solution[7]
        );

        // Constraints (all must be <= 0)
        let constraints = [
            -1.0 + 0.0025 * (x4 + x6),
            -1.0 + 0.0025 * (x5 + x7 - x4),
            -1.0 + 0.01 * (x8 - x5),
            -x1 * x6 + 833.33252 * x4 + 100.0 * x1 - 83333.333,
            -x2 * x7 + 1250.0 * x5 + x2 * x4 - 1250.0 * x4,
            -x3 * x8 + 1250000.0 + x3 * x5 - 2500.0 * x5,
        ];

        // Sum of violations: positive part for inequalities
        let violation: f64 = constraints.iter().map(|&c| c.max(0.0)).sum();

        // Objective
        let objective = x1 + x2 + x3;

        (objective, violation)
    };

    let bounds = vec![
        (100.0, 10000.0),  // x1
        (1000.0, 10000.0), // x2
        (1000.0, 10000.0), // x3
        (10.0, 1000.0),    // x4
        (10.0, 1000.0),    // x5
        (10.0, 1000.0),    // x6
        (10.0, 1000.0),    // x7
        (10.0, 1000.0),    // x8
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

    let mut score_viol_pso = 0;
    let mut score_viol_fltzinc = 0;

    // best known objective for g10 (placeholder — set real value if available)
    let best_known_obj = 0.0;

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

    println!("\nFinal Scores:");
    println!("PSO: {}", score_viol_pso);
    println!("Flatzinc PSO: {}", score_viol_fltzinc);

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
use constraint_evaluator::heuristics::flatzinc_based_pso::FlatzincBasedPSO;
use constraint_evaluator::heuristics::pso::PSO;
use std::path::Path;
use std::sync::Arc;

const MODEL: &str = "g07";
const VIOLATION_THRESHOLD: f64 = 1e-3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fzn_path = Path::new(".\\minizinc\\json_flatzinc").join(MODEL.to_string() + ".json");
    let ozn_path = Path::new(".\\minizinc").join(MODEL.to_string() + ".ozn");

    let swarm_size: i64 = 100;
    let max_iteration: i64 = 500;
    let w: f64 = 0.669;
    let c1: f64 = 2.385;
    let c2: f64 = 0.558;
    let seed = rand::random_range(0..100);

    let eval_fn = |solution: &[f64]| {
        let (x1, x2, x3, x4, x5, x6, x7, x8, x9, x10) = (
            solution[0], solution[1], solution[2], solution[3], solution[4],
            solution[5], solution[6], solution[7], solution[8], solution[9]
        );

        let constraints = [
            -105.0 + 4.0 * x1 + 5.0 * x2 - 3.0 * x7 + 9.0 * x8,
            10.0 * x1 - 8.0 * x2 - 17.0 * x7 + 2.0 * x8,
            -8.0 * x1 + 2.0 * x2 + 5.0 * x9 - 2.0 * x10 - 12.0,
            3.0 * (x1 - 2.0).powf(2.0) + 4.0 * (x2 - 3.0).powf(2.0) + 2.0 * x3.powf(2.0) - 7.0 * x4 - 120.0,
            5.0 * x1.powf(2.0) + 8.0 * x2 + (x3 - 6.0).powf(2.0) - 2.0 * x4 - 40.0,
            x1.powf(2.0) + 2.0 * (x2 - 2.0).powf(2.0) - 2.0 * x1 * x2 + 14.0 * x5 - 6.0 * x6,
            0.5 * (x1 - 8.0).powf(2.0) + 2.0 * (x2 - 4.0).powf(2.0) + 3.0 * x5.powf(2.0) - x6 - 30.0,
            -3.0 * x1 + 6.0 * x2 + 12.0 * (x9 - 8.0).powf(2.0) - 7.0 * x10,
        ];

        let violation: f64 = constraints.iter().map(|&c| c.max(0.0)).sum();

        let objective =
            x1.powf(2.0) + x2.powf(2.0) + x1 * x2
            - 14.0 * x1 - 16.0 * x2
            + (x3 - 10.0).powf(2.0)
            + 4.0 * (x4 - 5.0).powf(2.0)
            + (x5 - 3.0).powf(2.0)
            + 2.0 * (x6 - 1.0).powf(2.0)
            + 5.0 * x7.powf(2.0)
            + 7.0 * (x8 - 11.0).powf(2.0)
            + 2.0 * (x9 - 10.0).powf(2.0)
            + (x10 - 7.0).powf(2.0)
            + 45.0;

        (objective, violation)
    };

    let bounds = vec![
        (-10.0, 10.0), // x1
        (-10.0, 10.0), // x2
        (-10.0, 10.0), // x3
        (-10.0, 10.0), // x4
        (-10.0, 10.0), // x5
        (-10.0, 10.0), // x6
        (-10.0, 10.0), // x7
        (-10.0, 10.0), // x8
        (-10.0, 10.0), // x9
        (-10.0, 10.0), // x10
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

    let best_known_obj = 24.30620906818;

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
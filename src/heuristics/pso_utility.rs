const FEASIBILITY_TOL: f64 = 1e-3;

/// Determines whether a candidate solution is better than the incumbent solution based on objective values and constraint violations, following a
/// lexicographic ordering where feasibility is prioritized over optimality.
///
/// # Arguments
/// * `candidate_obj` - The objective value of the candidate solution, which may be  `None` if the objective is not defined for the candidate.
/// * `candidate_violation` - The total violation of constraints for the candidate solution, where a value of 0 or less indicates feasibility.
/// * `incumbent_obj` - The objective value of the incumbent solution, which may be `None` if the objective is not defined for the incumbent.
/// * `incumbent_violation` - The total violation of constraints for the incumbent solution, where a value of 0 or less indicates feasibility.
/// # Returns
/// `true` if the candidate solution is considered better than the incumbent solution based on the following criteria:
/// 1. If the candidate is feasible and the incumbent is not, the candidate is better.
/// 2. If the incumbent is feasible and the candidate is not, the incumbent is better.
/// 3. If both are feasible, the one with the better (lower) objective value is better.
pub fn is_better_candidate(
    candidate_obj: Option<f64>,
    candidate_violation: f64,
    incumbent_obj: Option<f64>,
    incumbent_violation: f64,
) -> bool {
    let candidate_feasible = candidate_violation <= FEASIBILITY_TOL;
    let incumbent_feasible = incumbent_violation <= FEASIBILITY_TOL;

    if candidate_feasible && !incumbent_feasible {
        return true;
    }
    if !candidate_feasible && incumbent_feasible {
        return false;
    }

    if candidate_feasible && incumbent_feasible {
        match (candidate_obj, incumbent_obj) {
            (Some(c), Some(i)) => c < i,
            (Some(_), None) => true,
            _ => false,
        }
    } else {
        candidate_violation < incumbent_violation
    }
}
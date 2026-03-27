use std::collections::HashMap;
use crate::solution_provider::VariableValue;

/// Computes the left-hand side value of an integer linear expression given coefficients, variable identifiers, and a solution map.
///
/// # Arguments
/// * `coeff` - A vector of integer coefficients.
/// * `vars_involved` - A vector of variable identifiers corresponding to the coefficients.
/// * `complete_solution` - A map from variable names to their integer values.
///
/// # Returns
/// The computed integer value of the linear expression.
pub fn int_lin_left_term(
    coeff: Vec<i64>,
    vars_involved: Vec<String>,
    complete_solution: &HashMap<String, VariableValue>,
) -> i64 {
    let left_side_term: i64 = coeff
        .iter()
        .zip(vars_involved.iter())
        .map(|(c, id)| {
            let var_val = complete_solution
                .get(id)
                .and_then(|int_val| match int_val {
                    VariableValue::Int(int_val) => Some(*int_val),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("No value defined for the variable {}", id));

            c * var_val
        })
        .sum();
    left_side_term
}

/// Computes the left-hand side value of a float linear expression given coefficients, variable identifiers, and a solution map.
///
/// # Arguments
/// * `coeff` - A vector of float coefficients.
/// * `vars_involved` - A vector of variable identifiers corresponding to the coefficients.
/// * `complete_solution` - A map from variable names to their float values.
///
/// # Returns
/// The computed float value of the linear expression.
pub fn float_lin_left_term(
    coeff: Vec<f64>,
    vars_involved: Vec<String>,
    complete_solution: &HashMap<String, VariableValue>,
) -> f64 {
    let left_side_term: f64 = coeff
        .iter()
        .zip(vars_involved.iter())
        .map(|(c, id)| {
            let var_val = complete_solution
                .get(id)
                .and_then(|int_val| match int_val {
                    VariableValue::Float(int_val) => Some(*int_val),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("No value defined for the variable {}", id));
            c * var_val
        })
        .sum();
    left_side_term
}
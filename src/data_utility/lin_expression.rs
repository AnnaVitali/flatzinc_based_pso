use std::collections::HashMap;
use flatzinc_serde::Identifier;
use crate::solution_provider::VariableValue;

pub fn int_lin_left_term(
        coeff: Vec<i64>,
        vars_involved: Vec<Identifier>,
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

pub fn float_lin_left_term(
        coeff: Vec<f64>,
        vars_involved: Vec<Identifier>,
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
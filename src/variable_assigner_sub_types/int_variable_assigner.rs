use crate::args_extractor_sub_types::int_args_extractor::IntArgsExtractor;
use crate::invariant_evaluator::CallWithDefines;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{ Array, Identifier};
use std::cmp::{max, min};
use std::collections::HashMap;
use crate::invariant_evaluator_sub_types::int_invariant_evaluator::{A_TERM_INDEX, B_TERM_INDEX, C_TERM_INDEX, COEFF_LIN_CONSTR_INDEX, CONST_LIN_CONSTR_INDEX, IntInvariantEvaluator, R_TERM_INDEX, VARS_LIN_CONSTR_INDEX};

#[derive(Debug, Clone, Default)]
pub struct IntVariableAssigner {
    args_extractor: IntArgsExtractor,
    arrays: HashMap<Identifier, Array>,
}

impl IntVariableAssigner {
    pub fn new(arrays: HashMap<Identifier, Array>) -> Self {
        let args_extractor = IntArgsExtractor::new();

        Self {
            args_extractor,
            arrays,
        }
    }

    pub fn array_int_element(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for array_int_element");
        if vars_identifier.contains(&defined_var) {
            self.args_extractor.extract_int_element_array(&constraint.call, &self.arrays, complete_solution)
        } else {
            self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn int_abs(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_abs");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            a.abs()
        } else {
            self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn int_div(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_div");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a / b
        } else {
            self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn int_eq(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_eq");
        if vars_identifier.contains(&defined_var) {
            self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution)
        } else {
            self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn int_eq_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_eq_reif");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a == b
        } else {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, &complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, &complete_solution);
            let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution);
            (a == b) == r
        }
    }

    pub fn int_le_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_le_reif");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a <= b
        } else {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, &complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, &complete_solution);
            let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution);
            (a <= b) == r
        }
    }

    pub fn int_lin_eq(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
        variable: &String,
    ) -> i64 {
        let mut coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let mut vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays,);

        let var_idx = vars_involved.iter().position(|id| id == variable);

        let term = self.args_extractor.extract_int_value(
            CONST_LIN_CONSTR_INDEX,
            &constraint.call,
            complete_solution,
        );

        if var_idx.is_none() {
            let left_side_term = self.int_lin_left_term(coeff, vars_involved, complete_solution);
            let result = left_side_term - term;
            return result;
        }

        let var_idx = var_idx.unwrap();

        let var_coeff = coeff.remove(var_idx);
        vars_involved.remove(var_idx);

        let sum = self.int_lin_left_term(coeff, vars_involved, complete_solution);

        let result = (term - sum) / var_coeff;

        result

    }

    pub fn int_lin_eq_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_lin_eq_reif");
        if vars_identifier.contains(&defined_var) {
            let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
                COEFF_LIN_CONSTR_INDEX,
                &constraint.call,
                &self.arrays,
            );
            let vars_involved = self
                .args_extractor
                .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
            let term = self.args_extractor.extract_int_value(
                CONST_LIN_CONSTR_INDEX,
                &constraint.call,
                complete_solution,
            );
            let left_side_term = self.int_lin_left_term(coeff, vars_involved, complete_solution);
            left_side_term == term
        } else {
            let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
                COEFF_LIN_CONSTR_INDEX,
                &constraint.call,
                &self.arrays,
            );
            let vars_involved = self
                .args_extractor
                .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
            let term = self.args_extractor.extract_int_value(
                CONST_LIN_CONSTR_INDEX,
                &constraint.call,
                &complete_solution,
            );
            let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution);
            let left_side_term = self.int_lin_left_term(coeff, vars_involved, &complete_solution);
            (left_side_term == term) == r
        }
    }

    pub fn int_lin_le_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_lin_le_reif");
        if vars_identifier.contains(&defined_var) {
            let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
                COEFF_LIN_CONSTR_INDEX,
                &constraint.call,
                &self.arrays,
            );
            let vars_involved = self
                .args_extractor
                .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
            let term = self.args_extractor.extract_int_value(
                CONST_LIN_CONSTR_INDEX,
                &constraint.call,
                complete_solution,
            );
            let left_side_term = self.int_lin_left_term(coeff, vars_involved, complete_solution);
            left_side_term <= term
        } else {
            let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
                COEFF_LIN_CONSTR_INDEX,
                &constraint.call,
                &self.arrays,
            );
            let vars_involved = self
                .args_extractor
                .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
            let term = self.args_extractor.extract_int_value(
                CONST_LIN_CONSTR_INDEX,
                &constraint.call,
                &complete_solution,
            );
            let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution);
            let left_side_term = self.int_lin_left_term(coeff, vars_involved, &complete_solution);
            (left_side_term <= term) == r
        }
    }

    pub fn int_lin_ne_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_lin_ne_reif");
        if vars_identifier.contains(&defined_var) {
            let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
                COEFF_LIN_CONSTR_INDEX,
                &constraint.call,
                &self.arrays,
            );
            let vars_involved = self
                .args_extractor
                .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
            let term = self.args_extractor.extract_int_value(
                CONST_LIN_CONSTR_INDEX,
                &constraint.call,
                complete_solution,
            );
            let left_side_term = self.int_lin_left_term(coeff, vars_involved, complete_solution);
            left_side_term != term
        } else {
            let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
                COEFF_LIN_CONSTR_INDEX,
                &constraint.call,
                &self.arrays,
            );
            let vars_involved = self
                .args_extractor
                .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
            let term = self.args_extractor.extract_int_value(
                CONST_LIN_CONSTR_INDEX,
                &constraint.call,
                &complete_solution,
            );
            let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution);
            let left_side_term = self.int_lin_left_term(coeff, vars_involved, &complete_solution);
            (left_side_term != term) == r
        }
    }

    pub fn int_lt_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_lt_reif");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a < b
        } else {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, &complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, &complete_solution);
            let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution);
            (a < b) == r
        }
    }

    pub fn int_max(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_max");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, complete_solution);
            max(a, b)
        } else {
            self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn int_min(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_min");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, complete_solution);
            min(a, b)
        } else {
            self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn int_mod(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_mod");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a % b
        } else {
            self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn int_ne_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_ne_reif");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a != b
        } else {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, &complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, &complete_solution);
            let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution);
            (a != b) == r
        }
    }

    pub fn int_plus(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_plus");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a + b
        } else {
            self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn int_pow(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_pow");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a.pow(b as u32)
        } else {
            self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn int_minus(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_minus");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a - b
        } else {
            self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn int_times(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int_times");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a * b
        } else {
            self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    fn int_lin_left_term(
        &self,
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
}

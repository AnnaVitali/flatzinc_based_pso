use crate::args_extractor_sub_types::int_args_extractor::IntArgsExtractor;
use crate::data_utility::data_utility::ConstraintEvaluation;
use crate::invariant_evaluator::CallWithDefines;
use crate::logger::write_verbose_output;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Identifier};
use log::info;
use std::cmp::{max, min};
use std::collections::HashMap;

pub const A_TERM_INDEX: usize = 0;
pub const B_TERM_INDEX: usize = 1;
pub const C_TERM_INDEX: usize = 2;
pub const R_TERM_INDEX: usize = 2;
pub const R_TERM_LIN_EXPR_REIF_INDEX: usize = 3;
pub const COEFF_LIN_CONSTR_INDEX: usize = 0;
pub const VARS_LIN_CONSTR_INDEX: usize = 1;
pub const CONST_LIN_CONSTR_INDEX: usize = 2;

#[derive(Debug, Clone, Default)]
pub struct IntInvariantEvaluator {
    arrays: HashMap<Identifier, Array>,
    args_extractor: IntArgsExtractor,
    verbose: bool,
}

impl IntInvariantEvaluator {
    pub fn new(arrays: HashMap<Identifier, Array>, verbose: bool) -> Self {
        let args_extractor = IntArgsExtractor::new();

        Self {
            arrays,
            args_extractor,
            verbose,
        }
    }

    pub fn array_int_element(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;
        let array_value =
            self.args_extractor
                .extract_int_element_array(&constraint.call, &self.arrays, &solution);
        let value = self
            .args_extractor
            .extract_int_value(C_TERM_INDEX, &constraint.call, &solution);

        if array_value != value {
            if self.verbose {
                info!("Violated: array value {} = {}", array_value, value);
            }
            violation = (array_value - value).abs();
        }

        ConstraintEvaluation {
            constraint_id: constraint.id,
            violation: violation as f64,
        }
    }

    pub fn int_abs(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_abs = a.abs();
        if b != real_abs {
            if self.verbose {
                info!("Violated: abs({}) = {}", a, b);
            }
            violation = (b - real_abs).abs();
        }

        ConstraintEvaluation {
            constraint_id: constraint.id,
            violation: violation as f64,
        }
    }

    pub fn int_div(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self
            .args_extractor
            .extract_int_value(C_TERM_INDEX, &constraint.call, &solution);

        if b == 0 {
            violation = 1
        } else {
            let result = a / b;
            if c != result {
                if self.verbose {
                    info!("Violated: {} / {} = {}", a, b, c);
                }
                violation = (c - result).abs();
            }
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_eq(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);

        if a != b {
            if self.verbose {
                info!("Violated: {} = {}", a, b);
            }
            violation = (a - b).abs();

            ConstraintEvaluation {
                violation: violation as f64,
                constraint_id: constraint.id,
            }
        } else {
            ConstraintEvaluation {
                violation: violation as f64,
                constraint_id: constraint.id,
            }
        }
    }

    pub fn int_eq_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);
        let r = self
            .args_extractor
            .extract_bool_value(R_TERM_INDEX, &constraint.call, &solution);

        if r != (a == b) {
            if self.verbose {
                info!("constraint: {:?}", constraint);
                info!("Violated: {} <-> {} = {}", r, a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_le(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);

        if a > b {
            if self.verbose {
                info!("Violated: {} <= {}", a, b);
            }
            violation = (a - b).abs();
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_le_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);
        let r = self
            .args_extractor
            .extract_bool_value(R_TERM_INDEX, &constraint.call, &solution);

        if r != (a <= b) {
            if self.verbose {
                info!("Violated: {} <-> {} <= {}", r, a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_lin_eq(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX,  &constraint.call, &self.arrays);
        let term =
            self.args_extractor
                .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call, &solution);

        let mut verbose_terms = String::new();

        let left_side_term =
            self.int_lin_left_term(&coeff, solution, vars_involved.clone(), &mut verbose_terms);

        if left_side_term != term {
            if self.verbose {
                info!("constraint: {:?}", constraint);
                info!("Violated: {} = {}", verbose_terms, term);
            }
            violation = (left_side_term - term).abs();
        }

        ConstraintEvaluation{
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_lin_eq_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX,  &constraint.call, &self.arrays);
        let term =
            self.args_extractor
                .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call, &solution);
        let r = self.args_extractor.extract_bool_value(
            R_TERM_LIN_EXPR_REIF_INDEX,
            &constraint.call,
            &solution,
        );

        let mut verbose_terms = String::new();

        let left_side_term =
            self.int_lin_left_term(&coeff, solution, vars_involved, &mut verbose_terms);

        if r != (left_side_term == term) {
            if self.verbose {
                info!("Violated: {} <-> {} = {}", r, verbose_terms, term);
            }
            violation = 1
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_lin_le(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let term =
            self.args_extractor
                .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call, &solution);

        let mut verbose_terms = String::new();

        let left_side_term =
            self.int_lin_left_term(&coeff, solution, vars_involved.clone(), &mut verbose_terms);
        if left_side_term > term {
            if self.verbose {
                info!("Violated: {} <= {}", verbose_terms, term);
            }
            violation = (left_side_term - term).abs();
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }


    pub fn int_lin_le_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let term =
            self.args_extractor
                .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call, &solution);
        let r = self.args_extractor.extract_bool_value(
            R_TERM_LIN_EXPR_REIF_INDEX,
            &constraint.call,
            &solution,
        );

        let mut verbose_terms = String::new();

        let left_side_term =
            self.int_lin_left_term(&coeff, solution, vars_involved, &mut verbose_terms);

        if r != (left_side_term <= term) {
            if self.verbose {
                info!("Violated: {} <-> {} <= {}", r, verbose_terms, term);
            }
            violation = 1
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_lin_ne(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let term =
            self.args_extractor
                .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call, &solution);

        let mut verbose_terms = String::new();

        let left_side_term =
            self.int_lin_left_term(&coeff, solution, vars_involved, &mut verbose_terms);

        if left_side_term == term {
            if self.verbose {
                info!("Violated: {} != {}", verbose_terms, term);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_lin_ne_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let term =
            self.args_extractor
                .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call, &solution);
        let r = self.args_extractor.extract_bool_value(
            R_TERM_LIN_EXPR_REIF_INDEX,
            &constraint.call,
            &solution,
        );

        let mut verbose_terms = String::new();

        let left_side_term =
            self.int_lin_left_term(&coeff, solution, vars_involved, &mut verbose_terms);

        if r != (left_side_term != term) {
            if self.verbose {
                info!("Violated: {} <-> {} != {}", r, verbose_terms, term);
            }
            violation = 1
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_lt(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);

        if a >= b {
            if self.verbose {
                info!("Violated: {} < {}", a, b);
            }
            violation = a - b + 1;

        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_lt_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);
        let r = self
            .args_extractor
            .extract_bool_value(R_TERM_INDEX, &constraint.call, &solution);

        if r != (a < b) {
            if self.verbose {
                info!("Violated: {} <-> {} < {}", r, a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_max(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self
            .args_extractor
            .extract_int_value(C_TERM_INDEX, &constraint.call, &solution);

        let max = max(a, b);
        if c != max {
            if self.verbose {
                info!("Violated: max({},{}) = {}", a, b, c);
            }
            violation = (c - max).abs();
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_min(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self
            .args_extractor
            .extract_int_value(C_TERM_INDEX, &constraint.call, &solution);

        let min = min(a, b);
        if c != min {
            if self.verbose {
                info!("Violated: min({},{}) = {}", a, b, c);
            }
            violation = (c - min).abs();
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_mod(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self
            .args_extractor
            .extract_int_value(C_TERM_INDEX, &constraint.call, &solution);

        let real_mod = a % b;
        if c != real_mod {
            if self.verbose {
                info!("Violated: {} mod {} = {}", a, b, c);
            }
            violation = (c - real_mod).abs();
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_ne(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);

        if a == b {
            if self.verbose {
                info!("Violated: {} != {}", a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_ne_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);
        let r = self
            .args_extractor
            .extract_bool_value(R_TERM_INDEX, &constraint.call, &solution);

        if r != (a != b) {
            if self.verbose {
                info!("Violated: {} <-> {} != {}", r, a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_pow(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self
            .args_extractor
            .extract_int_value(C_TERM_INDEX, &constraint.call, &solution);

        let result = a.pow(b as u32);
        if c != result {
            if self.verbose {
                info!("Violated: {} ^ {} = {}", a, b, c);
            }
            violation = (c - result).abs();
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn int_times(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self
            .args_extractor
            .extract_int_value(C_TERM_INDEX, &constraint.call, &solution);

        let result = a * b;
        if c != result {
            if self.verbose {
                info!("Violated: {} * {} = {}", a, b, c);
            }
            violation = (c - result).abs();
        } 

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    fn int_lin_left_term(
        &self,
        coeff: &Vec<i64>,
        solution: &HashMap<String, VariableValue>,
        vars_involved: Vec<Identifier>,
        verbose_terms: &mut String,
    ) -> i64 {
        let left_side_term: i64 = coeff
            .iter()
            .zip(vars_involved.iter())
            .map(|(c, id)| {
                let var_val = solution
                    .get(id)
                    .and_then(|int_val| match int_val {
                        VariableValue::Int(int_val) => {
                            Some(*int_val)},
                        _ => None,
                    })
                    .unwrap_or_else(|| panic!("No value defined for the variable {}", id));

                if self.verbose {
                    write_verbose_output(verbose_terms, c, &var_val);
                }
                c * var_val
            })
            .sum();
        left_side_term
    }
}

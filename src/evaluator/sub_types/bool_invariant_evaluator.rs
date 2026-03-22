use crate::args_extractor::sub_types::bool_args_extractor::BoolArgsExtractor;
use crate::data_utility::constraint_evaluation::ConstraintEvaluation;
use crate::data_utility::logger::write_verbose_output;
use crate::evaluator::evaluator::CallWithDefines;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Identifier};
use log::info;
use std::collections::HashMap;

pub const A_TERM_INDEX: usize = 0;
pub const B_TERM_INDEX: usize = 1;
pub const C_TERM_INDEX: usize = 2;
pub const AS_ARRAY_INDEX: usize = 0;
pub const BS_ARRAY_INDEX: usize = 1;
pub const R_TERM_INDEX: usize = 1;
pub const COEFF_LIN_CONSTR_INDEX: usize = 0;

#[derive(Debug, Clone, Default)]
pub struct BoolInvariantEvaluator {
    arrays: HashMap<Identifier, Array>,
    args_extractor: BoolArgsExtractor,
    verbose: bool,
}

impl BoolInvariantEvaluator {
    pub fn new(arrays: HashMap<Identifier, Array>, verbose: bool) -> Self {
        let args_extractor = BoolArgsExtractor::new();

        Self {
            arrays,
            args_extractor,
            verbose,
        }
    }

    pub fn array_bool_and(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let as_array = self.args_extractor.extract_bool_array(
            AS_ARRAY_INDEX,
            &self.arrays,
            &constraint.call,
            &solution,
        );
        let r = self
            .args_extractor
            .extract_bool_value(R_TERM_INDEX, &constraint.call, &solution);

        if as_array.iter().all(|&item| item) != r {
            if self.verbose {
                let joined = as_array
                    .iter()
                    .map(|b| if *b { "true" } else { "false" })
                    .collect::<Vec<_>>()
                    .join(" /\\ ");
                info!("Violated: {} <-> {}", joined, r);
            }
            violation = 1
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn array_bool_element(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let array_value = self.args_extractor.extract_bool_element_array(
            AS_ARRAY_INDEX,
            &constraint.call,
            &self.arrays,
            &solution,
        );
        let value =
            self.args_extractor
                .extract_bool_value(C_TERM_INDEX, &constraint.call, &solution);

        if array_value != value {
            if self.verbose {
                info!("Violated: array value {} = {}", array_value, value);
            }
            violation = 1
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn array_bool_xor(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let array_value = self.args_extractor.extract_bool_array(
            AS_ARRAY_INDEX,
            &self.arrays,
            &constraint.call,
            &solution,
        );

        if !array_value.iter().fold(false, |acc, &item| acc ^ item) {
            if self.verbose {
                let joined = array_value
                    .iter()
                    .map(|b| if *b { "true" } else { "false" })
                    .collect::<Vec<_>>()
                    .join(" xor ");
                info!("Violated: {} ", joined);
            }
            violation = 1
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool_and(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_bool_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_bool_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self
            .args_extractor
            .extract_bool_value(C_TERM_INDEX, &constraint.call, &solution);

        if c != (a && b) {
            if self.verbose {
                info!("Violated: {} <-> {} /\\ {}", c, a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool_clause(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let as_array = self.args_extractor.extract_bool_array(
            AS_ARRAY_INDEX,
            &self.arrays,
            &constraint.call,
            &solution,
        );
        let bs_array = self.args_extractor.extract_bool_array(
            BS_ARRAY_INDEX,
            &self.arrays,
            &constraint.call,
            &solution,
        );

        let or_as_array = as_array.iter().map(|&b| b as i64).sum::<i64>();
        let or_bs_array = bs_array.iter().map(|&b| b as i64).sum::<i64>();

        let result = or_as_array + or_bs_array;

        if result == 0 {
            if self.verbose {
                let mut interleaved: Vec<&str> = Vec::new();
                let max_len = std::cmp::max(as_array.len(), bs_array.len());
                for i in 0..max_len {
                    if let Some(a) = as_array.get(i) {
                        interleaved.push(if *a { "true" } else { "false" });
                    }
                    if let Some(b) = bs_array.get(i) {
                        interleaved.push(if *b { "not(true)" } else { "not(false)" });
                    }
                }
                let joined = interleaved.join(" \\/ ");
                info!("constraint{:?}", constraint.call);
                info!("Violated: {}", joined);
            }
            violation = 1
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool_eq(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_bool_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_bool_value(B_TERM_INDEX, &constraint.call, &solution);

        if a != b {
            if self.verbose {
                info!("Violated: {} = {}", a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool_eq_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_bool_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_bool_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self
            .args_extractor
            .extract_bool_value(C_TERM_INDEX, &constraint.call, &solution);

        if c != (a == b) {
            if self.verbose {
                info!("Violated: {} <-> {} = {}", c, a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool_le(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_bool_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_bool_value(B_TERM_INDEX, &constraint.call, &solution);

        if a > b {
            if self.verbose {
                info!("Violated: {} <= {}", a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool_le_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_bool_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_bool_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self
            .args_extractor
            .extract_bool_value(C_TERM_INDEX, &constraint.call, &solution);

        if c != (a <= b) {
            if self.verbose {
                info!("Violated: {} <-> {} <= {}", c, a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool_lin_eq(
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
        let bs_array = self.args_extractor.extract_bool_array(
            BS_ARRAY_INDEX,
            &self.arrays,
            &constraint.call,
            &solution,
        );
        let term = self
            .args_extractor
            .extract_int_constant_term_lin_expr(&constraint.call);

        let mut verbose_terms = String::new();

        let left_side_term = self.int_lin_left_term(&coeff, &bs_array, &mut verbose_terms);

        if left_side_term != term {
            if self.verbose {
                info!("Violated: {} = {}", verbose_terms, term);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool_lin_le(
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
        let bs_array = self.args_extractor.extract_bool_array(
            BS_ARRAY_INDEX,
            &self.arrays,
            &constraint.call,
            &solution,
        );
        let term = self
            .args_extractor
            .extract_int_constant_term_lin_expr(&constraint.call);

        let mut verbose_terms = String::new();

        let left_side_term = self.int_lin_left_term(&coeff, &bs_array, &mut verbose_terms);

        if left_side_term > term {
            if self.verbose {
                info!("Violated: {} <= {}", verbose_terms, term);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool_lt(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_bool_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_bool_value(B_TERM_INDEX, &constraint.call, &solution);

        if a >= b {
            if self.verbose {
                info!("Violated: {} < {}", a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool_lt_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_bool_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_bool_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self
            .args_extractor
            .extract_bool_value(C_TERM_INDEX, &constraint.call, &solution);

        if c != (a < b) {
            if self.verbose {
                info!("Violated: {} <-> {} < {}", c, a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool_not(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_bool_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_bool_value(B_TERM_INDEX, &constraint.call, &solution);

        if a == b {
            if self.verbose {
                info!("Violated: {} = not({})", a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool_or(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_bool_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_bool_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self
            .args_extractor
            .extract_bool_value(C_TERM_INDEX, &constraint.call, &solution);

        if c != (a || b) {
            if self.verbose {
                info!("Violated: {} <-> {} \\/ {}", c, a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool_xor(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_bool_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_bool_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self
            .args_extractor
            .extract_bool_value(C_TERM_INDEX, &constraint.call, &solution);

        if c != (a ^ b) {
            if self.verbose {
                info!("Violated: {} <-> {} xor {}", c, a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    pub fn bool2int(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        let mut violation = 0;

        let a = self
            .args_extractor
            .extract_bool_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_int_value(B_TERM_INDEX, &constraint.call, &solution);

        if a as i64 != b {
            if self.verbose {
                info!("Violated: {} = {}", a, b);
            }
            violation = 1;
        }

        ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id,
        }
    }

    fn int_lin_left_term(
        &self,
        coeff: &[i64],
        bs_array: &[bool],
        verbose_terms: &mut String,
    ) -> i64 {
        let left_side_term: i64 = coeff
            .iter()
            .zip(bs_array.iter())
            .map(|(c, b)| {
                let val = if *b { 1_i64 } else { 0_i64 };
                if self.verbose {
                    write_verbose_output(verbose_terms, c, &val);
                }
                c * val
            })
            .sum();
        left_side_term
    }
}

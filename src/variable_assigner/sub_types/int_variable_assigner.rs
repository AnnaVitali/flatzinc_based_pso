use crate::args_extractor::sub_types::int_args_extractor::IntArgsExtractor;
use crate::data_utility::lin_expression::int_lin_left_term;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::evaluator::sub_types::int_functional_evaluator::{
    A_TERM_INDEX, B_TERM_INDEX, C_TERM_INDEX, COEFF_LIN_CONSTR_INDEX, CONST_LIN_CONSTR_INDEX,
    R_TERM_INDEX, VARS_LIN_CONSTR_INDEX,
};
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Identifier};
use std::cmp::{max, min};
use std::collections::HashMap;

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
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> i64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for array_int_element");
            if vars_identifier.contains(&defined_var) {
                args_extractor.extract_int_element_array(&call, &arrays, solution)
            } else {
                args_extractor.extract_int_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn int_abs(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> i64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_abs");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                a.abs()
            } else {
                args_extractor.extract_int_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn int_div(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> i64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_div");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                a / b
            } else {
                args_extractor.extract_int_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn int_eq(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> i64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_eq");
            if vars_identifier.contains(&defined_var) {
                args_extractor.extract_int_value(A_TERM_INDEX, &call, solution)
            } else {
                args_extractor.extract_int_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn int_eq_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_eq_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                a == b
            } else {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                let r = args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution);
                (a == b) == r
            }
        })
    }

    pub fn int_le_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_le_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                a <= b
            } else {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                let r = args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution);
                (a <= b) == r
            }
        })
    }

    pub fn int_lin_eq(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
        variable: &String,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> i64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let variable = variable.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut coeff = args_extractor.extract_int_coefficients_lin_expr(
                COEFF_LIN_CONSTR_INDEX,
                &call,
                &arrays,
            );
            let mut vars_involved =
                args_extractor.extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &call, &arrays);
            let var_idx = vars_involved.iter().position(|id| id == &variable);
            let term = args_extractor.extract_int_value(CONST_LIN_CONSTR_INDEX, &call, solution);
            if var_idx.is_none() {
                let left_side_term = int_lin_left_term(coeff, vars_involved, solution);
                let result = left_side_term - term;
                return result;
            }
            let var_idx = var_idx.unwrap();
            let var_coeff = coeff.remove(var_idx);
            vars_involved.remove(var_idx);
            let sum = int_lin_left_term(coeff, vars_involved, solution);
            let result = (term - sum) / var_coeff;
            result
        })
    }

    pub fn int_lin_eq_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_lin_eq_reif");
            if vars_identifier.contains(&defined_var) {
                let coeff = args_extractor.extract_int_coefficients_lin_expr(
                    COEFF_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let vars_involved = args_extractor.extract_var_values_lin_expr(
                    VARS_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let term =
                    args_extractor.extract_int_value(CONST_LIN_CONSTR_INDEX, &call, solution);
                let left_side_term = int_lin_left_term(coeff, vars_involved, solution);
                left_side_term == term
            } else {
                let coeff = args_extractor.extract_int_coefficients_lin_expr(
                    COEFF_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let vars_involved = args_extractor.extract_var_values_lin_expr(
                    VARS_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let term =
                    args_extractor.extract_int_value(CONST_LIN_CONSTR_INDEX, &call, solution);
                let r = args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution);
                let left_side_term = int_lin_left_term(coeff, vars_involved, solution);
                (left_side_term == term) == r
            }
        })
    }

    pub fn int_lin_le_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_lin_le_reif");
            if vars_identifier.contains(&defined_var) {
                let coeff = args_extractor.extract_int_coefficients_lin_expr(
                    COEFF_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let vars_involved = args_extractor.extract_var_values_lin_expr(
                    VARS_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let term =
                    args_extractor.extract_int_value(CONST_LIN_CONSTR_INDEX, &call, solution);
                let left_side_term = int_lin_left_term(coeff, vars_involved, solution);
                left_side_term <= term
            } else {
                let coeff = args_extractor.extract_int_coefficients_lin_expr(
                    COEFF_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let vars_involved = args_extractor.extract_var_values_lin_expr(
                    VARS_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let term =
                    args_extractor.extract_int_value(CONST_LIN_CONSTR_INDEX, &call, solution);
                let r = args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution);
                let left_side_term = int_lin_left_term(coeff, vars_involved, solution);
                (left_side_term <= term) == r
            }
        })
    }

    pub fn int_lin_ne_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_lin_ne_reif");
            if vars_identifier.contains(&defined_var) {
                let coeff = args_extractor.extract_int_coefficients_lin_expr(
                    COEFF_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let vars_involved = args_extractor.extract_var_values_lin_expr(
                    VARS_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let term =
                    args_extractor.extract_int_value(CONST_LIN_CONSTR_INDEX, &call, solution);
                let left_side_term = int_lin_left_term(coeff, vars_involved, solution);
                left_side_term != term
            } else {
                let coeff = args_extractor.extract_int_coefficients_lin_expr(
                    COEFF_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let vars_involved = args_extractor.extract_var_values_lin_expr(
                    VARS_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let term =
                    args_extractor.extract_int_value(CONST_LIN_CONSTR_INDEX, &call, solution);
                let r = args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution);
                let left_side_term = int_lin_left_term(coeff, vars_involved, solution);
                (left_side_term != term) == r
            }
        })
    }

    pub fn int_lt_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_lt_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                a < b
            } else {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                let r = args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution);
                (a < b) == r
            }
        })
    }

    pub fn int_max(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> i64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_max");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                max(a, b)
            } else {
                args_extractor.extract_int_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn int_min(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> i64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_min");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                min(a, b)
            } else {
                args_extractor.extract_int_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn int_mod(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> i64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_mod");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                a % b
            } else {
                args_extractor.extract_int_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn int_ne_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_ne_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                a != b
            } else {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                let r = args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution);
                (a != b) == r
            }
        })
    }

    pub fn int_pow(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> i64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_pow");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                a.pow(b as u32)
            } else {
                args_extractor.extract_int_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn int_times(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> i64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int_times");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_int_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_int_value(B_TERM_INDEX, &call, solution);
                a * b
            } else {
                args_extractor.extract_int_value(C_TERM_INDEX, &call, solution)
            }
        })
    }
}

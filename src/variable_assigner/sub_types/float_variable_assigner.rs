use crate::args_extractor::sub_types::float_args_extractor::FloatArgsExtractor;
use crate::data_utility::lin_expression::float_lin_left_term;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::evaluator::sub_types::float_functional_evaluator::{
    A_TERM_INDEX, B_TERM_INDEX, C_TERM_INDEX, COEFF_LIN_CONSTR_INDEX, CONST_LIN_CONSTR_INDEX,
    R_TERM_INDEX, VARS_LIN_CONSTR_INDEX,
};
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Identifier};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct FloatVariableAssigner {
    args_extractor: FloatArgsExtractor,
    arrays: HashMap<Identifier, Array>,
}

impl FloatVariableAssigner {
    pub fn new(arrays: HashMap<Identifier, Array>) -> Self {
        let args_extractor = FloatArgsExtractor::new();

        Self {
            args_extractor,
            arrays,
        }
    }

    pub fn array_float_element(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for array_float_element");
            if vars_identifier.contains(&defined_var) {
                args_extractor.extract_float_element_in_array(&call, &arrays, solution)
            } else {
                args_extractor.extract_float_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_abs(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_abs");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                a.abs()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_div(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_div");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX, &call, solution);
                a / b
            } else {
                args_extractor.extract_float_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_eq(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_eq");
            if vars_identifier.contains(&defined_var) {
                args_extractor.extract_float_value(A_TERM_INDEX, &call, solution)
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_max(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_max");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX, &call, solution);
                a.max(b)
            } else {
                args_extractor.extract_float_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_min(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_min");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX, &call, solution);
                a.min(b)
            } else {
                args_extractor.extract_float_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_plus(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_plus");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX, &call, solution);
                a + b
            } else {
                args_extractor.extract_float_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_pow(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_pow");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX, &call, solution);
                a.powf(b)
            } else {
                args_extractor.extract_float_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_times(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_times");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX, &call, solution);
                a * b
            } else {
                args_extractor.extract_float_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_acos(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_acos");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                a.acos()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_acosh(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_acosh");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                a.acosh()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_asin(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_asin");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                a.asin()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_asinh(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_asinh");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                a.asinh()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_atan(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_atan");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                a.atan()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_atanh(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_atanh");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                a.atanh()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_cos(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_cos");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                a.cos()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_cosh(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_cosh");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                a.cosh()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_eq_reif(
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
                .expect("Expected a defined variable for float_eq_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX, &call, solution);
                a == b
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_exp(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_exp");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                a.exp()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_le_reif(
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
                .expect("Expected a defined variable for float_le_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX, &call, solution);
                a <= b
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_lin_eq(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
        variable: &String,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let arrays = self.arrays.clone();
        let variable = variable.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut coeff = args_extractor.extract_float_coefficients_lin_expr(
                COEFF_LIN_CONSTR_INDEX,
                &call,
                &arrays,
            );
            let term = args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &call, solution);
            let mut vars_involved =
                args_extractor.extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &call, &arrays);
            let var_idx = vars_involved.iter().position(|id| id == &variable);
            if var_idx.is_none() {
                let left_side_term = float_lin_left_term(coeff, vars_involved, solution);
                let result = left_side_term - term;
                return result;
            }
            let var_idx = var_idx.unwrap();
            let var_coeff = coeff.remove(var_idx);
            vars_involved.remove(var_idx);
            let sum = float_lin_left_term(coeff, vars_involved, solution);
            let result = (term - sum) / var_coeff;
            result
        })
    }

    pub fn float_lin_eq_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        let arrays = self.arrays.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_lin_eq_reif");
            if vars_identifier.contains(&defined_var) {
                let coeff = args_extractor.extract_float_coefficients_lin_expr(
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
                    args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &call, solution);
                let left_side_term = float_lin_left_term(coeff, vars_involved, solution);
                left_side_term == term
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_lin_le_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        let arrays = self.arrays.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_lin_le_reif");
            if vars_identifier.contains(&defined_var) {
                let coeff = args_extractor.extract_float_coefficients_lin_expr(
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
                    args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &call, solution);
                let left_side_term = float_lin_left_term(coeff, vars_involved, solution);
                left_side_term <= term
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_lin_lt_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        let arrays = self.arrays.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_lin_lt_reif");
            if vars_identifier.contains(&defined_var) {
                let coeff = args_extractor.extract_float_coefficients_lin_expr(
                    COEFF_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let vars_involved = args_extractor.extract_var_values_lin_expr(
                    VARS_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let term = args_extractor.extract_float_value(
                    CONST_LIN_CONSTR_INDEX,
                    &call,
                    solution,
                );
                let left_side_term = float_lin_left_term(coeff, vars_involved, solution);
                left_side_term < term
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_lin_ne_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        let arrays = self.arrays.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_lin_ne_reif");
            if vars_identifier.contains(&defined_var) {
                let coeff = args_extractor.extract_float_coefficients_lin_expr(
                    COEFF_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let vars_involved = args_extractor.extract_var_values_lin_expr(
                    VARS_LIN_CONSTR_INDEX,
                    &call,
                    &arrays,
                );
                let term = args_extractor.extract_float_value(
                    CONST_LIN_CONSTR_INDEX,
                    &call,
                    solution,
                );
                let left_side_term = float_lin_left_term(coeff, vars_involved, solution);
                left_side_term != term
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_ln(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_ln");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(
                    A_TERM_INDEX,
                    &call,
                    solution,
                );
                a.ln()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_log10(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_log10");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(
                    A_TERM_INDEX,
                    &call,
                    solution,
                );
                a.log10()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_log2(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_log2");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(
                    A_TERM_INDEX,
                    &call,
                    solution,
                );
                a.log2()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_lt_reif(
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
                .expect("Expected a defined variable for float_lt_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(
                    A_TERM_INDEX,
                    &call,
                    solution,
                );
                let b = args_extractor.extract_float_value(
                    B_TERM_INDEX,
                    &call,
                    solution,
                );
                a < b
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_ne_reif(
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
                .expect("Expected a defined variable for float_ne_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(
                    A_TERM_INDEX,
                    &call,
                    solution,
                );
                let b = args_extractor.extract_float_value(
                    B_TERM_INDEX,
                    &call,
                    solution,
                );
                a != b
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_sin(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_sin");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(
                    A_TERM_INDEX,
                    &call,
                    solution,
                );
                a.sin()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_sinh(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_sinh");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(
                    A_TERM_INDEX,
                    &call,
                    solution,
                );
                a.sinh()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_sqrt(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_sqrt");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(
                    A_TERM_INDEX,
                    &call,
                    solution,
                );
                a.sqrt()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_tan(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_tan");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(
                    A_TERM_INDEX,
                    &call,
                    solution,
                );
                a.tan()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn float_tanh(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for float_tanh");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(
                    A_TERM_INDEX,
                    &call,
                    solution,
                );
                a.tanh()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn int2float(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for int2float");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_int_value(
                    A_TERM_INDEX,
                    &call,
                    solution,
                );
                a as f64
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX, &call, solution)
            }
        })
    }
}

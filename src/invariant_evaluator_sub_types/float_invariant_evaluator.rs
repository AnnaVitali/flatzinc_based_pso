use crate::{args_extractor_sub_types::float_args_extractor::FloatArgsExtractor, invariant_evaluator::CallWithDefines};
use crate::logger::write_verbose_output;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Identifier};
use log::info;
use std::collections::HashMap;
use crate::data_utility::data_utility::ConstraintEvaluation;

pub const A_TERM_INDEX: usize = 0;
pub const B_TERM_INDEX: usize = 1;
pub const C_TERM_INDEX: usize = 2;
pub const R_TERM_INDEX: usize = 2;
pub const R_TERM_LIN_EXPR_REIF_INDEX: usize = 3;
pub const COEFF_LIN_CONSTR_INDEX: usize = 0;
pub const VARS_LIN_CONSTR_INDEX: usize = 1;
pub const CONST_LIN_CONSTR_INDEX: usize = 2;
pub const FLOAT_EQ_TOLERANCE: f64 = 1e-4;

#[derive(Debug, Clone, Default)]
pub struct FloatInvariantEvaluator {
    arrays: HashMap<Identifier, Array>,
    args_extractor: FloatArgsExtractor,
    verbose: bool,
}

impl FloatInvariantEvaluator {
    pub fn new(
        arrays: HashMap<Identifier, Array>,
        verbose: bool,
    ) -> Self {
        let args_extractor = FloatArgsExtractor::new();

        Self {
            arrays,
            args_extractor,
            verbose,
        }
    }

    pub fn array_float_element(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let array_value = self.args_extractor.extract_float_element_in_array(
            &constraint.call,
            &self.arrays,
            &solution,
        );
        let value = self
            .args_extractor
            .extract_float_value(C_TERM_INDEX, &constraint.call, &solution);

        if array_value != value {
            if self.verbose {
                info!("Violated: array value {} = {}", array_value, value);
            }
            violation = (array_value - value).abs();
        }
        
        ConstraintEvaluation {
            violation, 
            constraint_id: constraint.id
        }
    }

    pub fn float_abs(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_abs = a.abs();
        if !self.float_eq_tol(b, real_abs) {
            if self.verbose {
                info!("Violated: abs({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_abs);

        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_acos(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_acos = a.acos();
        if !self.float_eq_tol(b, real_acos) {
            if self.verbose {
                info!("Violated: acos({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_acos);
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_acosh(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_acosh = a.acosh();
        if !self.float_eq_tol(b, real_acosh) {
            if self.verbose {
                info!("Violated: acosh({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_acosh);
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_asin(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_asin = a.asin();

        if !self.float_eq_tol(b, real_asin) {
            if self.verbose {
                info!("Violated: asin({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_asin);
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_asinh(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_asinh = a.asinh();

        if !self.float_eq_tol(b, real_asinh) {
            if self.verbose {
                info!("Violated: asinh({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_asinh);
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_atan(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_atan = a.atan();

        if !self.float_eq_tol(b, real_atan) {
            if self.verbose {
                info!("Violated: atan({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_atan);
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_atanh(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_atanh = a.atanh();
        if !self.float_eq_tol(b, real_atanh) {
            if self.verbose {
                info!("Violated: atanh({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_atanh);
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_cos(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_cos = a.cos();

        if !self.float_eq_tol(b, real_cos) {
            if self.verbose {
                info!("Violated: cos({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_cos);
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_cosh(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_cosh = a.cosh();

        if !self.float_eq_tol(b, real_cosh) {
            if self.verbose {
                info!("Violated: cosh({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_cosh);
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_div(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self
            .args_extractor
            .extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_float_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self
            .args_extractor
            .extract_float_value(C_TERM_INDEX, &constraint.call, &solution);

        if b == 0.0 {
            violation = 1.0;

            ConstraintEvaluation {
                violation,
                constraint_id: constraint.id
            }
        } else {
            let result = a / b;
            if !self.float_eq_tol(c, result) {
                if self.verbose {
                    info!("Violated: {} / {} = {}", a, b, c);
                }
                violation = self.float_eq_violation(c, result);
            }

            ConstraintEvaluation {
                violation,
                constraint_id: constraint.id
            }
        }
    }

    pub fn float_eq(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self
            .args_extractor
            .extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        if !self.float_eq_tol(a, b) {
            if self.verbose {
                info!("Violated: {} = {}", a, b);
            }
            violation = self.float_eq_violation(a, b);
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }   
    }

    pub fn float_eq_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a =
            self.args_extractor
                .extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b =
            self.args_extractor
                .extract_float_value(B_TERM_INDEX, &constraint.call, &solution);
        let r =
            self.args_extractor
                .extract_bool_value(R_TERM_INDEX, &constraint.call, &solution);

        if r != (a == b) {
            if self.verbose {
                info!("Violated: {} <-> {} = {}", r, a, b);
            }
            violation = 1.0;
        }

        ConstraintEvaluation{
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_exp(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_exp = a.exp();

        if !self.float_eq_tol(b, real_exp) {
            if self.verbose {
                info!("Violated: exp({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_exp);
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_le(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self
            .args_extractor
            .extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self
            .args_extractor
            .extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        if a > b {
            if self.verbose {
                info!("Violated: {} <= {}", a, b);
            }
            violation = (a - b).abs();
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_le_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a =
            self.args_extractor
                .extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b =
            self.args_extractor
                .extract_float_value(B_TERM_INDEX, &constraint.call, &solution);
        let r =
            self.args_extractor
                .extract_bool_value(R_TERM_INDEX, &constraint.call, &solution);

        if r != (a <= b) {
            if self.verbose {
                info!("Violated: {} <-> {} <= {}", r, a, b);
            }
            violation = 1.0;
        }

        ConstraintEvaluation{
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_lin_eq(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let term = self.args_extractor.extract_float_value(
            CONST_LIN_CONSTR_INDEX,
            &constraint.call,
            &solution,
        );

        let mut verbose_terms = String::new();

        let left_side_term = self.float_lin_left_term(&coeff, solution, vars_involved.clone(), &mut verbose_terms);

        if !self.float_eq_tol(left_side_term, term) {
            if self.verbose {
                info!("Violated: {} = {}", verbose_terms, term);
            }
            violation = self.float_eq_violation(left_side_term, term);
        }

            ConstraintEvaluation {
                violation,
                constraint_id: constraint.id
            }
    }

    pub fn float_lin_eq_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let term = self.args_extractor.extract_float_value(
            CONST_LIN_CONSTR_INDEX,
            &constraint.call,
            &solution,
        );
        let r =
            self.args_extractor
                .extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call, &solution);

        let mut verbose_terms = String::new();

        let left_side_term = self.float_lin_left_term(&coeff, solution, vars_involved, &mut verbose_terms);

        if r != (left_side_term == term) {
            if self.verbose {
                info!("Violated: {} <-> {} = {}", r, verbose_terms, term);
            }
            violation = 1.0
        }

        ConstraintEvaluation{
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_lin_le(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let term = self.args_extractor.extract_float_value(
            CONST_LIN_CONSTR_INDEX,
            &constraint.call,
            &solution,
        );

        let mut verbose_terms = String::new();

        let left_side_term = self.float_lin_left_term(&coeff, solution, vars_involved, &mut verbose_terms);

        if left_side_term > term {
            if self.verbose {
                info!("Violated: {} <= {}", verbose_terms, term);
            }
            violation = (left_side_term - term).abs();
                
            }

            ConstraintEvaluation {
                    violation,
                    constraint_id: constraint.id
            }
    }

    pub fn float_lin_le_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let term = self.args_extractor.extract_float_value(
            CONST_LIN_CONSTR_INDEX,
            &constraint.call,
            &solution,
        );
        let r =
            self.args_extractor
                .extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call, &solution);

        let mut verbose_terms = String::new();

        let left_side_term = self.float_lin_left_term(&coeff, solution, vars_involved, &mut verbose_terms);

        if r != (left_side_term <= term) {
            if self.verbose {
                info!("Violated: {} <-> {} <= {}", r, verbose_terms, term);
            }
            violation = 1.0
        }

        ConstraintEvaluation{
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_lin_lt(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let term = self.args_extractor.extract_float_value(
            CONST_LIN_CONSTR_INDEX,
            &constraint.call,
            &solution,
        );

        let mut verbose_terms = String::new();

        let left_side_term = self.float_lin_left_term(&coeff, solution, vars_involved.clone(), &mut verbose_terms);

        if left_side_term >= term {
            if self.verbose {
                info!("Violated: {} < {}", verbose_terms, term);
            }
            violation = (left_side_term - term).abs() + 1.0;
        } 

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
        
    }

    pub fn float_lin_lt_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let term = self.args_extractor.extract_float_value(
            CONST_LIN_CONSTR_INDEX,
            &constraint.call,
            &solution,
        );
        let r =
            self.args_extractor
                .extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call, &solution);

        let mut verbose_terms = String::new();

        let left_side_term = self.float_lin_left_term(&coeff, solution, vars_involved, &mut verbose_terms);

        if r != (left_side_term < term) {
            if self.verbose {
                info!("Violated: {} <-> {} < {}", r, verbose_terms, term);
            }
            violation = 1.0
        }

        ConstraintEvaluation{
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_lin_ne(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let term = self.args_extractor.extract_float_value(
            CONST_LIN_CONSTR_INDEX,
            &constraint.call,
            &solution,
        );

        let mut verbose_terms = String::new();

        let left_side_term = self.float_lin_left_term(&coeff, solution, vars_involved.clone(), &mut verbose_terms);

        if left_side_term == term {
            if self.verbose {
                info!("Violated: {} != {}", verbose_terms, term);
            }
            violation = 1.0;

        } 

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
        
    }

    pub fn float_lin_ne_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let term = self.args_extractor.extract_float_value(
            CONST_LIN_CONSTR_INDEX,
            &constraint.call,
            &solution,
        );
        let r =
            self.args_extractor
                .extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call, &solution);
        let mut verbose_terms = String::new();

        let left_side_term = self.float_lin_left_term(&coeff, solution, vars_involved, &mut verbose_terms);

        if !r != !(left_side_term != term) {
            if self.verbose {
                info!("Violated: {} <-> {} != {}", r, verbose_terms, term);
            }
            violation = 1.0
        }

        ConstraintEvaluation{
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_ln(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_ln = a.ln();
        if !self.float_eq_tol(b, real_ln) {
            if self.verbose {
                info!("Violated: ln({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_ln);
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_log10(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_log10 = a.log10();
        if !self.float_eq_tol(b, real_log10) {
            if self.verbose {
                info!("Violated: log10({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_log10);
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_log2(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_log2 = a.log2();
        if !self.float_eq_tol(b, real_log2) {
            if self.verbose {
                info!("Violated: log2({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_log2);
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_lt(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        if a >= b {
            if self.verbose {
                info!("Violated: {} < {}", a, b);
            }
            let diff = (a - b).abs() + 1.0;
            violation = diff;
        }

        ConstraintEvaluation {
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_lt_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a =
            self.args_extractor
                .extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b =
            self.args_extractor
                .extract_float_value(B_TERM_INDEX, &constraint.call, &solution);
        let r =
            self.args_extractor
                .extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call, &solution);

        if r != (a < b) {
            if self.verbose {
                info!("Violated: {} <-> {} < {}", r, a, b);
            }
            violation = 1.0;
        }

        ConstraintEvaluation{
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_max(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a =
            self.args_extractor
                .extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b =
            self.args_extractor
                .extract_float_value(B_TERM_INDEX, &constraint.call, &solution);
        let c =
            self.args_extractor
                .extract_float_value(C_TERM_INDEX, &constraint.call, &solution);

        let max = a.max(b);
        if !self.float_eq_tol(c, max) {
            if self.verbose {
                info!("Violated: max({},{}) = {}", a, b, c);
            }
            violation = self.float_eq_violation(c, max);
        }

        ConstraintEvaluation{
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_min(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a =
            self.args_extractor
                .extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b =
            self.args_extractor
                .extract_float_value(B_TERM_INDEX, &constraint.call, &solution);
        let c =
            self.args_extractor
                .extract_float_value(C_TERM_INDEX, &constraint.call, &solution);

        let min = a.min(b);
        if !self.float_eq_tol(c, min) {
            if self.verbose {
                info!("Violated: min({},{}) = {}", a, b, c);
            }
            violation = self.float_eq_violation(c, min);
        }

        ConstraintEvaluation{
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_ne(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a =
            self.args_extractor
                .extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b =
            self.args_extractor
                .extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        if a == b {
            if self.verbose {
                info!("Violated: {} != {}", a, b);
            }
            violation = 1.0;
        }

        ConstraintEvaluation{
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_ne_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a =
            self.args_extractor
                .extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b =
            self.args_extractor
                .extract_float_value(B_TERM_INDEX, &constraint.call, &solution);
        let r =
            self.args_extractor
                .extract_bool_value(R_TERM_INDEX, &constraint.call, &solution);

        if r != (a != b) {
            if self.verbose {
                info!("Violated: {} <-> {} != {}", r, a, b);
            }
            violation = 1.0;
        }

        ConstraintEvaluation{
            violation,
            constraint_id: constraint.id
        }
    }

    pub fn float_plus(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, &solution);

        let result = a + b;
        if !self.float_eq_tol(c, result) {
            if self.verbose {
                info!("Violated: {} + {} = {}", a, b, c);
            }
            violation = self.float_eq_violation(c, result);
        }

        ConstraintEvaluation { 
            violation, 
            constraint_id: constraint.id
        }
    }

    pub fn float_pow(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, &solution);

        let result = a.powf(b);
        if !self.float_eq_tol(c, result) {
            if self.verbose {
                info!("Violated: {} ^ {} = {}", a, b, c);
            }
            violation = self.float_eq_violation(c, result);
        }

        ConstraintEvaluation { 
            violation, 
            constraint_id: constraint.id
        }
    }

    pub fn float_sin(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_sin = a.sin();
        if !self.float_eq_tol(b, real_sin) {
            if self.verbose {
                info!("Violated: sin({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_sin);
        }

        ConstraintEvaluation { 
            violation, 
            constraint_id: constraint.id 
        }
    }

    pub fn float_sinh(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_sinh = a.sinh();
        if !self.float_eq_tol(b, real_sinh) {
            if self.verbose {
                info!("Violated: sinh({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_sinh);
        }

        ConstraintEvaluation { 
            violation, 
            constraint_id: constraint.id 
        }
    }

    pub fn float_sqrt(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_sqrt = a.sqrt();
        if !self.float_eq_tol(b, real_sqrt) {
            if self.verbose {
                info!("Violated: sqrt({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_sqrt);
        }

        ConstraintEvaluation { 
            violation, 
            constraint_id: constraint.id 
        }
    }

    pub fn float_tan(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_tan = a.tan();
        if !self.float_eq_tol(b, real_tan) {
            if self.verbose {
                info!("Violated: tan({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_tan);
        }

        ConstraintEvaluation { 
            violation, 
            constraint_id: constraint.id 
        }
    }

    pub fn float_tanh(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        let real_tanh = a.tanh();
        if !self.float_eq_tol(b, real_tanh) {
            if self.verbose {
                info!("Violated: tanh({}) = {}", a, b);
            }
            violation = self.float_eq_violation(b, real_tanh);
        }

        ConstraintEvaluation { 
            violation, 
            constraint_id: constraint.id 
        }
    }

    pub fn float_times(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;
        let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);
        let c = self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, &solution);

        let result = a * b;
        if !self.float_eq_tol(c, result) {
            if self.verbose {
                info!("Violated: {} * {} = {}", a, b, c);
            }
            violation = self.float_eq_violation(c, result);
        }

        ConstraintEvaluation { 
            violation, 
            constraint_id: constraint.id 
        }
    }


    pub fn int2float(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0.0;

        let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, &solution);
        let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &solution);

        if a as f64 != b {
            if self.verbose {
                info!("Violated: {} = {}", a, b);
            }
            violation = (a as f64 - b).abs();
        }

        ConstraintEvaluation{
            violation,
            constraint_id: constraint.id
        }
    }

    fn float_lin_left_term(
        &self,
        coeff: &Vec<f64>,
        solution: &HashMap<String, VariableValue>,
        vars_involved: Vec<Identifier>,
        verbose_terms: &mut String,
    ) -> f64 {
        let left_side_term: f64 = coeff
            .iter()
            .zip(vars_involved.iter())
            .map(|(c, id)| {
                let var_val = solution
                    .get(id)
                    .and_then(|int_val| match int_val {
                        VariableValue::Float(int_val) => Some(*int_val),
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

    #[inline]
    fn float_abs_diff(&self, a: f64, b: f64) -> f64 {
        (a - b).abs()
    }

    #[inline]
    fn float_eq_tol(&self, a: f64, b: f64) -> bool {
        self.float_abs_diff(a, b) <= FLOAT_EQ_TOLERANCE
    }

    #[inline]
    fn float_eq_violation(&self, a: f64, b: f64) -> f64 {
        (self.float_abs_diff(a, b) - FLOAT_EQ_TOLERANCE).max(0.0)
    }
}

use crate::args_extractor_sub_types::float_args_extractor::FloatArgsExtractor;
use crate::invariant_evaluator::CallWithDefines;
use crate::invariant_evaluator_sub_types::float_invariant_evaluator::{A_TERM_INDEX, B_TERM_INDEX, C_TERM_INDEX, COEFF_LIN_CONSTR_INDEX, CONST_LIN_CONSTR_INDEX, R_TERM_INDEX, VARS_LIN_CONSTR_INDEX};
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Argument, Array, Identifier, Literal};
use regex::escape;
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
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for array_float_element");
        if vars_identifier.contains(&defined_var) {
            self.args_extractor.extract_float_element_in_array(&constraint.call, &self.arrays, complete_solution)
        } else {
            self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_abs(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_abs");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            a.abs()
        } else {
            self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }
    // Repeat this pattern for float_div, float_eq, float_max, float_min, float_plus, float_pow, float_times
    pub fn float_div(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_div");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a / b
        } else {
            self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_eq(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_eq");
        if vars_identifier.contains(&defined_var) {
            self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution)
        } else {
            self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_max(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_max");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a.max(b)
        } else {
            self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_min(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_min");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a.min(b)
        } else {
            self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_plus(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_plus");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a + b
        } else {
            self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_pow(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_pow");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a.powf(b)
        } else {
            self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_times(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_times");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a * b
        } else {
            self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_acos(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
         let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_times");
         if vars_identifier.contains(&defined_var) {
             let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
             a.acos()
         }else{
                self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_acosh(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_times");
         if vars_identifier.contains(&defined_var) {
             let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
             a.acosh()
         }else{
                self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_asin(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_times");
         if vars_identifier.contains(&defined_var) {
             let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
             a.asin()
         }else{
                self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_asinh(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_times");
         if vars_identifier.contains(&defined_var) {
             let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
             a.asinh()
         }else{
                self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_atan(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_times");
         if vars_identifier.contains(&defined_var) {
             let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
             a.atan()
         }else{
                self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_atanh(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_times");
         if vars_identifier.contains(&defined_var) {
             let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
             a.atanh()
         }else{
                self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_cos(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_times");
         if vars_identifier.contains(&defined_var) {
             let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
             a.cos()
         }else{
                self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_cosh(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_times");
         if vars_identifier.contains(&defined_var) {
             let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
             a.cosh()
         }else{
                self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_eq_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_eq_reif");
         if vars_identifier.contains(&defined_var) {
        let a =
            self.args_extractor
                .extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
        let b =
            self.args_extractor
                .extract_float_value(B_TERM_INDEX, &constraint.call, complete_solution);

        a == b
         }else{

            self.args_extractor
                .extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution  )

         }
    }

    pub fn float_exp(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
       let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_exp");
         if vars_identifier.contains(&defined_var) {
             let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
             a.exp()
         }else{
                self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_le_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_le_reif");
         if vars_identifier.contains(&defined_var) {
        let a =
            self.args_extractor
                .extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
        let b =
            self.args_extractor
                .extract_float_value(B_TERM_INDEX, &constraint.call, complete_solution);

        a <= b
        }else{

            self.args_extractor
                .extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution  )

         }
    }

    pub fn float_lin_eq(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
        variable: &String,
    ) -> f64 {
        let mut coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let term = self.args_extractor.extract_float_value(
            CONST_LIN_CONSTR_INDEX,
            &constraint.call,
            complete_solution,
        );
        let mut vars_involved = self
            .extract_lin_expr_variables(constraint);
        let var_idx = vars_involved.iter().position(|id| id == variable);

        if var_idx.is_none() {
            let left_side_term = self.float_lin_left_term(coeff, vars_involved, complete_solution);
            let result = left_side_term - term;
            return result;
        }

        let var_idx = var_idx.unwrap();

        let var_coeff = coeff.remove(var_idx);
        vars_involved.remove(var_idx);

        let sum = self.float_lin_left_term(coeff, vars_involved, complete_solution);

        let result = (term - sum) / var_coeff;
        
        result
    }

    pub fn float_lin_eq_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_le_reif");
        if vars_identifier.contains(&defined_var) {
            
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .extract_lin_expr_variables(constraint);
        let term = self.args_extractor.extract_float_value(
            CONST_LIN_CONSTR_INDEX,
            &constraint.call,
            complete_solution,
        );

        let left_side_term = self.float_lin_left_term(coeff, vars_involved, complete_solution);

        left_side_term == term
    }else{
        self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    fn extract_lin_expr_variables(&self, constraint: &CallWithDefines) -> Vec<Identifier> {
        match constraint.call.args.get(VARS_LIN_CONSTR_INDEX) {
            Some(Argument::Array(lits)) => lits
                .iter()
                .filter_map(|lit| match lit {
                    Literal::Identifier(id) => Some(id.clone()),
                    _ => None,
                })
                .collect(),
            Some(Argument::Literal(Literal::Identifier(array_name))) => self
                .arrays
                .get(&Identifier::from(array_name.clone()))
                .map(|arr| {
                    arr.contents
                        .iter()
                        .filter_map(|lit| match lit {
                            Literal::Identifier(id) => Some(id.clone()),
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    pub fn float_lin_le_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
         let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_le_reif");
        if vars_identifier.contains(&defined_var) {
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
            complete_solution,
        );

        let left_side_term = self.float_lin_left_term(coeff, vars_involved, complete_solution);

        left_side_term <= term
        }else {

                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_lin_lt_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
         let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_le_reif");
        if vars_identifier.contains(&defined_var) {
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
            complete_solution,
        );

        let left_side_term = self.float_lin_left_term(coeff, vars_involved, complete_solution);

        left_side_term < term
    }else {
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_lin_ne_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
         let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_le_reif");
        if vars_identifier.contains(&defined_var) {

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
            complete_solution,
        );

        let left_side_term = self.float_lin_left_term(coeff, vars_involved, complete_solution);

        left_side_term != term
    }else {
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_ln(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_ln");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            a.ln()
        } else {
            self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_log10(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_log10");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            a.log10()
        } else {
            self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_log2(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_log2");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            a.log2()
        } else {
            self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_lt_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
         let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_le_reif");
        if vars_identifier.contains(&defined_var) {
        let a =
            self.args_extractor
                .extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
        let b =
            self.args_extractor
                .extract_float_value(B_TERM_INDEX, &constraint.call, complete_solution);

        a < b
        }else{
            self.args_extractor
                .extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution  )
        }
    }

    pub fn float_ne_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
         let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_le_reif");
        if vars_identifier.contains(&defined_var) {
            let a =
                self.args_extractor
                    .extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b =
                self.args_extractor
                .extract_float_value(B_TERM_INDEX, &constraint.call, complete_solution);

            a != b
        } else {
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }



    pub fn float_sin(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
         let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_le_reif");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            a.sin()
        }else{
                self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
       
    }

    pub fn float_sinh(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_le_reif");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            a.sinh()
        }else{
                self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_sqrt(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_sqrt");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            a.sqrt()
        } else {
            self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_tan(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_tan");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            a.tan()
        } else {
            self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    pub fn float_tanh(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_tanh");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, complete_solution);
            a.tanh()
        } else {
            self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }


    pub fn int2float(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for int2float");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, complete_solution);
            a as f64
        } else {
            self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, &complete_solution)
        }
    }

    fn float_lin_left_term(
        &self,
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
                //println!("c: {:?} var_val: {:?} for variable {}", c, var_val, id);
                c * var_val
            })
            .sum();
        left_side_term
    }
}

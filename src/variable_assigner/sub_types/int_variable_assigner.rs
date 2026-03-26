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

/// Struct responsible for assigning integer variables based on constraints and solutions.
///
/// # Fields
/// * `args_extractor` - Extracts arguments for integer constraints.
/// * `arrays` - Stores arrays mapped by their identifiers.
#[derive(Debug, Clone, Default)]
pub struct IntVariableAssigner {
    /// An instance of `IntArgsExtractor` used to extract arguments from integer constraints.
    args_extractor: IntArgsExtractor,
    /// A hashmap that maps identifiers to their corresponding arrays, used for resolving array references in constraints.
    arrays: HashMap<Identifier, Array>,
}

impl IntVariableAssigner {
    /// Creates a new `IntVariableAssigner` with the provided arrays.
    ///
    /// # Arguments
    /// * `arrays` - A map from identifiers to arrays used in integer constraints.
    ///
    /// # Returns
    /// A new instance of `IntVariableAssigner`.
    pub fn new(arrays: HashMap<Identifier, Array>) -> Self {
        let args_extractor = IntArgsExtractor::new();

        Self {
            args_extractor,
            arrays,
        }
    }

    /// Returns a closure that evaluates the `array_int_element` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the integer value from the array or a specific value depending on the defined variable.
    pub fn array_int_element(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_abs` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the absolute value of an integer or a specific value depending on the defined variable.
    pub fn int_abs(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_div` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the division of two integer values or a specific value depending on the defined variable.
    pub fn int_div(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_eq` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the equality of two integer values or a specific value depending on the defined variable.
    pub fn int_eq(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether two integer values are equal, or a boolean value depending on the defined variable.
    pub fn int_eq_reif(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one integer value is less than or equal to another, or a boolean value depending on the defined variable.
    pub fn int_le_reif(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_lin_eq` constraint for a specific variable.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    /// * `variable` - The variable to solve for in the linear equation.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the value of the specified variable in the linear equation.
    pub fn int_lin_eq(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_lin_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear equation holds, or a boolean value depending on the defined variable.
    pub fn int_lin_eq_reif(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_lin_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear expression is less than or equal to the term, or a boolean value depending on the defined variable.
    pub fn int_lin_le_reif(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_lin_ne_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear expression is not equal to the term, or a boolean value depending on the defined variable.
    pub fn int_lin_ne_reif(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_lt_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one integer value is less than another, or a boolean value depending on the defined variable.
    pub fn int_lt_reif(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_max` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the maximum of two integer values or a specific value depending on the defined variable.
    pub fn int_max(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_min` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the minimum of two integer values or a specific value depending on the defined variable.
    pub fn int_min(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_mod` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the remainder of the division of two integer values or a specific value depending on the defined variable.
    pub fn int_mod(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_ne_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether two integer values are not equal, or a boolean value depending on the defined variable.
    pub fn int_ne_reif(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_pow` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the result of raising one integer to the power of another or a specific value depending on the defined variable.
    pub fn int_pow(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `int_times` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the product of two integer values or a specific value depending on the defined variable.
    pub fn int_times(
        &self,
        constraint: &CallWithDefines,
        
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

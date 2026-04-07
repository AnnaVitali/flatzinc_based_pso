use crate::args_extractor::sub_types::float_args_extractor::FloatArgsExtractor;
use crate::data_utility::lin_expression::float_lin_left_term;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::evaluator::sub_types::float_evaluator::{
    A_TERM_INDEX, B_TERM_INDEX, C_TERM_INDEX, COEFF_LIN_CONSTR_INDEX, CONST_LIN_CONSTR_INDEX,
    R_TERM_INDEX, VARS_LIN_CONSTR_INDEX,
};
use crate::solution_provider::VariableValue;
use flatzinc_serde::Array;
use std::collections::HashMap;

/// Struct responsible for assigning float variables based on constraints and solutions.
///
/// # Fields
/// * `args_extractor` - Extracts arguments for float constraints.
/// * `arrays` - Stores arrays mapped by their identifiers.
#[derive(Debug, Clone, Default)]
pub struct FloatVariableAssigner {
    /// An instance of `FloatArgsExtractor` used to extract arguments from float constraints.
    args_extractor: FloatArgsExtractor,
    /// A hashmap that maps identifiers to their corresponding arrays, used for resolving array references in constraints.
    arrays: HashMap<String, Array>,
}

impl FloatVariableAssigner {
    /// Creates a new `FloatVariableAssigner` with the provided arrays.
    ///
    /// # Arguments
    /// * `arrays` - A map from identifiers to arrays used in float constraints.
    ///
    /// # Returns
    /// A new instance of `FloatVariableAssigner`.
    pub fn new(arrays: HashMap<String, Array>) -> Self {
        let args_extractor = FloatArgsExtractor::new();

        Self {
            args_extractor,
            arrays,
        }
    }

    /// Returns a closure that evaluates the `array_float_element` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the float value from the array or a specific value depending on the defined variable.
    pub fn array_float_element(
        &self,
        constraint: &CallWithDefines,
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
                args_extractor.extract_float_value(C_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_abs` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the absolute value of a float or a specific value depending on the defined variable.
    pub fn float_abs(
        &self,
        constraint: &CallWithDefines,
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.abs()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_div` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the division of two float values or a specific value depending on the defined variable.
    pub fn float_div(
        &self,
        constraint: &CallWithDefines,
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution);
                a / b
            } else {
                args_extractor.extract_float_value(C_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_eq` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the equality of two float values or a specific value depending on the defined variable.
    pub fn float_eq(
        &self,
        constraint: &CallWithDefines,       
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
                args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution)
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_max` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the maximum of two float values or a specific value depending on the defined variable.
    pub fn float_max(
        &self,
        constraint: &CallWithDefines,        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.max(b)
            } else {
                args_extractor.extract_float_value(C_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_min` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the minimum of two float values or a specific value depending on the defined variable.
    pub fn float_min(
        &self,
        constraint: &CallWithDefines,       
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.min(b)
            } else {
                args_extractor.extract_float_value(C_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_plus` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the sum of two float values or a specific value depending on the defined variable.
    pub fn float_plus(
        &self,
        constraint: &CallWithDefines,        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution);
                a + b
            } else {
                args_extractor.extract_float_value(C_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_pow` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the result of raising one float to the power of another or a specific value depending on the defined variable.
    pub fn float_pow(
        &self,
        constraint: &CallWithDefines,       
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.powf(b)
            } else {
                args_extractor.extract_float_value(C_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_times` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the product of two float values or a specific value depending on the defined variable.
    pub fn float_times(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution);
                a * b
            } else {
                args_extractor.extract_float_value(C_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_acos` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the arccosine of a float value or a specific value depending on the defined variable.
    pub fn float_acos(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.acos()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_acosh` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the hyperbolic arccosine of a float value or a specific value depending on the defined variable.
    pub fn float_acosh(
        &self,
        constraint: &CallWithDefines,       
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.acosh()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_asin` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the arcsine of a float value or a specific value depending on the defined variable.
    pub fn float_asin(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.asin()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_asinh` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the hyperbolic arcsine of a float value or a specific value depending on the defined variable.
    pub fn float_asinh(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.asinh()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_atan` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the arctangent of a float value or a specific value depending on the defined variable.
    pub fn float_atan(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.atan()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_atanh` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the hyperbolic arctangent of a float value or a specific value depending on the defined variable.
    pub fn float_atanh(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.atanh()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_cos` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the cosine of a float value or a specific value depending on the defined variable.
    pub fn float_cos(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.cos()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_cosh` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the hyperbolic cosine of a float value or a specific value depending on the defined variable.
    pub fn float_cosh(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.cosh()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether two float values are equal, or a boolean value depending on the defined variable.
    pub fn float_eq_reif(
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
                .expect("Expected a defined variable for float_eq_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution);
                a == b
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_exp` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the exponential of a float value or a specific value depending on the defined variable.
    pub fn float_exp(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.exp()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one float value is less than or equal to another, or a boolean value depending on the defined variable.
    pub fn float_le_reif(
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
                .expect("Expected a defined variable for float_le_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution);
                a <= b
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_lin_eq` constraint for a specific variable.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    /// * `variable` - The variable to solve for in the linear equation.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the value of the specified variable in the linear equation.
    pub fn float_lin_eq(
        &self,
        constraint: &CallWithDefines,
        
        variable: &String,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let arrays = self.arrays.clone();
        let variable = variable.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut coeff = args_extractor.extract_float_coefficients_lin_expr(
                COEFF_LIN_CONSTR_INDEX.try_into().unwrap(),
                &call,
                &arrays,
            );
            let term = args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX.try_into().unwrap(), &call, solution);
            let mut vars_involved =
                args_extractor.extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX.try_into().unwrap(), &call, &arrays);
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

    /// Returns a closure that evaluates the `float_lin_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear equation holds, or a boolean value depending on the defined variable.
    pub fn float_lin_eq_reif(
        &self,
        constraint: &CallWithDefines,
        
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
                    COEFF_LIN_CONSTR_INDEX.try_into().unwrap(),
                    &call,
                    &arrays,
                );
                let vars_involved = args_extractor.extract_var_values_lin_expr(
                    VARS_LIN_CONSTR_INDEX.try_into().unwrap(),
                    &call,
                    &arrays,
                );
                let term =
                    args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX.try_into().unwrap(), &call, solution);
                let left_side_term = float_lin_left_term(coeff, vars_involved, solution);
                left_side_term == term
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_lin_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear expression is less than or equal to the term, or a boolean value depending on the defined variable.
    pub fn float_lin_le_reif(
        &self,
        constraint: &CallWithDefines,
        
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
                    COEFF_LIN_CONSTR_INDEX.try_into().unwrap(),
                    &call,
                    &arrays,
                );
                let vars_involved = args_extractor.extract_var_values_lin_expr(
                    VARS_LIN_CONSTR_INDEX.try_into().unwrap(),
                    &call,
                    &arrays,
                );
                let term =
                    args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX.try_into().unwrap(), &call, solution);
                let left_side_term = float_lin_left_term(coeff, vars_involved, solution);
                left_side_term <= term
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_lin_lt_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear expression is less than the term, or a boolean value depending on the defined variable.
    pub fn float_lin_lt_reif(
        &self,
        constraint: &CallWithDefines,
        
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
                    COEFF_LIN_CONSTR_INDEX.try_into().unwrap(),
                    &call,
                    &arrays,
                );
                let vars_involved = args_extractor.extract_var_values_lin_expr(
                    VARS_LIN_CONSTR_INDEX.try_into().unwrap(),
                    &call,
                    &arrays,
                );
                let term =
                    args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX.try_into().unwrap(), &call, solution);
                let left_side_term = float_lin_left_term(coeff, vars_involved, solution);
                left_side_term < term
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_lin_ne_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear expression is not equal to the term, or a boolean value depending on the defined variable.
    pub fn float_lin_ne_reif(
        &self,
        constraint: &CallWithDefines,
        
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
                    COEFF_LIN_CONSTR_INDEX.try_into().unwrap(),
                    &call,
                    &arrays,
                );
                let vars_involved = args_extractor.extract_var_values_lin_expr(
                    VARS_LIN_CONSTR_INDEX.try_into().unwrap(),
                    &call,
                    &arrays,
                );
                let term =
                    args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX.try_into().unwrap(), &call, solution);
                let left_side_term = float_lin_left_term(coeff, vars_involved, solution);
                left_side_term != term
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_ln` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the natural logarithm of a float value or a specific value depending on the defined variable.
    pub fn float_ln(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.ln()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_log10` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the base-10 logarithm of a float value or a specific value depending on the defined variable.
    pub fn float_log10(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.log10()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_log2` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the base-2 logarithm of a float value or a specific value depending on the defined variable.
    pub fn float_log2(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.log2()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_lt_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one float value is less than another, or a boolean value depending on the defined variable.
    pub fn float_lt_reif(
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
                .expect("Expected a defined variable for float_lt_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution);
                a < b
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_ne_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether two float values are not equal, or a boolean value depending on the defined variable.
    pub fn float_ne_reif(
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
                .expect("Expected a defined variable for float_ne_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                let b = args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution);
                a != b
            } else {
                args_extractor.extract_bool_value(R_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_sin` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the sine of a float value or a specific value depending on the defined variable.
    pub fn float_sin(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.sin()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_sinh` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the hyperbolic sine of a float value or a specific value depending on the defined variable.
    pub fn float_sinh(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.sinh()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_sqrt` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the square root of a float value or a specific value depending on the defined variable.
    pub fn float_sqrt(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.sqrt()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_tan` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the tangent of a float value or a specific value depending on the defined variable.
    pub fn float_tan(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.tan()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `float_tanh` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the hyperbolic tangent of a float value or a specific value depending on the defined variable.
    pub fn float_tanh(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_float_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a.tanh()
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `int2float` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the float representation of an integer value or a specific value depending on the defined variable.
    pub fn int2float(
        &self,
        constraint: &CallWithDefines,
        
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
                let a = args_extractor.extract_int_value(A_TERM_INDEX.try_into().unwrap(), &call, solution);
                a as f64
            } else {
                args_extractor.extract_float_value(B_TERM_INDEX.try_into().unwrap(), &call, solution)
            }
        })
    }
}

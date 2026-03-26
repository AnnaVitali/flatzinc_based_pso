use crate::args_extractor::sub_types::bool_args_extractor::BoolArgsExtractor;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::evaluator::sub_types::bool_functional_evaluator::{
    A_TERM_INDEX, AS_ARRAY_INDEX, B_TERM_INDEX, BS_ARRAY_INDEX, C_TERM_INDEX,
};
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Identifier};
use std::collections::HashMap;

/// Struct responsible for assigning boolean variables based on constraints and solutions.
///
/// # Fields
/// * `args_extractor` - Extracts arguments for boolean constraints.
/// * `arrays` - Stores arrays mapped by their identifiers.
#[derive(Debug, Clone, Default)]
pub struct BoolVariableAssigner {
    /// An instance of `BoolArgsExtractor` used to extract arguments from boolean constraints.
    args_extractor: BoolArgsExtractor,
    /// A hashmap that maps identifiers to their corresponding arrays, used for resolving array references in constraints.
    arrays: HashMap<Identifier, Array>,
}

impl BoolVariableAssigner {
    /// Creates a new `BoolVariableAssigner` with the provided arrays.
    ///
    /// # Arguments
    /// * `arrays` - A map from identifiers to arrays used in boolean constraints.
    ///
    /// # Returns
    /// A new instance of `BoolVariableAssigner`.
    pub fn new(arrays: HashMap<Identifier, Array>) -> Self {
        let args_extractor = BoolArgsExtractor::new();

        Self {
            args_extractor,
            arrays,
        }
    }

    /// Returns a closure that evaluates the `array_bool_and` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns `true` if all elements in the array are true, or matches the expected result.
    pub fn array_bool_and(
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
                .expect("Expected a defined variable for array_bool_and");
            if vars_identifier.contains(&defined_var) {
                let as_array = args_extractor.extract_bool_array(
                    AS_ARRAY_INDEX,
                    &arrays,
                    &call,
                    solution,
                );
                as_array.iter().all(|&item| item)
            } else {
                let as_array = args_extractor.extract_bool_array(
                    AS_ARRAY_INDEX,
                    &arrays,
                    &call,
                    solution,
                );
                let r = args_extractor.extract_bool_value(
                    B_TERM_INDEX,
                    &call,
                    solution,
                );
                as_array.iter().all(|&item| item) == r
            }
        })
    }

    /// Returns a closure that evaluates the `array_bool_element` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the result of extracting a boolean element from the array.
    pub fn array_bool_element(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            args_extractor.extract_bool_element_array(
                AS_ARRAY_INDEX,
                &call,
                &arrays,
                solution,
            )
        })
    }

    /// Returns a closure that evaluates the `bool_and` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the logical AND of two boolean values, or a single value depending on the defined variable.
    pub fn bool_and(
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
                .expect("Expected a defined variable for bool_and");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution);
                a && b
            } else {
                args_extractor.extract_bool_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `bool_clause` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns `true` if any element in the array is true or if all elements in another array are not false.
    pub fn bool_clause(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let as_array = args_extractor.extract_bool_array(
                AS_ARRAY_INDEX,
                &arrays,
                &call,
                solution,
            );
            let bs_array = args_extractor.extract_bool_defined_elements_array(
                BS_ARRAY_INDEX,
                &arrays,
                &call,
                solution,
            );
            as_array.iter().any(|&item| item == true) || !bs_array.iter().any(|&item| item == false)
        })
    }

    /// Returns a closure that evaluates the `bool_eq` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the equality of two boolean values or a single value depending on the defined variable.
    pub fn bool_eq(
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
                .expect("Expected a defined variable for bool_eq");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                a
            } else {
                args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `bool_not` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the logical NOT of a boolean value or a single value depending on the defined variable.
    pub fn bool_not(
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
                .expect("Expected a defined variable for bool_not");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                !a
            } else {
                args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `bool_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether two boolean values are equal, or a single value depending on the defined variable.
    pub fn bool_eq_reif(
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
                .expect("Expected a defined variable for bool_eq_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution);
                a == b
            } else {
                args_extractor.extract_bool_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `bool_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one boolean value is less than or equal to another, or a single value depending on the defined variable.
    pub fn bool_le_reif(
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
                .expect("Expected a defined variable for bool_le_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution);
                a <= b
            } else {
                args_extractor.extract_bool_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `bool_lt_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one boolean value is less than another, or a single value depending on the defined variable.
    pub fn bool_lt_reif(
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
                .expect("Expected a defined variable for bool_lt_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution);
                a < b
            } else {
                args_extractor.extract_bool_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `bool_or` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the logical OR of two boolean values, or a single value depending on the defined variable.
    pub fn bool_or(
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
                .expect("Expected a defined variable for bool_or");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution);
                a || b
            } else {
                args_extractor.extract_bool_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `bool_xor` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the logical XOR of two boolean values, or a single value depending on the defined variable.
    pub fn bool_xor(
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
                .expect("Expected a defined variable for bool_xor");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution);
                a ^ b
            } else {
                args_extractor.extract_bool_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    /// Returns a closure that evaluates the `bool2int` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the integer representation of a boolean value, or an integer value depending on the defined variable.
    pub fn bool2int(
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
                .expect("Expected a defined variable for bool2int");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                a as i64
            } else {
                args_extractor.extract_int_value(B_TERM_INDEX, &call, solution)
            }
        })
    }
}

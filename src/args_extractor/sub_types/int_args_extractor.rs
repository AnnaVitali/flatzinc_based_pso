use crate::args_extractor::args_extractor::ArgsExtractor;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Argument, Array, Call, Identifier};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
/// A helper struct for extracting integer arguments from constraints, 
/// utilizing an internal `ArgsExtractor` for common extraction logic.
pub struct IntArgsExtractor {
    args_extractor: ArgsExtractor,
}

/// This struct provides methods to extract integer values, arrays of integers, and related information from constraint arguments, 
/// handling both literals and identifiers that reference variables or arrays in the solution.
impl IntArgsExtractor {
    /// Creates a new `IntArgsExtractor` with an internal `ArgsExtractor`.
    pub fn new() -> Self {
        Self {
            args_extractor: ArgsExtractor::new(),
        }
    }

    /// Extracts a mapping from indices to literal identifiers from the given arguments.
    ///
    /// # Arguments
    /// * `args` - A slice of `Argument` to extract identifiers from.
    ///
    /// # Returns
    /// A `HashMap` mapping indices to identifier names.
    pub fn extract_literal_identifiers_with_index(
        &self,
        args: &[Argument],
    ) -> HashMap<i64, String> {
        self.args_extractor
            .extract_literal_identifiers_with_index(args)
    }

    /// Extracts all literal identifiers from the given arguments.
    ///
    /// # Arguments
    /// * `args` - A slice of `Argument` to extract identifiers from.
    ///
    /// # Returns
    /// A vector of identifier names as `String`.
    pub fn extract_literal_identifiers(&self, args: &[Argument]) -> Vec<String> {
        self.args_extractor.extract_literal_identifiers(args)
    }

    /// Extracts variable identifiers from a linear expression in the constraint.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `constraint` - The constraint call containing the argument.
    /// * `arrays` - A map of identifiers to arrays for resolving array references.
    ///
    /// # Returns
    /// A vector of variable identifiers from the linear expression.
    pub fn extract_var_values_lin_expr(
        &self,
        index: usize,
        constraint: &Call,
        arrays: &HashMap<Identifier, Array>,
    ) -> Vec<Identifier> {
        self.args_extractor
            .extract_var_values_lin_expr(index, constraint, arrays)
    }

    /// Extracts a boolean value from the constraint at the specified argument index.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `constraint` - The constraint call containing the argument.
    /// * `solution` - The solution map for resolving identifiers.
    ///
    /// # Returns
    /// The extracted boolean value.
    pub fn extract_bool_value(
        &self,
        index: usize,
        constraint: &Call,
        solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let literal = self.args_extractor.extract_term(constraint, index);
        self.args_extractor.extract_bool_value(literal, solution)
    }

    /// Extracts an integer value from an array element in the constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call containing the argument.
    /// * `arrays` - A map of identifiers to arrays for resolving array references.
    /// * `solution` - The solution map for resolving identifiers.
    ///
    /// # Returns
    /// The extracted integer value from the array element.
    pub fn extract_int_element_array(
        &self,
        constraint: &Call,
        arrays: &HashMap<Identifier, Array>,
        solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        self.args_extractor
            .extract_int_array_element(constraint, arrays, solution)
    }

    /// Extracts an integer value from the constraint at the specified argument index.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `constraint` - The constraint call containing the argument.
    /// * `solution` - The solution map for resolving identifiers.
    ///
    /// # Returns
    /// The extracted integer value.
    pub fn extract_int_value(
        &self,
        index: usize,
        constraint: &Call,
        solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let literal = self.args_extractor.extract_term(constraint, index);
        self.args_extractor.extract_int_value(literal, solution)
    }

    /// Extracts integer coefficients from a linear expression in the constraint.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `constraint` - The constraint call containing the argument.
    /// * `arrays` - A map of identifiers to arrays for resolving array references.
    ///
    /// # Returns
    /// A vector of integer coefficients from the linear expression.
    pub fn extract_int_coefficients_lin_expr(
        &self,
        index: usize,
        constraint: &Call,
        arrays: &HashMap<Identifier, Array>,
    ) -> Vec<i64> {
        self.args_extractor
            .extract_int_coefficients_lin_expr(index, constraint, arrays)
    }
}

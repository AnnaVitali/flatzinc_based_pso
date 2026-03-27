use crate::args_extractor::args_extractor::ArgsExtractor;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Argument, Array, Constraint, Literal};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
/// A helper struct for extracting boolean arguments from constraints, 
/// utilizing an internal `ArgsExtractor` for common extraction logic.
pub struct BoolArgsExtractor {
    args_extractor: ArgsExtractor,
}

/// This struct provides methods to extract boolean values, arrays of booleans, and related information from constraint arguments, 
/// handling both literals and identifiers that reference variables or arrays in the solution.
impl BoolArgsExtractor {
    /// Creates a new `BoolArgsExtractor` with an internal `ArgsExtractor`.
    pub fn new() -> Self {
        let args_extractor = ArgsExtractor::new();
        Self { args_extractor }
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
        constraint: &Constraint,
        solution: &HashMap<String, VariableValue>,
    ) -> bool {
        self.args_extractor.extract_bool_value(
            self.args_extractor.extract_term(constraint, index),
            solution,
        )
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
        constraint: &Constraint,
        solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        self.args_extractor.extract_int_value(
            self.args_extractor.extract_term(constraint, index),
            solution,
        )
    }

    /// Extracts a boolean value from an array element at the specified index in the constraint.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `constraint` - The constraint call containing the argument.
    /// * `arrays` - A map of identifiers to arrays for resolving array references.
    /// * `solution` - The solution map for resolving identifiers.
    ///
    /// # Returns
    /// The extracted boolean value from the array element.
    pub fn extract_bool_element_array(
        &self,
        index: usize,
        constraint: &Constraint,
        arrays: &HashMap<String, Array>,
        solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let idx = self.args_extractor.extract_int_value(
            self.args_extractor.extract_term(constraint, index),
            solution,
        );

        let idx_usize = (idx - 1) as usize;

        let array = self.args_extractor.extract_array(constraint, arrays);

        match array.contents.get(idx_usize) {
            Some(Literal::Bool(v)) => *v,
            Some(Literal::Identifier(id)) => match solution.get(id) {
                Some(VariableValue::Bool(v)) => *v,
                Some(other) => panic!("Expected bool for variable `{}`, found {:?}", id, other),
                None => panic!("Missing value for variable `{}` referenced in array", id),
            },
            Some(other) => panic!(
                "Expected bool or identifier in array at index {}, found {:?}",
                idx, other
            ),
            None => panic!("No value present in array at index {} (out of bounds)", idx),
        }
    }

    /// Extracts a vector of boolean values from a defined elements array in the constraint.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `arrays` - A map of identifiers to arrays for resolving array references.
    /// * `constraint` - The constraint call containing the argument.
    /// * `solution` - The solution map for resolving identifiers.
    ///
    /// # Returns
    /// A vector of boolean values extracted from the defined elements array.
    pub fn extract_bool_defined_elements_array(
        &self,
        index: usize,
        arrays: &HashMap<String, Array>,
        constraint: &Constraint,
        solution: &HashMap<String, VariableValue>,
    ) -> Vec<bool> {
        let elements = constraint
            .args
            .get(index)
            .map(|arg| match arg {
                Argument::Array(l) => l
                    .iter()
                    .take(l.len().saturating_sub(1))
                    .map(|lit| match *lit {
                        Literal::Bool(v) => v,
                        Literal::Identifier(ref id) => match solution.get(id) {
                            Some(VariableValue::Bool(v)) => *v,
                            Some(other) => {
                                panic!("Expected bool for variable `{}`, found {:?}", id, other)
                            }
                            None => panic!(
                                "Missing value for variable `{}` referenced in array (constraint=`{}`, arg_index={})",
                                id,
                                constraint.id,
                                index
                            ),
                        },
                        _ => panic!("Expected bool or identifier in array"),
                    })
                    .collect(),
                Argument::Literal(lit) => match lit {
                    Literal::Bool(v) => vec![*v],
                    Literal::Identifier(name) => {
                        if let Some(arr) = arrays.get(name) {
                            arr.contents
                                .iter()
                                .take(arr.contents.len().saturating_sub(1))
                                .map(|lit| match *lit {
                                    Literal::Bool(v) => v,
                                    Literal::Identifier(ref id) => match solution.get(id) {
                                        Some(VariableValue::Bool(v)) => *v,
                                        Some(other) => {
                                            panic!("Expected bool for variable `{}`, found {:?}", id, other)
                                        }
                                        None => {
                                            panic!(
                                                "Missing value for variable `{}` referenced in array `{}` (constraint=`{}`, arg_index={})",
                                                id,
                                                name,
                                                constraint.id,
                                                index
                                            )
                                        }
                                    },
                                    _ => panic!(
                                        "Expected bool or identifier in array `{}`",
                                        name
                                    ),
                                })
                                .collect()
                        } else if let Some(VariableValue::Bool(v)) = solution.get(name) {
                            vec![*v]
                        } else {
                            panic!("No array or bool variable named `{}` found", name)
                        }
                    }
                    _ => Vec::new(),
                }
            })
            .unwrap_or_default();

        elements
    }

    /// Extracts a vector of boolean values from an array argument in the constraint.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `arrays` - A map of identifiers to arrays for resolving array references.
    /// * `constraint` - The constraint call containing the argument.
    /// * `solution` - The solution map for resolving identifiers.
    ///
    /// # Returns
    /// A vector of boolean values extracted from the array argument.
    pub fn extract_bool_array(
        &self,
        index: usize,
        arrays: &HashMap<String, Array>,
        constraint: &Constraint,
        solution: &HashMap<String, VariableValue>,
    ) -> Vec<bool> {
        let elements = constraint
            .args
            .get(index)
            .map(|arg| match arg {
                Argument::Array(l) => l
                    .iter()
                    .map(|lit| match *lit {
                        Literal::Bool(v) => v,
                        Literal::Identifier(ref id) => match solution.get(id) {
                            Some(VariableValue::Bool(v)) => *v,
                            Some(other) => {
                                panic!("Expected bool for variable `{}`, found {:?}", id, other)
                            }
                            None => panic!("Missing value for variable `{}` referenced in array", id),
                        },
                        _ => panic!("Expected bool or identifier in array"),
                    })
                    .collect(),
                Argument::Literal(lit) => match lit {
                    Literal::Bool(v) => vec![*v],
                    Literal::Identifier(name) => {
                        if let Some(arr) = arrays.get(name) {
                            arr.contents
                                .iter()
                                .map(|lit| match *lit {
                                    Literal::Bool(v) => v,
                                    Literal::Identifier(ref id) => match solution.get(id) {
                                        Some(VariableValue::Bool(v)) => *v,
                                        Some(other) => {
                                            panic!("Expected bool for variable `{}`, found {:?}", id, other)
                                        }
                                        None => {
                                            panic!("Missing value for variable `{}` referenced in array", id)
                                        }
                                    },
                                    _ => panic!(
                                        "Expected bool or identifier in array `{}`",
                                        name
                                    ),
                                })
                                .collect()
                        } else if let Some(VariableValue::Bool(v)) = solution.get(name) {
                            vec![*v]
                        } else {
                            panic!("No array or bool variable named `{}` found", name)
                        }
                    }
                    _ => Vec::new(),
                }
            })
            .unwrap_or_default();

        elements
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
        constraint: &Constraint,
        arrays: &HashMap<String, Array>,
    ) -> Vec<i64> {
        self.args_extractor
            .extract_int_coefficients_lin_expr(index, constraint, arrays)
    }

    /// Extracts the integer constant term from a linear expression in the constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call containing the linear expression.
    ///
    /// # Returns
    /// The integer constant term from the linear expression.
    pub fn extract_int_constant_term_lin_expr(&self, constraint: &Constraint) -> i64 {
        self.args_extractor
            .extract_int_constant_term_lin_expr(constraint)
    }
}

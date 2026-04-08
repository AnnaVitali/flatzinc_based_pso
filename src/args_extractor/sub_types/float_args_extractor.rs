use crate::args_extractor::args_extractor::ArgsExtractor;
use flatzinc_serde::{Argument, Array, Constraint, Literal};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
/// A helper struct for extracting float arguments from constraints,
/// utilizing an internal `ArgsExtractor` for common extraction logic.
pub struct FloatArgsExtractor {
    /// An internal `ArgsExtractor` used for extracting various types of arguments from constraints.
    args_extractor: ArgsExtractor,
}

/// This struct provides methods to extract float values, arrays of floats, and related information from constraint arguments,
/// handling both literals and identifiers that reference variables or arrays in the solution.
impl FloatArgsExtractor {
    /// Creates a new `FloatArgsExtractor` with an internal `ArgsExtractor`.
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

    /// Extracts a float value from the constraint at the specified argument index.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `constraint` - The constraint call containing the argument.
    ///
    /// # Returns
    /// The extracted float value.
    pub fn extract_float_value(&self, index: i64, constraint: &Constraint) -> f64 {
        let literal = self.args_extractor.extract_term(constraint, index as usize);

        match literal {
            Literal::Float(v) => v,
            _ => panic!("Literal {:?} is not float neither identifier", literal),
        }
    }

    /// Extracts a boolean value from the constraint at the specified argument index.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `constraint` - The constraint call containing the argument.
    ///
    /// # Returns
    /// The extracted boolean value.
    pub fn extract_bool_value(&self, index: i64, constraint: &Constraint) -> bool {
        self.args_extractor
            .extract_bool_value(self.args_extractor.extract_term(constraint, index as usize))
    }

    /// Extracts an integer value from the constraint at the specified argument index.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `constraint` - The constraint call containing the argument.
    ///
    /// # Returns
    /// The extracted integer value.
    pub fn extract_int_value(&self, index: i64, constraint: &Constraint) -> i64 {
        self.args_extractor
            .extract_int_value(self.args_extractor.extract_term(constraint, index as usize))
    }

    /// Extracts float coefficients from a linear expression in the constraint.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `constraint` - The constraint call containing the argument.
    /// * `arrays` - A map of identifiers to arrays for resolving array references.
    ///
    /// # Returns
    /// A vector of float coefficients from the linear expression.
    pub fn extract_float_coefficients_lin_expr(
        &self,
        index: i64,
        constraint: &Constraint,
        arrays: &HashMap<String, Array>,
    ) -> Vec<f64> {
        let coeff: Vec<f64> = constraint
            .args
            .get(index as usize)
            .map(|arg| match arg {
                Argument::Array(l) => l
                    .iter()
                    .filter_map(|lit| match *lit {
                        Literal::Float(v) => Some(v),
                        _ => None,
                    })
                    .collect(),
                Argument::Literal(lit) => match lit {
                    Literal::Float(v) => vec![*v],
                    Literal::Identifier(name) => arrays
                        .get(name)
                        .map(|arr| {
                            arr.contents
                                .iter()
                                .filter_map(|lit| match *lit {
                                    Literal::Float(v) => Some(v),
                                    _ => None,
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                    _ => Vec::new(),
                },
            })
            .unwrap_or_default();
        coeff
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
        index: i64,
        constraint: &Constraint,
        arrays: &HashMap<String, Array>,
    ) -> Vec<String> {
        self.args_extractor
            .extract_var_values_lin_expr(index as usize, constraint, arrays)
    }
}

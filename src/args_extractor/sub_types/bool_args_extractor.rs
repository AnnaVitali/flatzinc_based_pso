use crate::args_extractor::args_extractor::ArgsExtractor;
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

    pub fn extract_bool_array(
        &self,
        index: usize,
        args: &[Argument],
        arrays: &HashMap<String, Array>,
    ) -> Vec<String> {
        if let Some(arg) = args.get(index) {
            match arg {
                Argument::Literal(Literal::Identifier(array_key)) => {
                    if let Some(array) = arrays.get(array_key) {
                        array
                            .contents
                            .iter()
                            .map(|elem| match elem {
                                Literal::Identifier(i) => i.clone(),
                                _ => panic!(
                                    "Expected identifier in array for bool_clause constraint"
                                ),
                            })
                            .collect()
                    } else {
                        vec![array_key.clone()]
                    }
                }
                Argument::Array(literals) => literals
                    .iter()
                    .map(|elem| match elem {
                        Literal::Identifier(i) => i.clone(),
                        _ => panic!("Expected identifier in array for bool_clause constraint"),
                    })
                    .collect(),
                _ => vec![],
            }
        } else {
            vec![]
        }
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
    pub fn extract_bool_value(&self, index: i64, constraint: &Constraint) -> bool {
        self.args_extractor
            .extract_bool_value(self.args_extractor.extract_term(constraint, index as usize))
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
    pub fn extract_int_value(&self, index: i64, constraint: &Constraint) -> i64 {
        self.args_extractor
            .extract_int_value(self.args_extractor.extract_term(constraint, index as usize))
    }
}

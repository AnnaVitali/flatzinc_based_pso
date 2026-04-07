use crate::args_extractor::args_extractor::ArgsExtractor;
use flatzinc_serde::{Argument, Constraint, Literal};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
/// A helper struct for extracting set arguments from constraints,
/// utilizing an internal `ArgsExtractor` for common extraction logic.
pub struct SetArgsExtractor {
    args_extractor: ArgsExtractor,
}

/// This struct provides methods to extract set values, arrays of sets, and related information from constraint arguments,
/// handling both literals and identifiers that reference variables or arrays in the solution.
impl SetArgsExtractor {
    /// Creates a new `SetArgsExtractor` with an internal `ArgsExtractor`.
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

    /// Extracts a set of integers from the constraint at the specified argument index.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `constraint` - The constraint call containing the argument.
    /// * `solution` - The solution map for resolving identifiers.
    ///
    /// # Returns
    /// A `HashSet<i64>` representing the extracted set.
    pub fn extract_set_value(&self, index: i64, constraint: &Constraint) -> HashSet<i64> {
        let literal = self.args_extractor.extract_term(constraint, index as usize);

        match literal {
            Literal::IntSet(v) => {
                let mut set: HashSet<i64> = HashSet::new();
                for r in v.into_iter() {
                    let start = *r.start();
                    let end = *r.end();
                    for v in start..=end {
                        set.insert(v);
                    }
                }
                set
            }
            _ => panic!("Literal {:?} is not a set neither identifier", literal),
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

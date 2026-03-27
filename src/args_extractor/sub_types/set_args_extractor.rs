use crate::args_extractor::args_extractor::ArgsExtractor;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Argument, Array, Constraint, Literal};
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
    pub fn extract_set_value(
        &self,
        index: usize,
        constraint: &Constraint,
        solution: &HashMap<String, VariableValue>,
    ) -> HashSet<i64> {
        let literal = self.args_extractor.extract_term(constraint, index);

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
            Literal::Identifier(i) => match solution.get(&i) {
                Some(VariableValue::Set(v)) => v.clone(),
                Some(_) => panic!("Variable {} is not a set", i),
                None => panic!("Variable {} not found in solution", i),
            },
            _ => panic!("Literal {:?} is not a set neither identifier", literal),
        }
    }

    /// Extracts an integer element from a set argument in the constraint.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `constraint` - The constraint call containing the argument.
    /// * `solution` - The solution map for resolving identifiers.
    ///
    /// # Returns
    /// The extracted integer element from the set.
    pub fn extract_set_element(
        &self,
        index: usize,
        constraint: &Constraint,
        solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let elem = self.args_extractor.extract_int_value(
            self.args_extractor.extract_term(constraint, index),
            solution,
        );

        elem
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

    /// Extracts a set of integers from an array element in the constraint.
    ///
    /// # Arguments
    /// * `index` - The index of the argument in the constraint.
    /// * `constraint` - The constraint call containing the argument.
    /// * `arrays` - A map of identifiers to arrays for resolving array references.
    /// * `solution` - The solution map for resolving identifiers.
    ///
    /// # Returns
    /// A `HashSet<i64>` representing the extracted set from the array element.
    pub fn extract_set_array_element(
        &self,
        index: usize,
        constraint: &Constraint,
        arrays: &HashMap<String, Array>,
        solution: &HashMap<String, VariableValue>,
    ) -> HashSet<i64> {
        let idx = self.args_extractor.extract_int_value(
            self.args_extractor.extract_term(constraint, index),
            solution,
        );
        let idx_usize = (idx - 1) as usize;
        let array = self.args_extractor.extract_array(constraint, arrays);

        match array.contents.get(idx_usize) {
            Some(Literal::IntSet(range_list)) => {
                let mut set: HashSet<i64> = HashSet::new();
                for r in range_list.into_iter() {
                    let start = **r.start();
                    let end = **r.end();
                    for v in start..=end {
                        set.insert(v);
                    }
                }
                set
            }
            Some(Literal::Identifier(id)) => match solution.get(id) {
                Some(VariableValue::Set(set_val)) => set_val.clone(),
                Some(other) => panic!("Expected set for variable `{}`, found {:?}", id, other),
                None => panic!("Missing value for variable `{}` referenced in array", id),
            },
            Some(other) => panic!(
                "Expected IntSet or identifier at index {}, found {:?}",
                idx, other
            ),
            None => panic!("No value present in array at index {} (out of bounds)", idx),
        }
    }
}

use crate::solution_provider::VariableValue;
use flatzinc_serde::{Argument, Array, Call, Identifier, Literal};
use std::collections::HashMap;

pub const ARRAY_IDX_INDEX: usize = 0;
pub const TERM_INDEX: usize = 2;

#[derive(Debug, Clone, Default)]
/// A helper struct for extracting arguments from constraints, 
/// providing common methods for handling literals, identifiers, and arrays in the context of constraint evaluation.
pub struct ArgsExtractor {}

/// This struct provides methods to extract various types of arguments from constraints,
/// including integer values, boolean values, variable identifiers, and array elements,
/// handling both literals and identifiers that reference variables or arrays in the solution.
impl ArgsExtractor {
    /// Creates a new `ArgsExtractor`.
    pub fn new() -> Self {
        Self {}
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
        let mut ids = HashMap::with_capacity(args.len());
        let mut index: i64 = 0;

        for arg in args {
            match arg {
                Argument::Literal(lit) => match lit {
                    Literal::Identifier(name) => {
                        ids.insert(index, name.clone());
                    }
                    Literal::String(s) => {
                        ids.insert(index, s.clone());
                    }
                    _ => {}
                },
                Argument::Array(elems) => {
                    for lit in elems {
                        match lit {
                            Literal::Identifier(name) => {
                                ids.insert(index, name.clone());
                            }
                            Literal::String(s) => {
                                ids.insert(index, s.clone());
                            }
                            _ => {}
                        }
                    }
                }
            }
            index += 1;
        }

        ids
    }

    /// Extracts all literal identifiers from the given arguments.
    ///
    /// # Arguments
    /// * `args` - A slice of `Argument` to extract identifiers from.
    ///
    /// # Returns
    /// A vector of identifier names as `String`.
    pub fn extract_literal_identifiers(&self, args: &[Argument]) -> Vec<String> {
        let mut ids = Vec::with_capacity(args.len());

        for arg in args {
            match arg {
                Argument::Literal(lit) => match lit {
                    Literal::Identifier(name) => {
                        ids.push(name.clone());
                    }
                    Literal::String(s) => {
                        ids.push(s.clone());
                    }
                    _ => {}
                },
                Argument::Array(elems) => {
                    for lit in elems {
                        match lit {
                            Literal::Identifier(name) => {
                                ids.push(name.clone());
                            }
                            Literal::String(s) => {
                                ids.push(s.clone());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        ids
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
        let vars_involved: Vec<Identifier> = constraint
            .args
            .get(index)
            .map(|arg| match arg {
                Argument::Array(l) => l
                    .iter()
                    .filter_map(|lit| match *lit {
                        Literal::Identifier(ref id) => Some(id.clone()),
                        _ => None,
                    })
                    .collect(),
                Argument::Literal(literal) => match literal {
                    Literal::Identifier(id) => {
                        if arrays.contains_key(id) {
                            arrays
                                .get(id)
                                .map(|arr| {
                                    arr.contents
                                        .iter()
                                        .filter_map(|lit| match *lit {
                                            Literal::Identifier(ref id) => Some(id.clone()),
                                            _ => None,
                                        })
                                        .collect()
                                })
                                .unwrap_or_default()
                        } else {
                            vec![id.clone()]
                        }
                    }
                    _ => Vec::new(),
                },
            })
            .unwrap_or_default();
        vars_involved
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
    pub fn extract_int_array_element(
        &self,
        constraint: &Call,
        arrays: &HashMap<Identifier, Array>,
        solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let idx = self.extract_int_value(self.extract_term(constraint, ARRAY_IDX_INDEX), solution);
        let idx_usize = (idx - 1) as usize;

        let array = self.extract_array(constraint, arrays);

        match array.contents.get(idx_usize) {
            Some(Literal::Int(v)) => *v,
            Some(Literal::Identifier(id)) => match solution.get(id) {
                Some(VariableValue::Int(v)) => *v,
                Some(other) => panic!("Expected int for variable `{}`, found {:?}", id, other),
                None => panic!("Missing value for variable `{}` referenced in array", id),
            },
            Some(other) => panic!(
                "Expected int or identifier in array at index {}, found {:?}",
                idx, other
            ),
            None => panic!("No value present in array at index {} (out of bounds)", idx),
        }
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
        let coeff: Vec<i64> = constraint
            .args
            .get(index)
            .map(|arg| match arg {
                Argument::Array(l) => l
                    .iter()
                    .filter_map(|lit| match *lit {
                        Literal::Int(v) => Some(v),
                        _ => None,
                    })
                    .collect(),
                Argument::Literal(lit) => match lit {
                    Literal::Int(v) => vec![*v],
                    Literal::Identifier(name) => arrays
                        .get(name)
                        .map(|arr| {
                            arr.contents
                                .iter()
                                .filter_map(|lit| match *lit {
                                    Literal::Int(v) => Some(v),
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

    /// Extracts the integer constant term from a linear expression in the constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call containing the linear expression.
    ///
    /// # Returns
    /// The integer constant term from the linear expression.
    pub fn extract_int_constant_term_lin_expr(&self, constraint: &Call) -> i64 {
        constraint
            .args
            .get(TERM_INDEX)
            .and_then(|arg| match arg {
                Argument::Literal(lit) => match lit {
                    Literal::Int(v) => Some(*v),
                    _ => None,
                },
                _ => None,
            })
            .unwrap_or_default()
    }

    /// Extracts an integer value from a literal or identifier.
    ///
    /// # Arguments
    /// * `literal` - The literal or identifier to extract the value from.
    /// * `solution` - The solution map for resolving identifiers.
    ///
    /// # Returns
    /// The extracted integer value.
    pub fn extract_int_value(
        &self,
        literal: Literal,
        solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        match literal {
            Literal::Int(v) => v,
            Literal::Identifier(i) => match solution.get(&i) {
                Some(VariableValue::Int(v)) => *v,
                Some(_) => panic!("Variable {} is not int", i),
                None => panic!("Variable {} not found in solution", i),
            },
            _ => panic!("Literal {:?} is not int neither identifier", literal),
        }
    }

    /// Extracts a boolean value from a literal or identifier.
    ///
    /// # Arguments
    /// * `literal` - The literal or identifier to extract the value from.
    /// * `solution` - The solution map for resolving identifiers.
    ///
    /// # Returns
    /// The extracted boolean value.
    pub fn extract_bool_value(
        &self,
        literal: Literal,
        solution: &HashMap<String, VariableValue>,
    ) -> bool {
        match literal {
            Literal::Bool(v) => v,
            Literal::Identifier(i) => match solution.get(&i) {
                Some(VariableValue::Bool(v)) => *v,
                Some(_) => panic!("Variable {} is not bool", i),
                None => panic!("Variable {} not found in solution", i),
            },
            _ => panic!("Literal {:?} is not bool neither identifier", literal),
        }
    }

    /// Extracts a literal term from the constraint at the specified argument index.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call containing the argument.
    /// * `index` - The index of the argument in the constraint.
    ///
    /// # Returns
    /// The extracted `Literal` term.
    pub fn extract_term(&self, constraint: &Call, index: usize) -> Literal {
        let arg = constraint
            .args
            .get(index)
            .unwrap_or_else(|| panic!("argument at index {} is missing", index));

        match arg {
            Argument::Literal(lit) => lit.clone(),
            _ => panic!("no elements found in args for constraint: {:?}", constraint),
        }
    }

    /// Extracts an array from the constraint using the identifier at argument index 1.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call containing the argument.
    /// * `arrays` - A map of identifiers to arrays for resolving array references.
    ///
    /// # Returns
    /// A reference to the extracted `Array`.
    pub fn extract_array<'a>(
        &self,
        constraint: &Call,
        arrays: &'a HashMap<Identifier, Array>,
    ) -> &'a Array {
        match self.extract_term(constraint, 1) {
            Literal::Identifier(id) => arrays.get(&id).unwrap_or_else(|| {
                panic!(
                    "Array {:?} not found for identifier used in constraint {:?}",
                    id, constraint
                )
            }),
            other => panic!(
                "Expected Identifier for array at args[1], found {:?} in constraint {:?}",
                other, constraint
            ),
        }
    }
}

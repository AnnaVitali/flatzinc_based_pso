use flatzinc_serde::{Argument, Array, Constraint, Literal};
use std::collections::HashMap;

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
        constraint: &Constraint,
        arrays: &HashMap<String, Array>,
    ) -> Vec<String> {
        let vars_involved: Vec<String> = constraint
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

    /// Extracts an integer value from a literal or identifier.
    ///
    /// # Arguments
    /// * `literal` - The literal or identifier to extract the value from.
    /// * `solution` - The solution map for resolving identifiers.
    ///
    /// # Returns
    /// The extracted integer value.
    pub fn extract_int_value(&self, literal: Literal) -> i64 {
        match literal {
            Literal::Int(v) => v,
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
    pub fn extract_bool_value(&self, literal: Literal) -> bool {
        match literal {
            Literal::Bool(v) => v,
            _ => panic!("Literal {:?} is not bool", literal),
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
    pub fn extract_term(&self, constraint: &Constraint, index: usize) -> Literal {
        let arg = constraint
            .args
            .get(index)
            .unwrap_or_else(|| panic!("argument at index {} is missing", index));

        match arg {
            Argument::Literal(lit) => lit.clone(),
            _ => panic!("no elements found in args for constraint: {:?}", constraint),
        }
    }
}

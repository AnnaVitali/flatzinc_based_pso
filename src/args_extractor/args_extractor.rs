use crate::solution_provider::VariableValue;
use flatzinc_serde::{Argument, Array, Call, Identifier, Literal};
use std::collections::HashMap;

pub const ARRAY_IDX_INDEX: usize = 0;
pub const TERM_INDEX: usize = 2;

#[derive(Debug, Clone, Default)]
pub struct ArgsExtractor {}

impl ArgsExtractor {
    pub fn new() -> Self {
        Self {}
    }

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
                    //Literal::String(s) => vec![s.clone()],
                    _ => Vec::new(),
                },
            })
            .unwrap_or_default();
        vars_involved
    }

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

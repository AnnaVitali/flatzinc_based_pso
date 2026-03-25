use crate::args_extractor::args_extractor::ArgsExtractor;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Argument, Array, Call, Identifier, Literal};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct BoolArgsExtractor {
    args_extractor: ArgsExtractor,
}

impl BoolArgsExtractor {
    pub fn new() -> Self {
        let args_extractor = ArgsExtractor::new();
        Self { args_extractor }
    }

    pub fn extract_literal_identifiers_with_index(&self, args: &[Argument]) -> HashMap<i64, String> {
        self.args_extractor
            .extract_literal_identifiers_with_index(args)
    }

    pub fn extract_literal_identifiers(&self, args: &[Argument]) -> Vec<String> {
        self.args_extractor.extract_literal_identifiers(args)
    }

    pub fn extract_bool_value(
        &self,
        index: usize,
        constraint: &Call,
        solution: &HashMap<String, VariableValue>,
    ) -> bool {
        self.args_extractor.extract_bool_value(
            self.args_extractor.extract_term(constraint, index),
            solution,
        )
    }

    pub fn extract_int_value(
        &self,
        index: usize,
        constraint: &Call,
        solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        self.args_extractor.extract_int_value(
            self.args_extractor.extract_term(constraint, index),
            solution,
        )
    }

    pub fn extract_bool_element_array(
        &self,
        index: usize,
        constraint: &Call,
        arrays: &HashMap<Identifier, Array>,
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

    pub fn extract_bool_defined_elements_array(
        &self,
        index: usize,
        arrays: &HashMap<Identifier, Array>,
        constraint: &Call,
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

    pub fn extract_bool_array(
        &self,
        index: usize,
        arrays: &HashMap<Identifier, Array>,
        constraint: &Call,
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

    pub fn extract_int_coefficients_lin_expr(
        &self,
        index: usize,
        constraint: &Call,
        arrays: &HashMap<Identifier, Array>,
    ) -> Vec<i64> {
        self.args_extractor
            .extract_int_coefficients_lin_expr(index, constraint, arrays)
    }

    pub fn extract_int_constant_term_lin_expr(&self, constraint: &Call) -> i64 {
        self.args_extractor
            .extract_int_constant_term_lin_expr(constraint)
    }
}

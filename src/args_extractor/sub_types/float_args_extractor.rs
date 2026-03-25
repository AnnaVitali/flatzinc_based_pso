use crate::args_extractor::args_extractor::ArgsExtractor;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Argument, Array, Call, Identifier, Literal};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct FloatArgsExtractor {
    args_extractor: ArgsExtractor,
}

impl FloatArgsExtractor {
    pub fn new() -> Self {
        let args_extractor = ArgsExtractor::new();
        Self { args_extractor }
    }

    pub fn extract_literal_identifier_with_index(&self, args: &[Argument]) -> HashMap<i64, String> {
        self.args_extractor
            .extract_literal_identifiers_with_index(args)
    }

    pub fn extract_literal_identifiers(&self, args: &[Argument]) -> Vec<String> {
        self.args_extractor.extract_literal_identifiers(args)
    }

    pub fn extract_float_element_in_array(
        &self,
        constraint: &Call,
        arrays: &HashMap<Identifier, Array>,
        solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let idx = self
            .args_extractor
            .extract_int_value(self.args_extractor.extract_term(constraint, 0), solution);
        let idx_usize = (idx - 1) as usize;

        let array = self.args_extractor.extract_array(constraint, arrays);

        match array.contents.get(idx_usize) {
            Some(Literal::Float(v)) => *v,
            Some(Literal::Identifier(id)) => match solution.get(id) {
                Some(VariableValue::Float(v)) => *v,
                Some(other) => panic!("Expected float for variable `{}`, found {:?}", id, other),
                None => panic!("Missing value for variable `{}` referenced in array", id),
            },
            Some(other) => panic!(
                "Expected float or identifier in array at index {}, found {:?}",
                idx, other
            ),
            None => panic!("No value present in array at index {} (out of bounds)", idx),
        }
    }

    pub fn extract_float_value(
        &self,
        index: usize,
        constraint: &Call,
        solution: &HashMap<String, VariableValue>,
    ) -> f64 {
        let literal = self.args_extractor.extract_term(constraint, index);

        match literal {
            Literal::Float(v) => v,
            Literal::Identifier(i) => match solution.get(&i) {
                Some(VariableValue::Float(v)) => *v,
                Some(_) => panic!("Variable {} is not float", i),
                None => panic!("Variable {} not found in solution", i),
            },
            _ => panic!("Literal {:?} is not float neither identifier", literal),
        }
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

    pub fn extract_float_coefficients_lin_expr(
        &self,
        index: usize,
        constraint: &Call,
        arrays: &HashMap<Identifier, Array>,
    ) -> Vec<f64> {
        let coeff: Vec<f64> = constraint
            .args
            .get(index)
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

    pub fn extract_var_values_lin_expr(
        &self,
        index: usize,
        constraint: &Call,
        arrays: &HashMap<Identifier, Array>,
    ) -> Vec<Identifier> {
        self.args_extractor
            .extract_var_values_lin_expr(index, constraint, arrays)
    }
}

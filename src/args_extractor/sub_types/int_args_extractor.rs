use crate::args_extractor::args_extractor::ArgsExtractor;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Argument, Array, Call, Identifier};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct IntArgsExtractor {
    args_extractor: ArgsExtractor,
}

impl IntArgsExtractor {
    pub fn new() -> Self {
        Self {
            args_extractor: ArgsExtractor::new(),
        }
    }

    pub fn extract_literal_identifiers_with_index(&self, args: &[Argument]) -> HashMap<i64, String> {
        self.args_extractor.extract_literal_identifiers_with_index(args)
    }

    pub fn extract_literal_identifiers(&self, args: &[Argument]) -> Vec<String> {
        self.args_extractor.extract_literal_identifiers(args)
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

    pub fn extract_bool_value(
        &self,
        index: usize,
        constraint: &Call,
        solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let literal = self.args_extractor.extract_term(constraint, index);
        self.args_extractor.extract_bool_value(literal, solution)
    }

    pub fn extract_int_element_array(
        &self,
        constraint: &Call,
        arrays: &HashMap<Identifier, Array>,
        solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        self.args_extractor
            .extract_int_array_element(constraint, arrays, solution)
    }

    pub fn extract_int_value(
        &self,
        index: usize,
        constraint: &Call,
        solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let literal = self.args_extractor.extract_term(constraint, index);
        self.args_extractor.extract_int_value(literal, solution)
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
}

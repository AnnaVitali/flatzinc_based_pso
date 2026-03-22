use crate::args_extractor::args_extractor::ArgsExtractor;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Call, Identifier, Literal};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct SetArgsExtractor {
    args_extractor: ArgsExtractor,
}

impl SetArgsExtractor {
    pub fn new() -> Self {
        let args_extractor = ArgsExtractor::new();
        Self { args_extractor }
    }

    pub fn extract_set_value(
        &self,
        index: usize,
        constraint: &Call,
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

    pub fn extract_set_element(
        &self,
        index: usize,
        constraint: &Call,
        solution: &HashMap<String, VariableValue>,
    ) -> i64 {
        let elem = self.args_extractor.extract_int_value(
            self.args_extractor.extract_term(constraint, index),
            solution,
        );

        elem
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

    pub fn extract_set_array_element(
        &self,
        index: usize,
        constraint: &Call,
        arrays: &HashMap<Identifier, Array>,
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

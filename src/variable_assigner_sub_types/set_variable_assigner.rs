
use crate::{invariant_evaluator_sub_types::set_invariant_evaluator::{X_TERM_INDEX, ARRAY_INDEX, Y_TERM_INDEX}, solution_provider::VariableValue};
use flatzinc_serde::{ Array, Call, Identifier};
use std::collections::{HashMap, HashSet};
use crate::args_extractor_sub_types::set_args_extractor::SetArgsExtractor;

#[derive(Debug, Clone, Default)]
pub struct SetVariableAssigner {
    args_extractor: SetArgsExtractor,
    arrays: HashMap<Identifier, Array>,
}

impl SetVariableAssigner {
    pub fn new(arrays: HashMap<Identifier, Array>) -> Self {
        let args_extractor = SetArgsExtractor::new();

        Self {
            args_extractor,
            arrays,
        }
    }

    pub fn array_set_element(
        &self,
        constraint: &Call,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> HashSet<i64> {
        self.args_extractor
            .extract_set_array_element(ARRAY_INDEX, constraint, &self.arrays, complete_solution)
    }

    pub fn set_diff( &self,
                     constraint: &Call,
                     complete_solution: &HashMap<String, VariableValue>) -> HashSet<i64>{
        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, constraint, complete_solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, constraint, complete_solution);

        x.difference(&y).copied().collect()
    }

    pub fn set_eq( &self,
                   constraint: &Call,
                   complete_solution: &HashMap<String, VariableValue>) -> HashSet<i64>{
        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, constraint, complete_solution);

        x.clone()
    }

    pub fn set_eq_reif( &self,
                   constraint: &Call,
                   complete_solution: &HashMap<String, VariableValue>) -> bool{
        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, constraint, complete_solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, constraint, complete_solution);

        x == y
    }

    pub fn set_in_reif(&self,
                  constraint: &Call,
                  complete_solution: &HashMap<String, VariableValue>) -> bool {
        let x = self.args_extractor.extract_int_value(X_TERM_INDEX, constraint, complete_solution);
        let s = self.args_extractor.extract_set_value(Y_TERM_INDEX, constraint, complete_solution);
        s.contains(&x)
    }

    pub fn set_intersect( &self,
                     constraint: &Call,
                     complete_solution: &HashMap<String, VariableValue>) -> HashSet<i64>{
        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, constraint, complete_solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, constraint, complete_solution);

        x.intersection(&y).copied().collect()
    }

    pub fn set_le_reif( &self,
                        constraint: &Call,
                        complete_solution: &HashMap<String, VariableValue>) -> bool{
        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, constraint, complete_solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, constraint, complete_solution);
        let mut xv: Vec<i64> = x.iter().cloned().collect();
        let mut yv: Vec<i64> = y.iter().cloned().collect();

        xv.sort();
        yv.sort();


        xv <= yv
    }

    pub fn set_lt_reif( &self,
                        constraint: &Call,
                        complete_solution: &HashMap<String, VariableValue>) -> bool{
        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, constraint, complete_solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, constraint, complete_solution);
        let mut xv: Vec<i64> = x.iter().cloned().collect();
        let mut yv: Vec<i64> = y.iter().cloned().collect();

        xv.sort();
        yv.sort();

        xv < yv
    }

    pub fn set_ne_reif( &self,
                        constraint: &Call,
                        complete_solution: &HashMap<String, VariableValue>) -> bool{
        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, constraint, complete_solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, constraint, complete_solution);

        x != y
    }

    pub fn set_subset_reif( &self,
                     constraint: &Call,
                     complete_solution: &HashMap<String, VariableValue>) -> bool{
        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, constraint, complete_solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, constraint, complete_solution);

        x.is_subset(&y)
    }

    pub fn set_superset_reif( &self,
                            constraint: &Call,
                            complete_solution: &HashMap<String, VariableValue>) -> bool{
        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, constraint, complete_solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, constraint, complete_solution);

        x.is_superset(&y)
    }

    pub fn set_symmetric_difference( &self,
                          constraint: &Call,
                          complete_solution: &HashMap<String, VariableValue>) -> HashSet<i64>{
        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, constraint, complete_solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, constraint, complete_solution);

        x.symmetric_difference(&y).copied().collect()
    }

    pub fn set_union( &self,
                                     constraint: &Call,
                                     complete_solution: &HashMap<String, VariableValue>) -> HashSet<i64>{
        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, constraint, complete_solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, constraint, complete_solution);

        x.union(&y).copied().collect()
    }

}

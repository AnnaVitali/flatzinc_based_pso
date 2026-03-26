use crate::args_extractor::sub_types::set_args_extractor::SetArgsExtractor;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::evaluator::sub_types::set_functional_evaluator::{
    ARRAY_INDEX, X_TERM_INDEX, Y_TERM_INDEX,
};
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Call, Identifier};
use std::collections::{HashMap, HashSet};

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
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> HashSet<i64> + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            args_extractor.extract_set_array_element(ARRAY_INDEX, &call, &arrays, solution)
        })
    }

    pub fn set_diff(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> HashSet<i64> + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x.difference(&y).copied().collect()
        })
    }

    pub fn set_eq(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> HashSet<i64> + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            x.clone()
        })
    }

    pub fn set_eq_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x == y
        })
    }

    pub fn set_in_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_int_value(X_TERM_INDEX, &call, solution);
            let s = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            s.contains(&x)
        })
    }

    pub fn set_intersect(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> HashSet<i64> + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x.intersection(&y).copied().collect()
        })
    }

    pub fn set_le_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            let mut xv: Vec<i64> = x.iter().cloned().collect();
            let mut yv: Vec<i64> = y.iter().cloned().collect();
            xv.sort();
            yv.sort();
            xv <= yv
        })
    }

    pub fn set_lt_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            let mut xv: Vec<i64> = x.iter().cloned().collect();
            let mut yv: Vec<i64> = y.iter().cloned().collect();
            xv.sort();
            yv.sort();
            xv < yv
        })
    }

    pub fn set_ne_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x != y
        })
    }

    pub fn set_subset_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x.is_subset(&y)
        })
    }

    pub fn set_superset_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x.is_superset(&y)
        })
    }

    pub fn set_symmetric_difference(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> HashSet<i64> + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x.symmetric_difference(&y).copied().collect()
        })
    }

    pub fn set_union(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> HashSet<i64> + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x.union(&y).copied().collect()
        })
    }
}

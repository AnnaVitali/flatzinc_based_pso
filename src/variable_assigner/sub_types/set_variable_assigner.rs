use crate::args_extractor::sub_types::set_args_extractor::SetArgsExtractor;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::evaluator::sub_types::set_functional_evaluator::{
    ARRAY_INDEX, X_TERM_INDEX, Y_TERM_INDEX,
};
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Identifier};
use std::collections::{HashMap, HashSet};

/// Struct responsible for assigning set variables based on constraints and solutions.
///
/// # Fields
/// * `args_extractor` - Extracts arguments for set constraints.
/// * `arrays` - Stores arrays mapped by their identifiers.
#[derive(Debug, Clone, Default)]
pub struct SetVariableAssigner {
    /// An instance of `SetArgsExtractor` used to extract arguments from set constraints.
    args_extractor: SetArgsExtractor,
    /// A hashmap that maps identifiers to their corresponding arrays, used for resolving array references in constraints.
    arrays: HashMap<Identifier, Array>,
}

impl SetVariableAssigner {
    /// Creates a new `SetVariableAssigner` with the provided arrays.
    ///
    /// # Arguments
    /// * `arrays` - A map from identifiers to arrays used in set constraints.
    ///
    /// # Returns
    /// A new instance of `SetVariableAssigner`.
    pub fn new(arrays: HashMap<Identifier, Array>) -> Self {
        let args_extractor = SetArgsExtractor::new();

        Self {
            args_extractor,
            arrays,
        }
    }

    /// Returns a closure that evaluates the `array_set_element` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the set value from the array.
    pub fn array_set_element(
        &self,
        constraint: &CallWithDefines,
        
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> HashSet<i64> + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            args_extractor.extract_set_array_element(ARRAY_INDEX, &call, &arrays, solution)
        })
    }

    /// Returns a closure that evaluates the `set_diff` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the difference between two sets.
    pub fn set_diff(
        &self,
        constraint: &CallWithDefines,
        
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> HashSet<i64> + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x.difference(&y).copied().collect()
        })
    }

    /// Returns a closure that evaluates the `set_eq` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the set value for equality comparison.
    pub fn set_eq(
        &self,
        constraint: &CallWithDefines,
        
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> HashSet<i64> + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            x.clone()
        })
    }

    /// Returns a closure that evaluates the `set_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether two sets are equal.
    pub fn set_eq_reif(
        &self,
        constraint: &CallWithDefines,
        
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x == y
        })
    }

    /// Returns a closure that evaluates the `set_in_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether an integer is in a set.
    pub fn set_in_reif(
        &self,
        constraint: &CallWithDefines,
        
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_int_value(X_TERM_INDEX, &call, solution);
            let s = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            s.contains(&x)
        })
    }

    /// Returns a closure that evaluates the `set_intersect` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the intersection of two sets.
    pub fn set_intersect(
        &self,
        constraint: &CallWithDefines,
        
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> HashSet<i64> + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x.intersection(&y).copied().collect()
        })
    }

    /// Returns a closure that evaluates the `set_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one set is less than or equal to another (by sorted order).
    pub fn set_le_reif(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `set_lt_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one set is less than another (by sorted order).
    pub fn set_lt_reif(
        &self,
        constraint: &CallWithDefines,
        
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

    /// Returns a closure that evaluates the `set_ne_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether two sets are not equal.
    pub fn set_ne_reif(
        &self,
        constraint: &CallWithDefines,
        
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x != y
        })
    }

    /// Returns a closure that evaluates the `set_subset_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one set is a subset of another.
    pub fn set_subset_reif(
        &self,
        constraint: &CallWithDefines,
        
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x.is_subset(&y)
        })
    }

    /// Returns a closure that evaluates the `set_superset_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one set is a superset of another.
    pub fn set_superset_reif(
        &self,
        constraint: &CallWithDefines,
        
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x.is_superset(&y)
        })
    }

    /// Returns a closure that evaluates the `set_symmetric_difference` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the symmetric difference of two sets.
    pub fn set_symmetric_difference(
        &self,
        constraint: &CallWithDefines,
        
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> HashSet<i64> + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);
            x.symmetric_difference(&y).copied().collect()
        })
    }

    /// Returns a closure that evaluates the `set_union` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the union of two sets.
    pub fn set_union(
        &self,
        constraint: &CallWithDefines,
        
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

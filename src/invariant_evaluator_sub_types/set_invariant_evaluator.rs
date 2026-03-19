use crate::{args_extractor_sub_types::set_args_extractor::SetArgsExtractor, invariant_evaluator::CallWithDefines};
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Identifier};
use log::info;
use std::collections::{HashMap, HashSet};
use crate::data_utility::data_utility::ConstraintEvaluation;

pub const ARRAY_INDEX: usize = 0;
pub const X_TERM_INDEX: usize = 0;
pub const Y_TERM_INDEX: usize = 1;
pub const Z_TERM_INDEX: usize = 2;
pub const R_TERM_INDEX: usize = 2;

#[derive(Debug, Clone, Default)]
pub struct SetInvariantEvaluator {
    arrays: HashMap<Identifier, Array>,
    args_extractor: SetArgsExtractor,
    verbose: bool,
}

impl SetInvariantEvaluator {
    pub fn new(
        arrays: HashMap<Identifier, Array>,
        verbose: bool,
    ) -> Self {
        let args_extractor = SetArgsExtractor::new();

        Self {
            arrays,
            args_extractor,
            verbose,
        }
    }

    pub fn array_set_element(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let array_value =
            self.args_extractor
                .extract_set_array_element(ARRAY_INDEX, &constraint.call, &self.arrays, solution);
        let value = self
            .args_extractor
            .extract_set_value(Z_TERM_INDEX, &constraint.call, solution);

        if array_value != value {
            if self.verbose {
                info!("Violated: array value {:?} = {:?}", array_value, value);
            }
            violation = array_value.difference(&value).count() as i64
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id: constraint.id        }
    }

    pub fn set_card(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let set = self
            .args_extractor
            .extract_set_value(X_TERM_INDEX,&constraint.call, solution);
        let real_card = set.len();
        let value = self
            .args_extractor
            .extract_int_value(Y_TERM_INDEX, &constraint.call, solution);

        if real_card != value as usize {
            if self.verbose {
                info!("Violated: |{}| = {}", set.len(), value);
            }
            violation = (real_card as i64 - value).abs()
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_diff(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);
        let z = self.args_extractor.extract_set_value(Z_TERM_INDEX, &constraint.call, solution);
        let diff: HashSet<i64> = x.difference(&y).copied().collect();

        if diff != z {
            if self.verbose {
                info!("Violated: {:?} \\ {:?} = {:?}", x, y, z);
            }

            violation = diff.symmetric_difference(&z).count() as i64
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_eq(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);

        if x != y {
            if self.verbose {
                info!("Violated: {:?} = {:?}", x, y);
            }
            violation = x.symmetric_difference(&y).count() as i64
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_eq_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

            let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);
        let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution);


        if !(r == (x == y)) {
            if self.verbose {
                info!("Violated: {} <-> {:?} = {:?}", r, x, y);
            }
            violation = 1
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_in(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;
        let elem = self.args_extractor.extract_set_element(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);


        if !y.contains(&elem) {
            if self.verbose {
                info!("Violated: {} in {:?}", elem, y);
            }
            violation = 1
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_in_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;
        let elem = self.args_extractor.extract_set_element(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);
        let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution);

        if !(r == y.contains(&elem)) {
            if self.verbose {
                info!("Violated: {} <-> {} in {:?}", r, elem, y);
            }
            violation = 1
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_intersect(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);
        let z = self.args_extractor.extract_set_value(Z_TERM_INDEX, &constraint.call, solution);
        let intersect: HashSet<i64> = x.intersection(&y).copied().collect();

        if intersect != z {
            if self.verbose {
                info!("Violated: {:?} intersect {:?} = {:?}", x, y, z);
            }
            violation = intersect.difference(&z).count() as i64
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_le(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);

        let mut xv: Vec<i64> = x.iter().cloned().collect();
        let mut yv: Vec<i64> = y.iter().cloned().collect();

        xv.sort();
        yv.sort();

        if !(xv <= yv) {
            if self.verbose {
                info!("Violated: {:?} <= {:?}", x, y);
            }

            for (i, elem) in yv.iter().enumerate() {
                if *elem < xv[i]{
                    violation += 1;
                }
            }
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_le_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);
        let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution);

        let mut xv: Vec<i64> = x.iter().cloned().collect();
        let mut yv: Vec<i64> = y.iter().cloned().collect();

        xv.sort();
        yv.sort();

        if !(r == (xv <= yv)) {
            if self.verbose {
                info!("Violated: {} <-> {:?} <= {:?}", r, x, y);
            }
            violation = 1
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_lt(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);

        let mut xv: Vec<i64> = x.iter().cloned().collect();
        let mut yv: Vec<i64> = y.iter().cloned().collect();

        xv.sort();
        yv.sort();

        if !(xv < yv) {
            if self.verbose {
                info!("Violated: {:?} < {:?}", x, y);
            }

            for (i, elem) in yv.iter().enumerate() {
                if *elem <= xv[i]{
                    violation += 1;
                }
            }
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_lt_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);
        let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution);


        let mut xv: Vec<i64> = x.iter().cloned().collect();
        let mut yv: Vec<i64> = y.iter().cloned().collect();

        xv.sort();
        yv.sort();

        if !(r == (xv < yv)) {
            if self.verbose {
                info!("Violated: {} <-> {:?} < {:?}", r, x, y);
            }
            violation = 1
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_ne(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);

        if x == y {
            if self.verbose {
                info!("Violated: {:?} != {:?}", x, y);
            }
            violation = 1
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_ne_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);
        let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution);


        if !(r == (x != y)) {
            if self.verbose {
                info!("Violated: {} <-> {:?} != {:?}", r, x, y);
            }
            violation = 1
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_subset(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);

        if !x.is_subset(&y) {
            if self.verbose {
                info!("Violated: {:?} subset {:?}", x, y);
            }
            violation = 1
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_subset_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);
        let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution);


        if !(r == x.is_subset(&y)) {
            if self.verbose {
                info!("Violated: r <-> {:?} subset {:?}", x, y);
            }
            violation = 1
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_superset(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);

        if !x.is_superset(&y) {
            if self.verbose {
                info!("Violated: {:?} superset {:?}", x, y);
            }
            violation = 1
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_superset_reif(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);
        let r = self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution);


        if !(r == x.is_superset(&y)) {
            if self.verbose {
                info!("Violated: r <-> {:?} superset {:?}", x, y);
            }
            violation = 1
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_symdiff(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);
        let z = self.args_extractor.extract_set_value(Z_TERM_INDEX, &constraint.call, solution);
        let sym_diff: HashSet<i64> = x.symmetric_difference(&y).copied().collect();

        if sym_diff != z {
            if self.verbose {
                info!("Violated: {:?} sym diff {:?} = {:?}", x, y, z);
            }
            violation = sym_diff.difference(&z).count() as i64;
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }

    pub fn set_union(&self, constraint: &CallWithDefines, solution: &HashMap<String, VariableValue>) -> ConstraintEvaluation {
        let mut violation = 0;

        let x = self.args_extractor.extract_set_value(X_TERM_INDEX, &constraint.call, solution);
        let y = self.args_extractor.extract_set_value(Y_TERM_INDEX, &constraint.call, solution);
        let z = self.args_extractor.extract_set_value(Z_TERM_INDEX, &constraint.call, solution);
        let union: HashSet<i64> = x.union(&y).copied().collect();

        if union != z {
            if self.verbose {
                info!("Violated: {:?} U {:?} = {:?}", x, y, z);
            }
            violation = union.difference(&z).count() as i64
        }

                ConstraintEvaluation {
            violation: violation as f64,
            constraint_id:  constraint.id        }
    }
}

use crate::args_extractor::sub_types::set_args_extractor::SetArgsExtractor;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Identifier};
use log::info;
use std::collections::{HashMap, HashSet};

pub const ARRAY_INDEX: usize = 0;
pub const X_TERM_INDEX: usize = 0;
pub const Y_TERM_INDEX: usize = 1;
pub const Z_TERM_INDEX: usize = 2;
pub const R_TERM_INDEX: usize = 2;

#[derive(Debug, Clone, Default)]
/// Evaluator for set constraints, providing methods to evaluate various set operations and constraints.
///
/// This struct contains methods for evaluating set constraints such as equality, inequality, linear expressions,
/// and arithmetic operations. It uses argument extraction utilities and supports verbose output for debugging. 
pub struct SetEvaluator {
    arrays: HashMap<Identifier, Array>,
    args_extractor: SetArgsExtractor,
    verbose: bool,
}

impl SetEvaluator {
    /// Returns a new `SetFunctionalEvaluator` instance.
    ///
    /// # Arguments
    /// * `arrays` - The arrays used for evaluation.
    /// * `verbose` - Whether to enable verbose logging.
    /// # Returns
    /// A new instance of `SetFunctionalEvaluator`.
    pub fn new(arrays: HashMap<Identifier, Array>, verbose: bool) -> Self {
        let args_extractor = SetArgsExtractor::new();

        Self {
            arrays,
            args_extractor,
            verbose,
        }
    }

    /// Returns a functional evaluator for the `array_set_element` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the set difference count if violated, 0.0 otherwise.
    pub fn array_set_element(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let verbose = self.verbose;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let array_value =
                args_extractor.extract_set_array_element(ARRAY_INDEX, &call, &arrays, solution);
            let value = args_extractor.extract_set_value(Z_TERM_INDEX, &call, solution);

            if array_value != value {
                if verbose {
                    info!(
                        "Violated constraint: array_set_element {:?} = {:?}",
                        array_value, value
                    );
                }
                violation = array_value.difference(&value).count() as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_card` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn set_card(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let set_key: Option<String> = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let set_const: Option<HashSet<i64>> = if set_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let value_key: Option<String> = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let value_const: Option<i64> = if value_key.is_none() {
            Some(
                self.args_extractor
                    .extract_int_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let set = if let Some(ref s) = set_const {
                s.clone()
            } else {
                let key_ref = set_key
                    .as_ref()
                    .expect("Expected variable for set term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };

            let real_card = set.len();
            let value = if let Some(v) = value_const {
                v
            } else {
                let key_ref = value_key
                    .as_ref()
                    .expect("Expected variable for value term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Int(v)) => *v,
                    _ => panic!("Expected integer variable, found other type variable"),
                }
            };

            if real_card != value as usize {
                if verbose {
                    let set_display = if let Some(ref s) = set_const {
                        format!("{:?}", s)
                    } else {
                        set_key.as_ref().unwrap().to_string()
                    };
                    let value_display = if value_key.is_none() {
                        value.to_string()
                    } else {
                        value_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: set_card |{}| = {}",
                        set_display, value_display
                    );
                }
                violation = (real_card as i64 - value).abs() as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_diff` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the symmetric difference count if violated, 0.0 otherwise.
    pub fn set_diff(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let z_key = self.identifier_from_vars(&vars_involved, Z_TERM_INDEX);
        let z_const = if z_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Z_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let z_value = if let Some(ref s) = z_const {
                s.clone()
            } else {
                let key_ref = z_key
                    .as_ref()
                    .expect("Expected variable for z term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };

            let diff: HashSet<i64> = x_value.difference(&y_value).copied().collect();

            if diff != z_value {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    let z_display = if z_key.is_none() {
                        format!("{:?}", z_value)
                    } else {
                        z_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: set_diff {} \\ {} = {}",
                        x_display, y_display, z_display
                    );
                }

                violation = diff.symmetric_difference(&z_value).count() as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_eq` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the symmetric difference count if violated, 0.0 otherwise.
    pub fn set_eq(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };

            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };

            if x_value != y_value {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };

                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: set_eq {} = {}", x_display, y_display);
                }
                violation = x_value.symmetric_difference(&y_value).count() as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_eq_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let r_key = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);
        let r_const = if r_key.is_none() {
            Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let r_value = if let Some(v) = r_const {
                v
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable for r term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Bool(v)) => *v,
                    _ => panic!("Expected boolean variable, found other type variable"),
                }
            };

            if !(r_value == (x_value == y_value)) {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r_value.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated consraint: set_eq_reif {} <-> {:?} = {:?}",
                        r_display, x_display, y_display
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_in` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_in(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let elem_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let elem_const = if elem_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_element(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;
            let elem_value = if let Some(v) = elem_const {
                v
            } else {
                let key_ref = elem_key
                    .as_ref()
                    .expect("Expected variable for element term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Int(v)) => *v,
                    _ => panic!("Expected integer variable, found other type variable"),
                }
            };

            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };

            if !y_value.contains(&elem_value) {
                if verbose {
                    let elem_display = if elem_key.is_none() {
                        elem_value.to_string()
                    } else {
                        elem_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: set_in {} in {:?}",
                        elem_display, y_display
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_in_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_in_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let elem_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let elem_const = if elem_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_element(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let r_key = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);
        let r_const = if r_key.is_none() {
            Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;
            let elem_value = if let Some(v) = elem_const {
                v
            } else {
                let key_ref = elem_key
                    .as_ref()
                    .expect("Expected variable for element term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Int(v)) => *v,
                    _ => panic!("Expected integer variable, found other type variable"),
                }
            };
            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let r_value = if let Some(v) = r_const {
                v
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable for r term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Bool(v)) => *v,
                    _ => panic!("Expected boolean variable, found other type variable"),
                }
            };

            if !(r_value == y_value.contains(&elem_value)) {
                if verbose {
                    let elem_display = if elem_key.is_none() {
                        elem_value.to_string()
                    } else {
                        elem_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r_value.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: set_in_reif {} <-> {} in {:?}",
                        r_display, elem_display, y_display
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_intersect` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the set difference count if violated, 0.0 otherwise.
    pub fn set_intersect(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let z_key = self.identifier_from_vars(&vars_involved, Z_TERM_INDEX);
        let z_const = if z_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Z_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;
            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };

            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };

            let z_value = if let Some(ref s) = z_const {
                s.clone()
            } else {
                let key_ref = z_key
                    .as_ref()
                    .expect("Expected variable for z term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };

            let intersect: HashSet<i64> = x_value.intersection(&y_value).copied().collect();

            if intersect != z_value {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    let z_display = if z_key.is_none() {
                        format!("{:?}", z_value)
                    } else {
                        z_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: {} set_intersect {} = {}",
                        x_display, y_display, z_display
                    );
                }
                violation = intersect.difference(&z_value).count() as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_le` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the violation count if violated, 0.0 otherwise.
    pub fn set_le(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };

            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };

            let mut xv: Vec<i64> = x_value.iter().cloned().collect();
            let mut yv: Vec<i64> = y_value.iter().cloned().collect();

            xv.sort();
            yv.sort();

            if !(xv <= yv) {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: set_le {} <= {}", x_display, y_display);
                }

                for (i, elem) in yv.iter().enumerate() {
                    if *elem < xv[i] {
                        violation += 1.0;
                    }
                }
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_le_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let r_key = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);
        let r_const = if r_key.is_none() {
            Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;
            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let r_value = if let Some(v) = r_const {
                v
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable for r term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Bool(v)) => *v,
                    _ => panic!("Expected boolean variable, found other type variable"),
                }
            };

            let mut xv: Vec<i64> = x_value.iter().cloned().collect();
            let mut yv: Vec<i64> = y_value.iter().cloned().collect();

            xv.sort();
            yv.sort();

            if !(r_value == (xv <= yv)) {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r_value.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: set_le_reif {} <-> {} <= {}",
                        r_display, x_display, y_display
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_lt` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the violation count if violated, 0.0 otherwise.
    pub fn set_lt(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let verbose = self.verbose;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let x = args_extractor.extract_set_value(X_TERM_INDEX, &call, solution);
            let y = args_extractor.extract_set_value(Y_TERM_INDEX, &call, solution);

            let mut xv: Vec<i64> = x.iter().cloned().collect();
            let mut yv: Vec<i64> = y.iter().cloned().collect();

            xv.sort();
            yv.sort();

            if !(xv < yv) {
                if verbose {
                    info!("Violated: {:?} < {:?}", x, y);
                }

                for (i, elem) in yv.iter().enumerate() {
                    if *elem <= xv[i] {
                        violation += 1.0;
                    }
                }
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_lt_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_lt_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let r_key = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);
        let r_const = if r_key.is_none() {
            Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let r_value = if let Some(v) = r_const {
                v
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable for r term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Bool(v)) => *v,
                    _ => panic!("Expected boolean variable, found other type variable"),
                }
            };

            let mut xv: Vec<i64> = x_value.iter().cloned().collect();
            let mut yv: Vec<i64> = y_value.iter().cloned().collect();

            xv.sort();
            yv.sort();

            if !(r_value == (xv < yv)) {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r_value.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: set_lt_reif {} <-> {} < {}",
                        r_display, x_display, y_display
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_ne` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_ne(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };

            if x_value == y_value {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: set_ne {} != {}", x_display, y_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_ne_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_ne_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let r_key = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);
        let r_const = if r_key.is_none() {
            Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let r_value = if let Some(ref b) = r_const {
                *b
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable for r term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Bool(v)) => *v,
                    _ => panic!("Expected bool variable, found other type variable"),
                }
            };

            if !(r_value == (x_value != y_value)) {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r_value.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: set_ne_reif {} <-> {:?} != {:?}",
                        r_display, x_display, y_display
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_subset` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the set difference count if violated, 0.0 otherwise.
    pub fn set_subset(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };

            if !x_value.is_subset(&y_value) {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: set_subset {} subset {}",
                        x_display, y_display
                    );
                }
                violation = x_value.difference(&y_value).count() as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_subset_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_subset_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let r_key = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);
        let r_const = if r_key.is_none() {
            Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let r_value = if let Some(v) = r_const {
                v
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable for r term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Bool(v)) => *v,
                    _ => panic!("Expected boolean variable, found other type variable"),
                }
            };

            if !(r_value == x_value.is_subset(&y_value)) {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r_value.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: set_subset_reif {} <-> {} subset {}",
                        r_display, x_display, y_display
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_superset` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the set difference count if violated, 0.0 otherwise.
    pub fn set_superset(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };

            if !x_value.is_superset(&y_value) {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: set_superset {} superset {}",
                        x_display, y_display
                    );
                }
                violation = x_value.difference(&y_value).count() as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_superset_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_superset_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let r_key = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);
        let r_const = if r_key.is_none() {
            Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let r_value = if let Some(ref b) = r_const {
                *b
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable for r term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Bool(b)) => *b,
                    _ => panic!("Expected bool variable, found other type variable"),
                }
            };

            if !(r_value == x_value.is_superset(&y_value)) {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r_value.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: set_superset_reif {} <-> {} superset {}",
                        r_display, x_display, y_display
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_symdiff` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the set difference count if violated, 0.0 otherwise.
    pub fn set_symdiff(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let z_key = self.identifier_from_vars(&vars_involved, Z_TERM_INDEX);
        let z_const = if z_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Z_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;
            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let z_value = if let Some(ref s) = z_const {
                s.clone()
            } else {
                let key_ref = z_key
                    .as_ref()
                    .expect("Expected variable for z term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let sym_diff: HashSet<i64> = x_value.symmetric_difference(&y_value).copied().collect();

            if sym_diff != z_value {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    let z_display = if z_key.is_none() {
                        format!("{:?}", z_value)
                    } else {
                        z_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: set_symdiff {} sym diff {} = {}",
                        x_display, y_display, z_display
                    );
                }
                violation = sym_diff.difference(&z_value).count() as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_union` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `_solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the set difference count if violated, 0.0 otherwise.
    pub fn set_union(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let x_key = self.identifier_from_vars(&vars_involved, X_TERM_INDEX);
        let x_const = if x_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let y_key = self.identifier_from_vars(&vars_involved, Y_TERM_INDEX);
        let y_const = if y_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };
        let z_key = self.identifier_from_vars(&vars_involved, Z_TERM_INDEX);
        let z_const = if z_key.is_none() {
            Some(
                self.args_extractor
                    .extract_set_value(Z_TERM_INDEX, &constraint.call, _solution),
            )
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;
            let x_value = if let Some(ref s) = x_const {
                s.clone()
            } else {
                let key_ref = x_key
                    .as_ref()
                    .expect("Expected variable for x term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let y_value = if let Some(ref s) = y_const {
                s.clone()
            } else {
                let key_ref = y_key
                    .as_ref()
                    .expect("Expected variable for y term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let z_value = if let Some(ref s) = z_const {
                s.clone()
            } else {
                let key_ref = z_key
                    .as_ref()
                    .expect("Expected variable for z term in solution");
                match solution.get(key_ref) {
                    Some(VariableValue::Set(v)) => v.clone(),
                    _ => panic!("Expected set variable, found other type variable"),
                }
            };
            let union: HashSet<i64> = x_value.union(&y_value).copied().collect();

            if union != z_value {
                if verbose {
                    let x_display = if x_key.is_none() {
                        format!("{:?}", x_value)
                    } else {
                        x_key.as_ref().unwrap().to_string()
                    };
                    let y_display = if y_key.is_none() {
                        format!("{:?}", y_value)
                    } else {
                        y_key.as_ref().unwrap().to_string()
                    };
                    let z_display = if z_key.is_none() {
                        format!("{:?}", z_value)
                    } else {
                        z_key.as_ref().unwrap().to_string()
                    };
                    info!(
                        "Violated constraint: set_union {} U {} = {}",
                        x_display, y_display, z_display
                    );
                }
                violation = union.difference(&z_value).count() as f64;
            }

            violation
        })
    }

    fn identifier_from_vars(
        &self,
        vars: &HashMap<i64, Identifier>,
        index: usize,
    ) -> Option<String> {
        vars.get(&(index as i64)).map(|id| id.to_string())
    }
}

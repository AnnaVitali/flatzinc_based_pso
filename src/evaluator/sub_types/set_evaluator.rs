use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::data_utility::types::VariableValue;
use crate::{
    args_extractor::sub_types::set_args_extractor::SetArgsExtractor, data_utility::types::Register,
};
use flatzinc_serde::{Array, Literal};
use log::info;
use std::collections::{HashMap, HashSet};

pub const ARRAY_INDEX: i64 = 0;
pub const X_TERM_INDEX: i64 = 0;
pub const Y_TERM_INDEX: i64 = 1;
pub const Z_TERM_INDEX: i64 = 2;
pub const R_TERM_INDEX: i64 = 2;

#[derive(Debug, Clone, Default)]
/// Evaluator for set constraints, providing methods to evaluate various set operations and constraints.
///
/// This struct contains methods for evaluating set constraints such as equality, inequality, linear expressions,
/// and arithmetic operations. It uses argument extraction utilities and supports verbose output for debugging.
pub struct SetEvaluator {
    /// A hashmap that maps identifiers to their corresponding arrays, used for resolving array references in constraints.
    arrays: HashMap<String, Array>,
    /// A hashmap that maps variable identifiers to their corresponding registers, used for resolving variable references in constraints.
    variable_register_map: HashMap<String, Register>,
    /// An instance of `SetArgsExtractor` used to extract arguments from set constraints.
    args_extractor: SetArgsExtractor,
    /// A boolean flag indicating whether to enable verbose logging for constraint violations.
    verbose: bool,
}

impl SetEvaluator {
    /// Returns a new `SetFunctionalEvaluator` instance.
    ///
    /// # Arguments
    /// * `arrays` - The arrays used for evaluation.
    /// * `variable_map` - A map from variable names to their corresponding register IDs.
    /// * `verbose` - Whether to enable verbose logging.
    /// # Returns
    /// A new instance of `SetFunctionalEvaluator`.
    pub fn new(
        arrays: HashMap<String, Array>,
        variable_register_map: HashMap<String, Register>,
        verbose: bool,
    ) -> Self {
        let args_extractor = SetArgsExtractor::new();

        Self {
            arrays,
            variable_register_map,
            args_extractor,
            verbose,
        }
    }

    /// Returns a functional evaluator for the `array_set_element` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.

    /// # Returns
    /// A closure that evaluates the constraint and returns the set difference count if violated, 0.0 otherwise.
    pub fn array_set_element(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let index_register = self
            .variable_register_map
            .get(vars_involved.get(&X_TERM_INDEX).unwrap())
            .copied()
            .expect("Index register not found");
        let array: Vec<HashSet<i64>> = self
            .arrays
            .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
            .expect("Expect a constant array for array_int_element constraint")
            .contents
            .iter()
            .map(|elem| match elem {
                Literal::IntSet(range_list) => range_list
                    .iter()
                    .flat_map(|r| {
                        let start = *r.start();
                        let end = *r.end();
                        start..=end
                    })
                    .collect::<HashSet<i64>>(),
                _ => panic!("Expected int set literal"),
            })
            .collect();

        let value_register = self
            .variable_register_map
            .get(vars_involved.get(&Z_TERM_INDEX).unwrap())
            .copied()
            .expect("Value register not found");
        let verbose = self.verbose;

        Box::new(move |solution: &[VariableValue]| {
            let array_value = array[solution[index_register as usize].as_int() as usize].clone();
            let value = solution[value_register as usize].as_set();

            let mut violation = 0.0;
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

    /// Returns a functional evaluator for the `array_var_set_element` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
     /// # Returns
     /// A closure that evaluates the constraint and returns the set difference count if violated, 0.0 otherwise.
    pub fn array_var_set_element(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let index_register = self
            .variable_register_map
            .get(vars_involved.get(&X_TERM_INDEX).unwrap())
            .copied()
            .expect("Index register not found");
        let array: Vec<String> = self
            .arrays
            .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
            .expect("Expect a variable array for array_var_int_element constraint")
            .contents
            .iter()
            .map(|elem| match elem {
                Literal::Identifier(i) => i.clone(),
                _ => panic!("Expected identifier in array for array_var_int_element constraint"),
            })
            .collect();
        let array_registers: Vec<Register> = array
            .iter()
            .map(|var_name| {
                self.variable_register_map
                    .get(var_name)
                    .copied()
                    .expect(&format!("Variable {} not found in variable map", var_name))
            })
            .collect();

        let value_register = self
            .variable_register_map
            .get(vars_involved.get(&Z_TERM_INDEX).unwrap())
            .copied()
            .expect("Value register not found");
        let verbose = self.verbose;

        Box::new(move |solution: &[VariableValue]| {
            let array_values: Vec<HashSet<i64>> = array_registers
                .iter()
                .map(|reg| match solution[*reg as usize] {
                    VariableValue::Set(ref s) => s.clone(),
                    _ => panic!(
                        "Expected set variable in solution for array_var_set_element constraint"
                    ),
                })
                .collect();

            let array_value =
                array_values[solution[index_register as usize].as_int() as usize].clone();
            let value = solution[value_register as usize].as_set();

            let mut violation = 0.0;
            if array_value != value {
                if verbose {
                    info!(
                        "Violated constraint: array_var_set_element {:?} = {:?}",
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
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn set_card(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            a_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            a_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            b_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            b_const = Some(
                self.args_extractor
                    .extract_int_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_set();
            } else {
                a_value = a_const.clone().expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let real_card = a_value.len();
            if real_card != b_value as usize {
                if verbose {
                    info!(
                        "Violated constraint: set_card |{}| = {}",
                        real_card, b_value
                    );
                }
                violation = (real_card as i64 - b_value as i64).abs() as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `set_diff` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the symmetric difference count if violated, 0.0 otherwise.
    pub fn set_diff(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }
        let mut z_register = None;
        let mut z_const = None;
        if vars_involved.get(&Z_TERM_INDEX).is_some() {
            z_register = self
                .variable_register_map
                .get(vars_involved.get(&Z_TERM_INDEX).unwrap())
                .copied();
        } else {
            z_const = Some(
                self.args_extractor
                    .extract_set_value(Z_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            let z_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if z_register.is_some() {
                z_value = solution[z_register.unwrap() as usize].as_set();
            } else {
                z_value = z_const.clone().expect("Expected constant value for Z_TERM");
            }

            let mut violation = 0.0;

            let diff: HashSet<i64> = x_value.difference(&y_value).copied().collect();

            if diff != z_value {
                if verbose {
                    info!(
                        "Violated constraint: set_diff {:?} \\ {:?} = {:?}",
                        x_value, y_value, z_value
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
    /// # Returns
    /// A closure that evaluates the constraint and returns the symmetric difference count if violated, 0.0 otherwise.
    pub fn set_eq(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }

            let mut violation = 0.0;
            if x_value != y_value {
                if verbose {
                    info!("Violated constraint: set_eq {:?} = {:?}", x_value, y_value);
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
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_eq_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }
        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_register_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            let r_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM");
            }

            let mut violation = 0.0;
            if !(r_value == (x_value == y_value)) {
                if verbose {
                    info!(
                        "Violated consraint: set_eq_reif {} <-> {:?} = {:?}",
                        r_value, x_value, y_value
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
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_in(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_int_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_int();
            } else {
                x_value = x_const.expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }

            let mut violation = 0.0;
            if !y_value.contains(&x_value) {
                if verbose {
                    info!("Violated constraint: set_in {} in {:?}", x_value, y_value);
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
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_in_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_int_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }
        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_register_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            let r_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_int();
            } else {
                x_value = x_const.expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM");
            }

            let mut violation = 0.0;
            if !(r_value == y_value.contains(&x_value)) {
                if verbose {
                    info!(
                        "Violated constraint: set_in_reif {} <-> {} in {:?}",
                        r_value, x_value, y_value
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
    /// # Returns
    /// A closure that evaluates the constraint and returns the set difference count if violated, 0.0 otherwise.
    pub fn set_intersect(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }
        let mut z_register = None;
        let mut z_const = None;
        if vars_involved.get(&Z_TERM_INDEX).is_some() {
            z_register = self
                .variable_register_map
                .get(vars_involved.get(&Z_TERM_INDEX).unwrap())
                .copied();
        } else {
            z_const = Some(
                self.args_extractor
                    .extract_set_value(Z_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            let z_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if z_register.is_some() {
                z_value = solution[z_register.unwrap() as usize].as_set();
            } else {
                z_value = z_const.clone().expect("Expected constant value for Z_TERM");
            }

            let mut violation = 0.0;
            let intersect: HashSet<i64> = x_value.intersection(&y_value).copied().collect();

            if intersect != z_value {
                if verbose {
                    info!(
                        "Violated constraint: set_intersect {:?} intersect {:?} = {:?}",
                        x_value, y_value, z_value
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
    /// # Returns
    /// A closure that evaluates the constraint and returns the violation count if violated, 0.0 otherwise.
    pub fn set_le(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }

            let mut violation = 0.0;

            let mut xv: Vec<i64> = x_value.iter().cloned().collect();
            let mut yv: Vec<i64> = y_value.iter().cloned().collect();

            xv.sort();
            yv.sort();

            if !(xv <= yv) {
                if verbose {
                    info!("Violated constraint: set_le {:?} <= {:?}", x_value, y_value);
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
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_le_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }
        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_register_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            let r_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM");
            }

            let mut violation = 0.0;

            let mut xv: Vec<i64> = x_value.iter().cloned().collect();
            let mut yv: Vec<i64> = y_value.iter().cloned().collect();

            xv.sort();
            yv.sort();

            if !(r_value == (xv <= yv)) {
                if verbose {
                    info!(
                        "Violated constraint: set_le_reif {} <-> {:?} <= {:?}",
                        r_value, x_value, y_value
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
    /// # Returns
    /// A closure that evaluates the constraint and returns the violation count if violated, 0.0 otherwise.
    pub fn set_lt(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }

            let mut violation = 0.0;

            let mut xv: Vec<i64> = x_value.iter().cloned().collect();
            let mut yv: Vec<i64> = y_value.iter().cloned().collect();

            xv.sort();
            yv.sort();

            if !(xv < yv) {
                if verbose {
                    info!("Violated: {:?} < {:?}", x_value, y_value);
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
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_lt_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }
        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_register_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            let r_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM");
            }

            let mut violation = 0.0;

            let mut xv: Vec<i64> = x_value.iter().cloned().collect();
            let mut yv: Vec<i64> = y_value.iter().cloned().collect();

            xv.sort();
            yv.sort();

            if !(r_value == (xv < yv)) {
                if verbose {
                    info!(
                        "Violated constraint: set_lt_reif {} <-> {:?} < {:?}",
                        r_value, x_value, y_value
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
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_ne(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }

            let mut violation = 0.0;
            if x_value == y_value {
                if verbose {
                    info!("Violated constraint: set_ne {:?} != {:?}", x_value, y_value);
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
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_ne_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }
        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_register_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            let r_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM");
            }

            let mut violation = 0.0;

            if !(r_value == (x_value != y_value)) {
                if verbose {
                    info!(
                        "Violated constraint: set_ne_reif {} <-> {:?} != {:?}",
                        r_value, x_value, y_value
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
    /// # Returns
    /// A closure that evaluates the constraint and returns the set difference count if violated, 0.0 otherwise.
    pub fn set_subset(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }

            let mut violation = 0.0;
            if !x_value.is_subset(&y_value) {
                if verbose {
                    info!(
                        "Violated constraint: set_subset {:?} subset {:?}",
                        x_value, y_value
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
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_subset_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }
        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_register_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            let r_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM");
            }

            let mut violation = 0.0;

            if !(r_value == x_value.is_subset(&y_value)) {
                if verbose {
                    info!(
                        "Violated constraint: set_subset_reif {} <-> {:?} subset {:?}",
                        r_value, x_value, y_value
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
    /// # Returns
    /// A closure that evaluates the constraint and returns the set difference count if violated, 0.0 otherwise.
    pub fn set_superset(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }

            let mut violation = 0.0;
            if !x_value.is_superset(&y_value) {
                if verbose {
                    info!(
                        "Violated constraint: set_superset {:?} superset {:?}",
                        x_value, y_value
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
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn set_superset_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }
        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_register_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            let r_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM");
            }

            let mut violation = 0.0;

            if !(r_value == x_value.is_superset(&y_value)) {
                if verbose {
                    info!(
                        "Violated constraint: set_superset_reif {} <-> {:?} superset {:?}",
                        r_value, x_value, y_value
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
    /// # Returns
    /// A closure that evaluates the constraint and returns the set difference count if violated, 0.0 otherwise.
    pub fn set_symdiff(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }
        let mut z_register = None;
        let mut z_const = None;
        if vars_involved.get(&Z_TERM_INDEX).is_some() {
            z_register = self
                .variable_register_map
                .get(vars_involved.get(&Z_TERM_INDEX).unwrap())
                .copied();
        } else {
            z_const = Some(
                self.args_extractor
                    .extract_set_value(Z_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            let z_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if z_register.is_some() {
                z_value = solution[z_register.unwrap() as usize].as_set();
            } else {
                z_value = z_const.clone().expect("Expected constant value for Z_TERM");
            }

            let mut violation = 0.0;
            let sym_diff: HashSet<i64> = x_value.symmetric_difference(&y_value).copied().collect();

            if sym_diff != z_value {
                if verbose {
                    info!(
                        "Violated constraint: set_symdiff {:?} sym diff {:?} = {:?}",
                        x_value, y_value, z_value
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
    /// # Returns
    /// A closure that evaluates the constraint and returns the set difference count if violated, 0.0 otherwise.
    pub fn set_union(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_register_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }
        let mut y_register = None;
        let mut y_const = None;
        if vars_involved.get(&Y_TERM_INDEX).is_some() {
            y_register = self
                .variable_register_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }
        let mut z_register = None;
        let mut z_const = None;
        if vars_involved.get(&Z_TERM_INDEX).is_some() {
            z_register = self
                .variable_register_map
                .get(vars_involved.get(&Z_TERM_INDEX).unwrap())
                .copied();
        } else {
            z_const = Some(
                self.args_extractor
                    .extract_set_value(Z_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let x_value;
            let y_value;
            let z_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize].as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize].as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if z_register.is_some() {
                z_value = solution[z_register.unwrap() as usize].as_set();
            } else {
                z_value = z_const.clone().expect("Expected constant value for Z_TERM");
            }

            let mut violation = 0.0;
            let union: HashSet<i64> = x_value.union(&y_value).copied().collect();

            if union != z_value {
                if verbose {
                    info!(
                        "Violated constraint: set_union {:?} U {:?} = {:?}",
                        x_value, y_value, z_value
                    );
                }
                violation = union.difference(&z_value).count() as f64;
            }

            violation
        })
    }
}

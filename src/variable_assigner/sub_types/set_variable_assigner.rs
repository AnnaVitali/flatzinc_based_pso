use crate::args_extractor::sub_types::set_args_extractor::SetArgsExtractor;
use crate::data_utility::types::Register;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::evaluator::sub_types::set_evaluator::{R_TERM_INDEX, X_TERM_INDEX, Y_TERM_INDEX};
use crate::data_utility::types::VariableValue;
use flatzinc_serde::{Array, Literal};
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
    variable_map: HashMap<String, Register>,
    /// A hashmap that maps identifiers to their corresponding arrays, used for resolving array references in constraints.
    arrays: HashMap<String, Array>,
}

impl SetVariableAssigner {
    /// Creates a new `SetVariableAssigner` with the provided arrays.
    ///
    /// # Arguments
    /// * `arrays` - A map from identifiers to arrays used in set constraints.
    /// * `variable_map` - A map from variable names to their corresponding registers.
    ///
    /// # Returns
    /// A new instance of `SetVariableAssigner`.
    pub fn new(arrays: HashMap<String, Array>, variable_map: HashMap<String, Register>) -> Self {
        let args_extractor = SetArgsExtractor::new();

        Self {
            args_extractor,
            variable_map,
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
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> HashSet<i64> + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let index_register = self
            .variable_map
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            array[solution[index_register as usize]
                .as_ref()
                .expect("Missing value for register")
                .as_int() as usize]
                .clone()
        })
    }

    pub fn array_var_set_element(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> HashSet<i64> + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let index_register = self
            .variable_map
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
                self.variable_map
                    .get(var_name)
                    .copied()
                    .expect(&format!("Variable {} not found in variable map", var_name))
            })
            .collect();

        Box::new(move |solution: &[Option<VariableValue>]| {
            let array_values: Vec<HashSet<i64>> = array_registers
                .iter()
                .map(|reg| match solution[*reg as usize] {
                    Some(VariableValue::Set(ref s)) => s.clone(),
                    _ => panic!(
                        "Expected set variable in solution for array_var_set_element constraint"
                    ),
                })
                .collect();

            array_values[solution[index_register as usize]
                .as_ref()
                .expect("Missing value for register")
                .as_int() as usize]
                .clone()
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
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> HashSet<i64> + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_map
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
                .variable_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let x_value;
            let y_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }

            x_value.difference(&y_value).copied().collect()
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
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> HashSet<i64> + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_map
                .get(vars_involved.get(&X_TERM_INDEX).unwrap())
                .copied();
        } else {
            x_const = Some(
                self.args_extractor
                    .extract_set_value(X_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let x_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }

            x_value
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
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_map
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
                .variable_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else if y_register.is_none() {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let x_value;
            let y_value;
            let r_value: Option<bool>;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(
                    solution[r_register.unwrap() as usize]
                        .as_ref()
                        .expect("Missing value for register")
                        .as_bool(),
                );
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            } else {
                r_value = None;
            }

            if r_value.is_none() {
                x_value == y_value
            } else {
                (x_value == y_value) == r_value.unwrap()
            }
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
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_map
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
                .variable_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else if y_register.is_none() {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let x_value;
            let y_value;
            let r_value: Option<bool>;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_int();
            } else {
                x_value = x_const.expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(
                    solution[r_register.unwrap() as usize]
                        .as_ref()
                        .expect("Missing value for register")
                        .as_bool(),
                );
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            } else {
                r_value = None;
            }

            if r_value.is_none() {
                y_value.contains(&x_value)
            } else {
                (y_value.contains(&x_value)) == r_value.unwrap()
            }
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
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> HashSet<i64> + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_map
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
                .variable_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let x_value;
            let y_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }

            x_value.intersection(&y_value).copied().collect()
        })
    }

    /// Returns a closure that evaluates the `set_lt_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one set is less than another (by sorted order).
    pub fn set_le_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_map
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
                .variable_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else if y_register.is_none() {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let x_value;
            let y_value;
            let r_value: Option<bool>;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(
                    solution[r_register.unwrap() as usize]
                        .as_ref()
                        .expect("Missing value for register")
                        .as_bool(),
                );
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            } else {
                r_value = None;
            }

            let mut xv: Vec<i64> = x_value.iter().cloned().collect();
            let mut yv: Vec<i64> = y_value.iter().cloned().collect();

            xv.sort();
            yv.sort();

            if r_value.is_none() {
                xv <= yv
            } else {
                (xv <= yv) == r_value.unwrap()
            }
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
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_map
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
                .variable_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else if y_register.is_none() {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let x_value;
            let y_value;
            let r_value: Option<bool>;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(
                    solution[r_register.unwrap() as usize]
                        .as_ref()
                        .expect("Missing value for register")
                        .as_bool(),
                );
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            } else {
                r_value = None;
            }

            let mut xv: Vec<i64> = x_value.iter().cloned().collect();
            let mut yv: Vec<i64> = y_value.iter().cloned().collect();

            xv.sort();
            yv.sort();

            if r_value.is_none() {
                xv < yv
            } else {
                (xv < yv) == r_value.unwrap()
            }
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
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_map
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
                .variable_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else if y_register.is_none() {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let x_value;
            let y_value;
            let r_value: Option<bool>;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(
                    solution[r_register.unwrap() as usize]
                        .as_ref()
                        .expect("Missing value for register")
                        .as_bool(),
                );
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            } else {
                r_value = None;
            }

            if r_value.is_none() {
                x_value != y_value
            } else {
                (x_value != y_value) == r_value.unwrap()
            }
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
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_map
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
                .variable_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else if y_register.is_none() {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let x_value;
            let y_value;
            let r_value: Option<bool>;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(
                    solution[r_register.unwrap() as usize]
                        .as_ref()
                        .expect("Missing value for register")
                        .as_bool(),
                );
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            } else {
                r_value = None;
            }

            if r_value.is_none() {
                x_value.is_subset(&y_value)
            } else {
                (x_value.is_subset(&y_value)) == r_value.unwrap()
            }
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
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_map
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
                .variable_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else if y_register.is_none() {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_map
                .get(vars_involved.get(&R_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let x_value;
            let y_value;
            let r_value: Option<bool>;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }
            if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(
                    solution[r_register.unwrap() as usize]
                        .as_ref()
                        .expect("Missing value for register")
                        .as_bool(),
                );
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            } else {
                r_value = None;
            }

            if r_value.is_none() {
                x_value.is_superset(&y_value)
            } else {
                (x_value.is_superset(&y_value)) == r_value.unwrap()
            }
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
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> HashSet<i64> + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_map
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
                .variable_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let x_value;
            let y_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }

            x_value.symmetric_difference(&y_value).copied().collect()
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
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> HashSet<i64> + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut x_register = None;
        let mut x_const = None;
        if vars_involved.get(&X_TERM_INDEX).is_some() {
            x_register = self
                .variable_map
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
                .variable_map
                .get(vars_involved.get(&Y_TERM_INDEX).unwrap())
                .copied();
        } else {
            y_const = Some(
                self.args_extractor
                    .extract_set_value(Y_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let x_value;
            let y_value;
            if x_register.is_some() {
                x_value = solution[x_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                x_value = x_const.clone().expect("Expected constant value for X_TERM");
            }
            if y_register.is_some() {
                y_value = solution[y_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_set();
            } else {
                y_value = y_const.clone().expect("Expected constant value for Y_TERM");
            }

            x_value.union(&y_value).copied().collect()
        })
    }
}

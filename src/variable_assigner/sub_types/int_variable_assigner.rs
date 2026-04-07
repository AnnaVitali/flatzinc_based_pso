use crate::args_extractor::sub_types::int_args_extractor::IntArgsExtractor;
use crate::data_utility::types::Register;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::evaluator::sub_types::int_evaluator::{
    A_TERM_INDEX, B_TERM_INDEX, COEFF_LIN_CONSTR_INDEX, CONST_LIN_CONSTR_INDEX, R_TERM_INDEX,
    VARS_LIN_CONSTR_INDEX,
};
use crate::data_utility::types::VariableValue;
use flatzinc_serde::{Array, Literal};
use std::collections::HashMap;

/// Struct responsible for assigning integer variables based on constraints and solutions.
///
/// # Fields
/// * `args_extractor` - Extracts arguments for integer constraints.
/// * `arrays` - Stores arrays mapped by their identifiers.
#[derive(Debug, Clone, Default)]
pub struct IntVariableAssigner {
    /// An instance of `IntArgsExtractor` used to extract arguments from integer constraints.
    args_extractor: IntArgsExtractor,
    /// A hashmap that maps variable identifiers to their corresponding registers, used for resolving variable references in constraints.
    variable_register_map: HashMap<String, Register>,
    /// A hashmap that maps identifiers to their corresponding arrays, used for resolving array references in constraints.
    arrays: HashMap<String, Array>,
}

impl IntVariableAssigner {
    /// Creates a new `IntVariableAssigner` with the provided arrays.
    ///
    /// # Arguments
    /// * `arrays` - A map from identifiers to arrays used in integer constraints.
    ///
    /// # Returns
    /// A new instance of `IntVariableAssigner`.
    pub fn new(arrays: HashMap<String, Array>, variable_map: HashMap<String, Register>) -> Self {
        let args_extractor = IntArgsExtractor::new();

        Self {
            args_extractor,
            variable_register_map: variable_map,
            arrays,
        }
    }

    /// Returns a closure that evaluates the `array_int_element` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the integer value from the array or a specific value depending on the defined variable.
    pub fn array_int_element(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> i64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let index_register = self
            .variable_register_map
            .get(vars_involved.get(&A_TERM_INDEX).unwrap())
            .copied()
            .expect("Index register not found");
        let array: Vec<i64> = self
            .arrays
            .get(vars_involved.get(&B_TERM_INDEX).unwrap())
            .expect("Expect a constant array for array_int_element constraint")
            .contents
            .iter()
            .map(|elem| match elem {
                Literal::Int(i) => *i,
                _ => panic!("Expected int literal in array for array_int_element constraint"),
            })
            .collect();
        Box::new(move |solution: &[Option<VariableValue>]| {
            array[solution[index_register as usize]
                .as_ref()
                .expect("Missing value for register")
                .as_int() as usize]
        })
    }

    pub fn array_var_int_element(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> i64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let index_register = self
            .variable_register_map
            .get(vars_involved.get(&A_TERM_INDEX).unwrap())
            .copied()
            .expect("Index register not found");
        let array: Vec<String> = self
            .arrays
            .get(vars_involved.get(&B_TERM_INDEX).unwrap())
            .expect("Expect a variable array for array_var_int_element constraint")
            .contents
            .iter()
            .map(|elem| match elem {
                Literal::Identifier(i) => i.clone(),
                _ => panic!("Expected identifier in array for array_var_int_element constraint"),
            })
            .collect();
        let variable_map = self.variable_register_map.clone();

        Box::new(move |solution: &[Option<VariableValue>]| {
            let idx_in_array = solution[index_register as usize]
                .as_ref()
                .expect("Missing value for register")
                .as_int() as usize;
            let var_name = &array[idx_in_array];
            let var_idx = variable_map
                .get(var_name)
                .copied()
                .expect("Array value not found") as usize;

            solution[var_idx]
                .as_ref()
                .expect("Missing value for register")
                .as_int()
        })
    }

    /// Returns a closure that evaluates the `int_abs` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the absolute value of an integer or a specific value depending on the defined variable.
    pub fn int_abs(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> i64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self
                .variable_register_map
                .get(vars_involved.get(&A_TERM_INDEX).unwrap())
                .copied();
        } else {
            a_const = Some(
                self.args_extractor
                    .extract_int_value(A_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.abs()
        })
    }

    /// Returns a closure that evaluates the `int_div` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the division of two integer values or a specific value depending on the defined variable.
    pub fn int_div(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> i64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self
                .variable_register_map
                .get(vars_involved.get(&A_TERM_INDEX).unwrap())
                .copied();
        } else {
            a_const = Some(
                self.args_extractor
                    .extract_int_value(A_TERM_INDEX, &constraint.call),
            );
        }

        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self
                .variable_register_map
                .get(vars_involved.get(&B_TERM_INDEX).unwrap())
                .copied();
        } else {
            b_const = Some(
                self.args_extractor
                    .extract_int_value(B_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            a_value / b_value
        })
    }

    /// Returns a closure that evaluates the `int_eq` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the equality of two integer values or a specific value depending on the defined variable.
    pub fn int_eq(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> i64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self
                .variable_register_map
                .get(vars_involved.get(&A_TERM_INDEX).unwrap())
                .copied();
        } else {
            a_const = Some(
                self.args_extractor
                    .extract_int_value(A_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value
        })
    }

    /// Returns a closure that evaluates the `int_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether two integer values are equal, or a boolean value depending on the defined variable.
    pub fn int_eq_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self
                .variable_register_map
                .get(vars_involved.get(&A_TERM_INDEX).unwrap())
                .copied();
        } else {
            a_const = Some(
                self.args_extractor
                    .extract_int_value(A_TERM_INDEX, &constraint.call),
            );
        }

        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self
                .variable_register_map
                .get(vars_involved.get(&B_TERM_INDEX).unwrap())
                .copied();
        } else if b_register.is_none() {
            b_const = Some(
                self.args_extractor
                    .extract_int_value(B_TERM_INDEX, &constraint.call),
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            let r_value: Option<bool>;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
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
                a_value == b_value
            } else {
                (a_value == b_value) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `int_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one integer value is less than or equal to another, or a boolean value depending on the defined variable.
    pub fn int_le_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self
                .variable_register_map
                .get(vars_involved.get(&A_TERM_INDEX).unwrap())
                .copied();
        } else {
            a_const = Some(
                self.args_extractor
                    .extract_int_value(A_TERM_INDEX, &constraint.call),
            );
        }

        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self
                .variable_register_map
                .get(vars_involved.get(&B_TERM_INDEX).unwrap())
                .copied();
        } else if b_register.is_none() {
            b_const = Some(
                self.args_extractor
                    .extract_int_value(B_TERM_INDEX, &constraint.call),
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            let r_value: Option<bool>;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
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
                a_value <= b_value
            } else {
                (a_value <= b_value) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `int_lin_eq` constraint for a specific variable.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    /// * `variable` - The variable to solve for in the linear equation.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the value of the specified variable in the linear equation.
    pub fn int_lin_eq(
        &self,
        constraint: &CallWithDefines,
        variable: &String,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> i64 + Send + Sync> {
        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self.args_extractor.extract_var_values_lin_expr(
            VARS_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let mut registers = Vec::new();
        for var in &vars_involved {
            let reg = self
                .variable_register_map
                .get(var)
                .copied()
                .expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let var_idx = vars_involved
            .iter()
            .position(|id| id == variable)
            .expect("Assigned variable not found in vars_involved");
        let var_coeff = coeff[var_idx];
        let constant_term: i64 = self
            .args_extractor
            .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[Option<VariableValue>]| {
            let mut sum = 0i64;
            for (i, &reg) in registers.iter().enumerate() {
                if i != var_idx {
                    sum += coeff[i]
                        * solution[reg as usize]
                            .as_ref()
                            .expect("Missing value for register")
                            .as_int();
                }
            }
            let result = (constant_term - sum) / var_coeff;
            result
        })
    }

    /// Returns a closure that evaluates the `int_lin_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear equation holds, or a boolean value depending on the defined variable.
    pub fn int_lin_eq_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self.args_extractor.extract_var_values_lin_expr(
            VARS_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let mut registers = Vec::new();
        for var in &vars_involved {
            let reg = self
                .variable_register_map
                .get(var)
                .copied()
                .expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let defines = constraint.defines.clone();
        let r_register = defines
            .as_ref()
            .and_then(|r| self.variable_register_map.get(r).copied());
        let r_const = if r_register.is_none() {
            Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            )
        } else {
            None
        };
        let constant_term: i64 = self
            .args_extractor
            .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[Option<VariableValue>]| {
            let left_side_term = {
                let mut sum = 0i64;
                for (i, &reg) in registers.iter().enumerate() {
                    sum += coeff[i]
                        * solution[reg as usize]
                            .as_ref()
                            .expect("Missing value for register")
                            .as_int();
                }
                sum
            };
            let r_value: Option<bool>;
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
                left_side_term == constant_term
            } else {
                (left_side_term == constant_term) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `int_lin_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear expression is less than or equal to the term, or a boolean value depending on the defined variable.
    pub fn int_lin_le_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self.args_extractor.extract_var_values_lin_expr(
            VARS_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let mut registers = Vec::new();
        for var in &vars_involved {
            let reg = self
                .variable_register_map
                .get(var)
                .copied()
                .expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let defines = constraint.defines.clone();
        let r_register = defines
            .as_ref()
            .and_then(|r| self.variable_register_map.get(r).copied());
        let r_const = if r_register.is_none() {
            Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            )
        } else {
            None
        };
        let constant_term: i64 = self
            .args_extractor
            .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[Option<VariableValue>]| {
            let left_side_term = {
                let mut sum = 0i64;
                for (i, &reg) in registers.iter().enumerate() {
                    sum += coeff[i]
                        * solution[reg as usize]
                            .as_ref()
                            .expect("Missing value for register")
                            .as_int();
                }
                sum
            };
            let r_value: Option<bool>;
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
                left_side_term <= constant_term
            } else {
                (left_side_term <= constant_term) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `int_lin_ne_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear expression is not equal to the term, or a boolean value depending on the defined variable.
    pub fn int_lin_ne_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self.args_extractor.extract_var_values_lin_expr(
            VARS_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let mut registers = Vec::new();
        for var in &vars_involved {
            let reg = self
                .variable_register_map
                .get(var)
                .copied()
                .expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let defines = constraint.defines.clone();
        let r_register = defines
            .as_ref()
            .and_then(|r| self.variable_register_map.get(r).copied());
        let r_const = if r_register.is_none() {
            Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_INDEX, &constraint.call),
            )
        } else {
            None
        };
        let constant_term: i64 = self
            .args_extractor
            .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[Option<VariableValue>]| {
            let left_side_term = {
                let mut sum = 0i64;
                for (i, &reg) in registers.iter().enumerate() {
                    sum += coeff[i]
                        * solution[reg as usize]
                            .as_ref()
                            .expect("Missing value for register")
                            .as_int();
                }
                sum
            };
            let r_value: Option<bool>;
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
                left_side_term != constant_term
            } else {
                (left_side_term != constant_term) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `int_lt_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one integer value is less than another, or a boolean value depending on the defined variable.
    pub fn int_lt_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self
                .variable_register_map
                .get(vars_involved.get(&A_TERM_INDEX).unwrap())
                .copied();
        } else {
            a_const = Some(
                self.args_extractor
                    .extract_int_value(A_TERM_INDEX, &constraint.call),
            );
        }

        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self
                .variable_register_map
                .get(vars_involved.get(&B_TERM_INDEX).unwrap())
                .copied();
        } else if b_register.is_none() {
            b_const = Some(
                self.args_extractor
                    .extract_int_value(B_TERM_INDEX, &constraint.call),
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value = if let Some(a_reg) = a_register {
                solution[a_reg as usize]
                    .as_ref()
                    .expect("Missing value for a_register")
                    .as_int()
            } else {
                a_const.expect("Expected constant value for A_TERM")
            };
            let b_value = if let Some(b_reg) = b_register {
                solution[b_reg as usize]
                    .as_ref()
                    .expect("Missing value for b_register")
                    .as_int()
            } else {
                b_const.expect("Expected constant value for B_TERM")
            };
            let r_value: Option<bool> = if let Some(r_reg) = r_register {
                Some(
                    solution[r_reg as usize]
                        .as_ref()
                        .expect("Missing value for r_register")
                        .as_bool(),
                )
            } else if let Some(r_c) = r_const {
                Some(r_c)
            } else {
                None
            };

            if r_value.is_none() {
                a_value < b_value
            } else {
                (a_value < b_value) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `int_max` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the maximum of two integer values or a specific value depending on the defined variable.
    pub fn int_max(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> i64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self
                .variable_register_map
                .get(vars_involved.get(&A_TERM_INDEX).unwrap())
                .copied();
        } else {
            a_const = Some(
                self.args_extractor
                    .extract_int_value(A_TERM_INDEX, &constraint.call),
            );
        }

        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self
                .variable_register_map
                .get(vars_involved.get(&B_TERM_INDEX).unwrap())
                .copied();
        } else {
            b_const = Some(
                self.args_extractor
                    .extract_int_value(B_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value = if let Some(a_reg) = a_register {
                solution[a_reg as usize]
                    .as_ref()
                    .expect("Missing value for a_register")
                    .as_int()
            } else {
                a_const.expect("Expected constant value for A_TERM")
            };
            let b_value = if let Some(b_reg) = b_register {
                solution[b_reg as usize]
                    .as_ref()
                    .expect("Missing value for b_register")
                    .as_int()
            } else {
                b_const.expect("Expected constant value for B_TERM")
            };

            a_value.max(b_value)
        })
    }

    /// Returns a closure that evaluates the `int_min` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the minimum of two integer values or a specific value depending on the defined variable.
    pub fn int_min(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> i64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self
                .variable_register_map
                .get(vars_involved.get(&A_TERM_INDEX).unwrap())
                .copied();
        } else {
            a_const = Some(
                self.args_extractor
                    .extract_int_value(A_TERM_INDEX, &constraint.call),
            );
        }

        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self
                .variable_register_map
                .get(vars_involved.get(&B_TERM_INDEX).unwrap())
                .copied();
        } else {
            b_const = Some(
                self.args_extractor
                    .extract_int_value(B_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value = if let Some(a_reg) = a_register {
                solution[a_reg as usize]
                    .as_ref()
                    .expect("Missing value for a_register")
                    .as_int()
            } else {
                a_const.expect("Expected constant value for A_TERM")
            };
            let b_value = if let Some(b_reg) = b_register {
                solution[b_reg as usize]
                    .as_ref()
                    .expect("Missing value for b_register")
                    .as_int()
            } else {
                b_const.expect("Expected constant value for B_TERM")
            };

            a_value.min(b_value)
        })
    }

    /// Returns a closure that evaluates the `int_mod` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the remainder of the division of two integer values or a specific value depending on the defined variable.
    pub fn int_mod(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> i64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self
                .variable_register_map
                .get(vars_involved.get(&A_TERM_INDEX).unwrap())
                .copied();
        } else {
            a_const = Some(
                self.args_extractor
                    .extract_int_value(A_TERM_INDEX, &constraint.call),
            );
        }

        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self
                .variable_register_map
                .get(vars_involved.get(&B_TERM_INDEX).unwrap())
                .copied();
        } else {
            b_const = Some(
                self.args_extractor
                    .extract_int_value(B_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value = if let Some(a_reg) = a_register {
                solution[a_reg as usize]
                    .as_ref()
                    .expect("Missing value for a_register")
                    .as_int()
            } else {
                a_const.expect("Expected constant value for A_TERM")
            };
            let b_value = if let Some(b_reg) = b_register {
                solution[b_reg as usize]
                    .as_ref()
                    .expect("Missing value for b_register")
                    .as_int()
            } else {
                b_const.expect("Expected constant value for B_TERM")
            };

            a_value % b_value
        })
    }

    /// Returns a closure that evaluates the `int_ne_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether two integer values are not equal, or a boolean value depending on the defined variable.
    pub fn int_ne_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self
                .variable_register_map
                .get(vars_involved.get(&A_TERM_INDEX).unwrap())
                .copied();
        } else {
            a_const = Some(
                self.args_extractor
                    .extract_int_value(A_TERM_INDEX, &constraint.call),
            );
        }

        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self
                .variable_register_map
                .get(vars_involved.get(&B_TERM_INDEX).unwrap())
                .copied();
        } else if b_register.is_none() {
            b_const = Some(
                self.args_extractor
                    .extract_int_value(B_TERM_INDEX, &constraint.call),
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value = if let Some(a_reg) = a_register {
                solution[a_reg as usize]
                    .as_ref()
                    .expect("Missing value for a_register")
                    .as_int()
            } else {
                a_const.expect("Expected constant value for A_TERM")
            };
            let b_value = if let Some(b_reg) = b_register {
                solution[b_reg as usize]
                    .as_ref()
                    .expect("Missing value for b_register")
                    .as_int()
            } else {
                b_const.expect("Expected constant value for B_TERM")
            };
            let r_value: Option<bool>;
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
                a_value != b_value
            } else {
                (a_value != b_value) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `int_pow` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the result of raising one integer to the power of another or a specific value depending on the defined variable.
    pub fn int_pow(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> i64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self
                .variable_register_map
                .get(vars_involved.get(&A_TERM_INDEX).unwrap())
                .copied();
        } else {
            a_const = Some(
                self.args_extractor
                    .extract_int_value(A_TERM_INDEX, &constraint.call),
            );
        }

        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self
                .variable_register_map
                .get(vars_involved.get(&B_TERM_INDEX).unwrap())
                .copied();
        } else {
            b_const = Some(
                self.args_extractor
                    .extract_int_value(B_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value = if let Some(a_reg) = a_register {
                solution[a_reg as usize]
                    .as_ref()
                    .expect("Missing value for a_register")
                    .as_int()
            } else {
                a_const.expect("Expected constant value for A_TERM")
            };
            let b_value = if let Some(b_reg) = b_register {
                solution[b_reg as usize]
                    .as_ref()
                    .expect("Missing value for b_register")
                    .as_int()
            } else {
                b_const.expect("Expected constant value for B_TERM")
            };

            a_value.pow(b_value as u32)
        })
    }

    /// Returns a closure that evaluates the `int_times` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the product of two integer values or a specific value depending on the defined variable.
    pub fn int_times(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> i64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self
                .variable_register_map
                .get(vars_involved.get(&A_TERM_INDEX).unwrap())
                .copied();
        } else {
            a_const = Some(
                self.args_extractor
                    .extract_int_value(A_TERM_INDEX, &constraint.call),
            );
        }

        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self
                .variable_register_map
                .get(vars_involved.get(&B_TERM_INDEX).unwrap())
                .copied();
        } else {
            b_const = Some(
                self.args_extractor
                    .extract_int_value(B_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value = if let Some(a_reg) = a_register {
                solution[a_reg as usize]
                    .as_ref()
                    .expect("Missing value for a_register")
                    .as_int()
            } else {
                a_const.expect("Expected constant value for A_TERM")
            };
            let b_value = if let Some(b_reg) = b_register {
                solution[b_reg as usize]
                    .as_ref()
                    .expect("Missing value for b_register")
                    .as_int()
            } else {
                b_const.expect("Expected constant value for B_TERM")
            };
            a_value * b_value
        })
    }
}

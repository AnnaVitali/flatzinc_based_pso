use crate::args_extractor::sub_types::bool_args_extractor::BoolArgsExtractor;
use crate::data_utility::types::Register;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::evaluator::sub_types::bool_evaluator::{
    A_TERM_INDEX, AS_ARRAY_INDEX, B_TERM_INDEX, BS_ARRAY_INDEX, C_TERM_INDEX, R_TERM_INDEX,
};
use crate::data_utility::types::VariableValue;
use flatzinc_serde::{Array, Literal};
use std::collections::HashMap;

/// Struct responsible for assigning boolean variables based on constraints and solutions.
///
/// # Fields
/// * `args_extractor` - Extracts arguments for boolean constraints.
/// * `arrays` - Stores arrays mapped by their identifiers.
#[derive(Debug, Clone, Default)]
pub struct BoolVariableAssigner {
    /// An instance of `BoolArgsExtractor` used to extract arguments from boolean constraints.
    args_extractor: BoolArgsExtractor,
    // A hashmap that maps variable identifiers to their corresponding registers, used for resolving variable references in constraints.
    variable_register_map: HashMap<String, Register>,
    /// A hashmap that maps identifiers to their corresponding arrays, used for resolving array references in constraints.
    arrays: HashMap<String, Array>,
}

impl BoolVariableAssigner {
    /// Creates a new `BoolVariableAssigner` with the provided arrays.
    ///
    /// # Arguments
    /// * `arrays` - A map from identifiers to arrays used in boolean constraints.
    ///
    /// # Returns
    /// A new instance of `BoolVariableAssigner`.
    pub fn new(arrays: HashMap<String, Array>, variable_register_map: HashMap<String, Register>) -> Self {
        let args_extractor = BoolArgsExtractor::new();

        Self {
            args_extractor,
            variable_register_map,
            arrays,
        }
    }

    /// Returns a closure that evaluates the `array_bool_and` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns `true` if all elements in the array are true, or matches the expected result.
    pub fn array_bool_and(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let array: Vec<String> = self.args_extractor.extract_bool_array(
            AS_ARRAY_INDEX as usize,
            &constraint.call.args,
            &self.arrays,
        );
        let mut vars_register = Vec::with_capacity(array.len());
        for var_name in &array {
            let var_register = self
                .variable_register_map
                .get(var_name)
                .copied()
                .expect("Array value not found in variable map");
            vars_register.push(var_register);
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
            let r_value: Option<bool>;
            let mut array_values = Vec::with_capacity(vars_register.len());
            for (_, var_register) in vars_register.iter().enumerate() {
                let value = solution[*var_register as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
                array_values.push(value);
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
                array_values.iter().all(|&item| item)
            } else {
                array_values.iter().all(|&item| item) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `array_bool_element` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the result of extracting a boolean element from the array.
    pub fn array_bool_element(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let index_register = self
            .variable_register_map
            .get(vars_involved.get(&A_TERM_INDEX).unwrap())
            .copied()
            .expect("Index register not found");
        let array: Vec<bool> = self
            .arrays
            .get(vars_involved.get(&B_TERM_INDEX).unwrap())
            .expect("Expect a constant array for array_bool_element constraint")
            .contents
            .iter()
            .map(|elem| match elem {
                Literal::Bool(b) => *b,
                _ => panic!("Expected bool literal in array for array_bool_element constraint"),
            })
            .collect();

        Box::new(move |solution: &[Option<VariableValue>]| {
            array[solution[index_register as usize]
                .as_ref()
                .expect("Missing value for register")
                .as_int() as usize]
        })
    }

    /// Returns a closure that evaluates the `array_var_bool_element` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    /// # Returns
    /// A closure that, given a solution, returns the result of extracting a boolean element from an array of variables based on the index provided in the solution.
    pub fn array_var_bool_element(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
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
            .expect("Expect a variable array for array_var_bool_element constraint")
            .contents
            .iter()
            .map(|elem| match elem {
                Literal::Identifier(i) => i.clone(),
                _ => panic!("Expected identifier in array for array_var_bool_element constraint"),
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
                .expect("Missing value for variable")
                .as_bool()
        })
    }

    /// Returns a closure that evaluates the `bool_and` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the logical AND of two boolean values, or a single value depending on the defined variable.
    pub fn bool_and(
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
                    .extract_bool_value(A_TERM_INDEX, &constraint.call),
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
                    .extract_bool_value(B_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            a_value && b_value
        })
    }

    /// Returns a closure that evaluates the `bool_clause` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns `true` if any element in the array is true or if all elements in another array are not false.
    pub fn bool_clause(
        &self,
        constraint: &CallWithDefines,
        variable: &String,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let as_array: Vec<String> = self.args_extractor.extract_bool_array(
            AS_ARRAY_INDEX as usize,
            &constraint.call.args,
            &self.arrays,
        );
        let mut as_vars_register = Vec::with_capacity(as_array.len());
        for var_name in &as_array {
            if var_name != variable {
                let var_register = self
                    .variable_register_map
                    .get(var_name)
                    .copied()
                    .expect("Array value not found in variable map");
                as_vars_register.push(var_register);
            }
        }

        let bs_array: Vec<String> = self.args_extractor.extract_bool_array(
            BS_ARRAY_INDEX as usize,
            &constraint.call.args,
            &self.arrays,
        );
        let mut bs_vars_register = Vec::with_capacity(bs_array.len());
        for var_name in &bs_array {
            if var_name != variable {
                let var_register = self
                    .variable_register_map
                    .get(var_name)
                    .copied()
                    .expect("Array value not found in variable map");
                bs_vars_register.push(var_register);
            }
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let mut as_values = Vec::with_capacity(as_vars_register.len());
            for &var_register in &as_vars_register {
                let value = solution[var_register as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
                as_values.push(value);
            }
            let mut bs_values = Vec::with_capacity(bs_vars_register.len());
            for &var_register in &bs_vars_register {
                let value = solution[var_register as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
                bs_values.push(value);
            }
            as_values.iter().any(|&item| item) || !bs_values.iter().any(|&item| !item)
        })
    }

    /// Returns a closure that evaluates the `bool_eq` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the equality of two boolean values or a single value depending on the defined variable.
    pub fn bool_eq(
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
                    .extract_bool_value(A_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value
        })
    }

    /// Returns a closure that evaluates the `bool_not` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the logical NOT of a boolean value or a single value depending on the defined variable.
    pub fn bool_not(
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
                    .extract_bool_value(A_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            !a_value
        })
    }

    /// Returns a closure that evaluates the `bool_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether two boolean values are equal, or a single value depending on the defined variable.
    pub fn bool_eq_reif(
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
                    .extract_bool_value(A_TERM_INDEX, &constraint.call),
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
                    .extract_bool_value(B_TERM_INDEX, &constraint.call),
            );
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            r_register = self
                .variable_register_map
                .get(vars_involved.get(&C_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(C_TERM_INDEX, &constraint.call),
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
                    .as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
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

    /// Returns a closure that evaluates the `bool_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one boolean value is less than or equal to another, or a single value depending on the defined variable.
    pub fn bool_le_reif(
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
                    .extract_bool_value(A_TERM_INDEX, &constraint.call),
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
                    .extract_bool_value(B_TERM_INDEX, &constraint.call),
            );
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self
                .variable_register_map
                .get(vars_involved.get(&C_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(C_TERM_INDEX, &constraint.call),
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
                    .as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
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

    /// Returns a closure that evaluates the `bool_lt_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one boolean value is less than another, or a single value depending on the defined variable.
    pub fn bool_lt_reif(
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
                    .extract_bool_value(A_TERM_INDEX, &constraint.call),
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
                    .extract_bool_value(B_TERM_INDEX, &constraint.call),
            );
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            r_register = self
                .variable_register_map
                .get(vars_involved.get(&C_TERM_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(C_TERM_INDEX, &constraint.call),
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
                    .as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
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
                a_value < b_value
            } else {
                (a_value < b_value) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `bool_or` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the logical OR of two boolean values, or a single value depending on the defined variable.
    pub fn bool_or(
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
                    .extract_bool_value(A_TERM_INDEX, &constraint.call),
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
                    .extract_bool_value(B_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            a_value || b_value
        })
    }

    /// Returns a closure that evaluates the `bool_xor` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the logical XOR of two boolean values, or a single value depending on the defined variable.
    pub fn bool_xor(
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
                    .extract_bool_value(A_TERM_INDEX, &constraint.call),
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
                    .extract_bool_value(B_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            a_value ^ b_value
        })
    }

    /// Returns a closure that evaluates the `bool2int` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the integer representation of a boolean value, or an integer value depending on the defined variable.
    pub fn bool2int(
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
                    .extract_bool_value(A_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize]
                    .as_ref()
                    .expect("Missing value for register")
                    .as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value as i64
        })
    }
}

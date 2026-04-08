use crate::args_extractor::sub_types::bool_args_extractor::BoolArgsExtractor;
use crate::data_utility::types::Register;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::data_utility::types::VariableValue;
use flatzinc_serde::{Array, Literal};
use log::info;
use std::collections::HashMap;

pub const A_TERM_INDEX: i64 = 0;
pub const B_TERM_INDEX: i64 = 1;
pub const C_TERM_INDEX: i64 = 2;
pub const AS_ARRAY_INDEX: i64 = 0;
pub const BS_ARRAY_INDEX: i64 = 1;
pub const R_TERM_INDEX: i64 = 1;
pub const COEFF_LIN_CONSTR_INDEX: usize = 0;

/// Evaluator for boolean constraints in MiniZinc models, providing functional evaluators for various boolean operations.
/// This struct stores arrays, an argument extractor, and a verbosity flag, and provides methods to generate functional evaluators for boolean constraints such as AND, OR, XOR, NOT, and reified versions.
#[derive(Debug, Clone, Default)]
pub struct BoolEvaluator {
    /// Map of array identifiers to their values.
    arrays: HashMap<String, Array>,
    /// Map of variable identifiers to their register values.
    variable_register_map: HashMap<String, Register>,
    /// Helper for extracting boolean arguments from constraints.
    args_extractor: BoolArgsExtractor,
    /// If true, enables verbose logging of constraint violations.
    verbose: bool,
}

impl BoolEvaluator {
    /// Creates a new `BoolFunctionalEvaluator` with the provided arrays and verbosity flag.
    ///
    /// # Arguments
    /// * `arrays` - Map of array identifiers to their values.
    /// * `variable_register_map` - Map of variable identifiers to their register values.
    /// * `verbose` - If true, enables verbose logging of constraint violations.
    ///
    /// # Returns
    /// A new `BoolFunctionalEvaluator` instance.
    pub fn new(
        arrays: HashMap<String, Array>,
        variable_register_map: HashMap<String, Register>,
        verbose: bool,
    ) -> Self {
        let args_extractor = BoolArgsExtractor::new();
        Self {
            arrays,
            variable_register_map,
            args_extractor,
            verbose,
        }
    }

    /// Returns a functional evaluator for the `array_bool_and` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    ///
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn array_bool_and(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
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
        let verbose = self.verbose;

        Box::new(move |solution: &[VariableValue]| {
            let r_value;
            let mut array_values = Vec::with_capacity(vars_register.len());
            for (_, var_register) in vars_register.iter().enumerate() {
                let value = solution[*var_register as usize].as_bool();
                array_values.push(value);
            }

            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM");
            }

            let mut violation = 0.0;
            if array_values.iter().all(|&item| item) != r_value {
                if verbose {
                    let joined = array_values
                        .iter()
                        .map(|b| if *b { "true" } else { "false" })
                        .collect::<Vec<_>>()
                        .join(" /\\ ");
                    info!("Violated: array_bool_and {} <-> {}", joined, r_value);
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `array_bool_element` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    ///
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn array_bool_element(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
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
        let value_register = self
            .variable_register_map
            .get(vars_involved.get(&C_TERM_INDEX).unwrap())
            .copied()
            .expect("Value register not found");
        let verbose = self.verbose;

        Box::new(move |solution: &[VariableValue]| {
            let array_value = array[solution[index_register as usize].as_int() as usize];
            let value = solution[value_register as usize].as_bool();

            let violation = (value as i32 - array_value as i32).abs() as f64;

            if violation > 0.0 && verbose {
                info!(
                    "Violated constraint: array_bool_element {} = {}",
                    array_value, value
                );
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `array_var_bool_element` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn array_var_bool_element(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
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
        let value_register = self
            .variable_register_map
            .get(vars_involved.get(&C_TERM_INDEX).unwrap())
            .copied()
            .expect("Value register not found");
        let verbose = self.verbose;
        let variable_map = self.variable_register_map.clone();

        Box::new(move |solution: &[VariableValue]| {
            let idx_in_array = solution[index_register as usize].as_int() as usize;
            let var_name = &array[idx_in_array];
            let var_idx = variable_map
                .get(var_name)
                .copied()
                .expect("Array value not found") as usize;
            let array_value = solution[var_idx].as_bool();
            let value = solution[value_register as usize].as_bool();

            let violation = (value as i32 - array_value as i32).abs() as f64;

            if violation > 0.0 && verbose {
                info!(
                    "Violated constraint: array_var_bool_element {} = {}",
                    array_value, value
                );
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `array_bool_xor` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    ///
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn array_bool_xor(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
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
        let verbose = self.verbose;

        Box::new(move |solution: &[VariableValue]| {
            let mut array_values = Vec::with_capacity(vars_register.len());
            for (_, var_register) in vars_register.iter().enumerate() {
                let value = solution[*var_register as usize].as_bool();
                array_values.push(value);
            }

            let mut violation = 0.0;
            if !array_values.iter().fold(false, |acc, &item| acc ^ item) {
                if verbose {
                    let joined = array_values
                        .iter()
                        .map(|b| if *b { "true" } else { "false" })
                        .collect::<Vec<_>>()
                        .join(" xor ");
                    info!("Violated constraint: array_bool_xor {}", joined);
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_and` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    ///
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool_and(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
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

        let mut c_register = None;
        let mut c_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            c_register = self
                .variable_register_map
                .get(vars_involved.get(&C_TERM_INDEX).unwrap())
                .copied();
        } else {
            c_const = Some(
                self.args_extractor
                    .extract_bool_value(C_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_bool();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_bool();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let mut violation = 0.0;
            if c_value != (a_value && b_value) {
                if verbose {
                    info!(
                        "Violated constraint: array_bool_and {} <-> {} /\\ {}",
                        c_value, a_value, b_value
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_clause` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    ///
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool_clause(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let as_array: Vec<String> = self.args_extractor.extract_bool_array(
            AS_ARRAY_INDEX as usize,
            &constraint.call.args,
            &self.arrays,
        );
        let mut as_vars_register = Vec::with_capacity(as_array.len());
        for var_name in &as_array {
            let var_register = self
                .variable_register_map
                .get(var_name)
                .copied()
                .expect("Array value not found in variable map");
            as_vars_register.push(var_register);
        }

        let bs_array: Vec<String> = self.args_extractor.extract_bool_array(
            BS_ARRAY_INDEX as usize,
            &constraint.call.args,
            &self.arrays,
        );
        let mut bs_vars_register = Vec::with_capacity(bs_array.len());
        for var_name in &bs_array {
            let var_register = self
                .variable_register_map
                .get(var_name)
                .copied()
                .expect("Array value not found in variable map");
            bs_vars_register.push(var_register);
        }
        let verbose = self.verbose;

        Box::new(move |solution: &[VariableValue]| {
            let mut violation = 0.0;

            let mut as_values = Vec::with_capacity(as_vars_register.len());
            for (_, var_register) in as_vars_register.iter().enumerate() {
                let value = solution[*var_register as usize].as_bool();
                as_values.push(value);
            }
            let mut bs_values = Vec::with_capacity(bs_vars_register.len());
            for (_, var_register) in bs_vars_register.iter().enumerate() {
                let value = solution[*var_register as usize].as_bool();
                bs_values.push(value);
            }

            let or_as_array = as_values.iter().map(|&b| b as i64).sum::<i64>();
            let or_bs_array = bs_values.iter().map(|&b| b as i64).sum::<i64>();

            let result = or_as_array + or_bs_array;

            if result == 0 {
                if verbose {
                    let mut interleaved: Vec<&str> = Vec::new();
                    let max_len = std::cmp::max(as_values.len(), bs_values.len());
                    for i in 0..max_len {
                        if let Some(a) = as_values.get(i) {
                            interleaved.push(if *a { "true" } else { "false" });
                        }
                        if let Some(b) = bs_values.get(i) {
                            interleaved.push(if *b { "not(true)" } else { "not(false)" });
                        }
                    }
                    let joined = interleaved.join(r" \/ ");
                    info!("Violated constraint: bool_clause {}", joined);
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_eq` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    ///
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool_eq(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
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

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_bool();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            if a_value != b_value {
                if verbose {
                    info!("Violated constraint: bool_eq {} = {}", a_value, b_value);
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool_eq_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
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

        let mut c_register = None;
        let mut c_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            c_register = self
                .variable_register_map
                .get(vars_involved.get(&C_TERM_INDEX).unwrap())
                .copied();
        } else {
            c_const = Some(
                self.args_extractor
                    .extract_bool_value(C_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_bool();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_bool();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let mut violation = 0.0;
            if c_value != (a_value == b_value) {
                if verbose {
                    info!(
                        "Violated constraint: bool_eq_reif {} <-> {} = {}",
                        c_value, a_value, b_value
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_le` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool_le(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
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

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_bool();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            if a_value > b_value {
                if verbose {
                    info!("Violated constraint: bool_le {} <= {}", a_value, b_value);
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool_le_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
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

        let mut c_register = None;
        let mut c_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            c_register = self
                .variable_register_map
                .get(vars_involved.get(&C_TERM_INDEX).unwrap())
                .copied();
        } else {
            c_const = Some(
                self.args_extractor
                    .extract_bool_value(C_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_bool();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_bool();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let mut violation = 0.0;
            if c_value != (a_value <= b_value) {
                if verbose {
                    info!(
                        "Violated constraint: bool_le_reif {} <-> {} <= {}",
                        c_value, a_value, b_value
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_lin_eq` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool_lin_eq(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let as_array: Vec<String> = self.args_extractor.extract_bool_array(
            AS_ARRAY_INDEX as usize,
            &constraint.call.args,
            &self.arrays,
        );
        let mut as_vars_register = Vec::with_capacity(as_array.len());
        for var_name in &as_array {
            let var_register = self
                .variable_register_map
                .get(var_name)
                .copied()
                .expect("Array value not found in variable map");
            as_vars_register.push(var_register);
        }

        let bs_array: Vec<String> = self.args_extractor.extract_bool_array(
            BS_ARRAY_INDEX as usize,
            &constraint.call.args,
            &self.arrays,
        );
        let mut bs_vars_register = Vec::with_capacity(bs_array.len());
        for var_name in &bs_array {
            let var_register = self
                .variable_register_map
                .get(var_name)
                .copied()
                .expect("Array value not found in variable map");
            bs_vars_register.push(var_register);
        }
        let verbose = self.verbose;
        let constant_term: i64 = self
            .args_extractor
            .extract_int_value(C_TERM_INDEX, &constraint.call);

        Box::new(move |solution: &[VariableValue]| {
            let mut violation = 0.0;

            let mut as_values = Vec::with_capacity(as_vars_register.len());
            for (_, var_register) in as_vars_register.iter().enumerate() {
                let value = solution[*var_register as usize].as_bool();
                as_values.push(value);
            }
            let mut bs_values = Vec::with_capacity(bs_vars_register.len());
            for (_, var_register) in bs_vars_register.iter().enumerate() {
                let value = solution[*var_register as usize].as_bool();
                bs_values.push(value);
            }

            let left_side_term: i64 = as_values
                .iter()
                .zip(bs_values.iter())
                .map(|(a, b)| {
                    let a_val = if *a { 1_i64 } else { 0_i64 };
                    let b_val = if *b { 1_i64 } else { 0_i64 };
                    a_val * b_val
                })
                .sum();

            if left_side_term != constant_term {
                if verbose {
                    info!(
                        "Violated constraint: bool_lin_eq {} = {}",
                        left_side_term, constant_term
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_lin_le` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool_lin_le(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let as_array: Vec<String> = self.args_extractor.extract_bool_array(
            AS_ARRAY_INDEX as usize,
            &constraint.call.args,
            &self.arrays,
        );
        let mut as_vars_register = Vec::with_capacity(as_array.len());
        for var_name in &as_array {
            let var_register = self
                .variable_register_map
                .get(var_name)
                .copied()
                .expect("Array value not found in variable map");
            as_vars_register.push(var_register);
        }

        let bs_array: Vec<String> = self.args_extractor.extract_bool_array(
            BS_ARRAY_INDEX as usize,
            &constraint.call.args,
            &self.arrays,
        );
        let mut bs_vars_register = Vec::with_capacity(bs_array.len());
        for var_name in &bs_array {
            let var_register = self
                .variable_register_map
                .get(var_name)
                .copied()
                .expect("Array value not found in variable map");
            bs_vars_register.push(var_register);
        }
        let verbose = self.verbose;
        let constant_term: i64 = self
            .args_extractor
            .extract_int_value(C_TERM_INDEX, &constraint.call);

        Box::new(move |solution: &[VariableValue]| {
            let mut violation = 0.0;

            let mut as_values = Vec::with_capacity(as_vars_register.len());
            for (_, var_register) in as_vars_register.iter().enumerate() {
                let value = solution[*var_register as usize].as_bool();
                as_values.push(value);
            }
            let mut bs_values = Vec::with_capacity(bs_vars_register.len());
            for (_, var_register) in bs_vars_register.iter().enumerate() {
                let value = solution[*var_register as usize].as_bool();
                bs_values.push(value);
            }

            let left_side_term: i64 = as_values
                .iter()
                .zip(bs_values.iter())
                .map(|(a, b)| {
                    let a_val = if *a { 1_i64 } else { 0_i64 };
                    let b_val = if *b { 1_i64 } else { 0_i64 };
                    a_val * b_val
                })
                .sum();

            if left_side_term > constant_term {
                if verbose {
                    info!(
                        "Violated constraint: bool_lin_le {} <= {}",
                        left_side_term, constant_term
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_lt` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool_lt(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
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

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_bool();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            if a_value >= b_value {
                if verbose {
                    info!("Violated constraint: bool_lt {} < {}", a_value, b_value);
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_lt_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool_lt_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
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

        let mut c_register = None;
        let mut c_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            c_register = self
                .variable_register_map
                .get(vars_involved.get(&C_TERM_INDEX).unwrap())
                .copied();
        } else {
            c_const = Some(
                self.args_extractor
                    .extract_bool_value(C_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_bool();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_bool();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let mut violation = 0.0;
            if c_value != (a_value < b_value) {
                if verbose {
                    info!(
                        "Violated constraint: bool_lt_reif {} <-> {} < {}",
                        c_value, a_value, b_value
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_not` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool_not(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
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

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_bool();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            if a_value == b_value {
                if verbose {
                    info!(
                        "Violated constraint: bool_not {} = not({})",
                        a_value, b_value
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_or` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool_or(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
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

        let mut c_register = None;
        let mut c_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            c_register = self
                .variable_register_map
                .get(vars_involved.get(&C_TERM_INDEX).unwrap())
                .copied();
        } else {
            c_const = Some(
                self.args_extractor
                    .extract_bool_value(C_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_bool();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_bool();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let mut violation = 0.0;
            if c_value != (a_value || b_value) {
                if verbose {
                    info!(
                        r"Violated constraint: bool_or {} <-> {} \/ {}",
                        c_value, a_value, b_value
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_xor` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool_xor(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
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

        let mut c_register = None;
        let mut c_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            c_register = self
                .variable_register_map
                .get(vars_involved.get(&C_TERM_INDEX).unwrap())
                .copied();
        } else {
            c_const = Some(
                self.args_extractor
                    .extract_bool_value(C_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_bool();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_bool();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let mut violation = 0.0;
            if c_value != (a_value ^ b_value) {
                if verbose {
                    info!(
                        "Violated constraint: bool_xor {} <-> {} xor {}",
                        c_value, a_value, b_value
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `bool_xor` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn bool2int(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
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
                    .extract_int_value(B_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_bool();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            if a_value as i64 != b_value {
                if verbose {
                    info!("Violated constraint: bool2int {} = {}", a_value, b_value);
                }
                violation = 1.0;
            }

            violation
        })
    }
}

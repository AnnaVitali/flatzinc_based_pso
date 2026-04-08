use crate::args_extractor::sub_types::int_args_extractor::IntArgsExtractor;
use crate::data_utility::logger::write_verbose_output;
use crate::data_utility::types::Register;
use crate::data_utility::types::VariableValue;
use crate::evaluator::mini_evaluator::CallWithDefines;
use flatzinc_serde::{Array, Literal};
use log::info;
use std::collections::HashMap;

pub const A_TERM_INDEX: i64 = 0;
pub const B_TERM_INDEX: i64 = 1;
pub const C_TERM_INDEX: i64 = 2;
pub const R_TERM_INDEX: i64 = 2;
pub const R_TERM_LIN_EXPR_REIF_INDEX: i64 = 3;
pub const COEFF_LIN_CONSTR_INDEX: i64 = 0;
pub const VARS_LIN_CONSTR_INDEX: i64 = 1;
pub const CONST_LIN_CONSTR_INDEX: i64 = 2;

#[derive(Debug, Clone, Default)]
/// Evaluator for integer constraints, providing methods to evaluate various integer operations and constraints.
///
/// This struct contains methods for evaluating integer constraints such as equality, inequality, linear expressions,
/// and arithmetic operations. It uses argument extraction utilities and supports verbose output for debugging.
pub struct IntEvaluator {
    /// A map of identifiers to arrays used in constraint evaluation.
    arrays: HashMap<String, Array>,
    /// A map of variable identifiers to their corresponding registers, used for resolving variable references in constraints.
    variable_register_map: HashMap<String, Register>,
    /// An extractor for integer arguments from constraints.
    args_extractor: IntArgsExtractor,
    /// A flag to enable verbose output for debugging purposes.
    verbose: bool,
}

impl IntEvaluator {
    /// Creates a new `IntFunctionalEvaluator` instance.
    ///
    /// # Arguments
    /// * `arrays` - A map of identifiers to arrays used in constraint evaluation.
    /// * `variable_register_map` - A map of variable identifiers to their corresponding registers.
    /// * `verbose` - Boolean flag to enable verbose output for debugging.
    ///
    /// # Returns
    /// A new `IntFunctionalEvaluator` instance.
    pub fn new(
        arrays: HashMap<String, Array>,
        variable_register_map: HashMap<String, Register>,
        verbose: bool,
    ) -> Self {
        let args_extractor = IntArgsExtractor::new();
        Self {
            arrays,
            variable_register_map,
            args_extractor,
            verbose,
        }
    }

    /// Returns a functional evaluator for the `array_int_element` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn array_int_element(
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
        let value_register = self
            .variable_register_map
            .get(vars_involved.get(&C_TERM_INDEX).unwrap())
            .copied()
            .expect("Value register not found");
        let verbose = self.verbose;

        Box::new(move |solution: &[VariableValue]| {
            let array_value = array[solution[index_register as usize].as_int() as usize];
            let value = solution[value_register as usize].as_int();

            let violation = (value - array_value).abs() as f64;

            if violation > 0.0 && verbose {
                info!(
                    "Violated constraint: array_int_element {} = {}",
                    array_value, value
                );
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `array_var_int_element` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn array_var_int_element(
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
            .expect("Expect a variable array for array_var_int_element constraint")
            .contents
            .iter()
            .map(|elem| match elem {
                Literal::Identifier(i) => i.clone(),
                _ => panic!("Expected identifier in array for array_var_int_element constraint"),
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
            let array_value = solution[var_idx].as_int();
            let value = solution[value_register as usize].as_int();

            let violation = (value - array_value).abs() as f64;

            if violation > 0.0 && verbose {
                info!(
                    "Violated constraint: array_var_int_element {} = {}",
                    array_value, value
                );
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `int_abs` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_abs(
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

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            if b_value != a_value.abs() {
                if verbose {
                    info!(
                        "Violated constraint: int_abs abs({}) = {}",
                        a_value, b_value
                    );
                }
                violation = ((b_value - a_value.abs()).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_div` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_div(
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
                    .extract_int_value(C_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_int();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let mut violation = 0.0;
            if b_value == 0 {
                violation = 1.0;
            } else {
                let result = a_value / b_value;
                if c_value != result {
                    if verbose {
                        info!(
                            "Violated constraint: int_div {}/{} = {}",
                            a_value, b_value, c_value
                        );
                    }
                    violation = ((c_value - result).abs()) as f64;
                }
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `int_eq` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_eq(
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

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            let mut violation = 0.0;
            if a_value != b_value {
                if verbose {
                    info!("Violated constraint: int_eq {} = {}", a_value, b_value);
                }
                violation = ((a_value - b_value).abs()) as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `int_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_eq_reif(
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

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let r_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM");
            }

            let mut violation = 0.0;
            let eq_res = a_value == b_value;
            if r_value != eq_res {
                if verbose {
                    info!(
                        "Violated constraint: int_eq_reif {} <-> {} = {}",
                        a_value, b_value, r_value
                    );
                }
                violation = ((a_value - b_value).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_le` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_le(
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

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            let mut violation = 0.0;
            if a_value > b_value {
                if verbose {
                    info!("Violated constraint: int_le {} <= {}", a_value, b_value);
                }
                violation = ((a_value - b_value).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_le_reif(
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

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let r_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM");
            }

            let mut violation = 0.0;
            if r_value != (a_value <= b_value) {
                if verbose {
                    info!(
                        "Violated constraint: int_le_reif {} <-> {} <= {}",
                        r_value, a_value, b_value
                    );
                }
                violation = 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_lin_eq` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_lin_eq(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
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
        let constant_term: i64 = self
            .args_extractor
            .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term =
                Self::int_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);

            let mut violation = 0.0;
            if left_side_term != constant_term {
                if verbose {
                    info!(
                        "Violated constraint: int_lin_eq {} = {}",
                        left_side_term, constant_term
                    );
                }
                violation = ((left_side_term - constant_term).abs()) as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `int_lin_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_lin_eq_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
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
        let literal_vars_map = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut registers = Vec::new();
        for var in &vars_involved {
            let reg = self
                .variable_register_map
                .get(var)
                .copied()
                .expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let constant_term: i64 = self
            .args_extractor
            .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call);
        let mut r_register = None;
        let mut r_const = None;
        if literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).is_some() {
            r_register = self
                .variable_register_map
                .get(literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term =
                Self::int_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);
            let mut violation = 0.0;
            let r_value;
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM in int_lin_eq_reif");
            }

            if r_value != (left_side_term == constant_term) {
                if verbose {
                    info!(
                        "Violated constraint: int_lin_eq_reif {} <-> {} = {}",
                        r_value, left_side_term, constant_term
                    );
                }
                violation = 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_lin_le` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_lin_le(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
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
        let constant_term: i64 = self
            .args_extractor
            .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term =
                Self::int_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);
            let mut violation = 0.0;

            if left_side_term > constant_term {
                if verbose {
                    info!(
                        "Violated constraint: int_lin_le {} <= {}",
                        left_side_term, constant_term
                    );
                }
                violation = ((left_side_term - constant_term).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_lin_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_lin_le_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
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
        let literal_vars_map = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut registers = Vec::new();
        for var in &vars_involved {
            let reg = self
                .variable_register_map
                .get(var)
                .copied()
                .expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let constant_term: i64 = self
            .args_extractor
            .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call);
        let mut r_register = None;
        let mut r_const = None;
        if literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).is_some() {
            r_register = self
                .variable_register_map
                .get(literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term =
                Self::int_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);
            let r_value;
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM in int_lin_eq_reif");
            }

            let mut violation = 0.0;
            if r_value != (left_side_term <= constant_term) {
                if verbose {
                    info!(
                        "Violated constraint: int_lin_le_reif {} <-> {} <= {}",
                        r_value, left_side_term, constant_term
                    );
                }
                violation = 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_lin_ne` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_lin_ne(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
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
        let constant_term: i64 = self
            .args_extractor
            .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term =
                Self::int_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);

            let mut violation = 0.0;
            if left_side_term == constant_term {
                if verbose {
                    info!(
                        "Violated constraint: int_lin_ne {} == {}",
                        left_side_term, constant_term
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `int_lin_ne_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_lin_ne_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
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
        let literal_vars_map = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let mut registers = Vec::new();
        for var in &vars_involved {
            let reg = self
                .variable_register_map
                .get(var)
                .copied()
                .expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let constant_term: i64 = self
            .args_extractor
            .extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call);
        let mut r_register = None;
        let mut r_const = None;
        if literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).is_some() {
            r_register = self
                .variable_register_map
                .get(literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).unwrap())
                .copied();
        } else {
            r_const = Some(
                self.args_extractor
                    .extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term =
                Self::int_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);
            let r_value;
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM in int_lin_eq_reif");
            }

            let mut violation = 0.0;
            if r_value != (left_side_term != constant_term) {
                if verbose {
                    info!(
                        "Violated constraint: int_lin_ne_reif {} <-> {} != {}",
                        r_value, left_side_term, constant_term
                    );
                }
                violation = 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_lt` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_lt(
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

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            if a_value >= b_value {
                if verbose {
                    info!("Violated constraint: int_lt {} < {}", a_value, b_value);
                }
                violation = (a_value - b_value + 1) as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `int_lt_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_lt_reif(
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

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let r_value = if r_register.is_some() {
                solution[r_register.unwrap() as usize].as_bool()
            } else {
                r_const.expect("Expected constant value for R_TERM")
            };
            if r_value != (a_value < b_value) {
                if verbose {
                    info!(
                        "Violated constraint: int_lt_reif {} <-> {} < {}",
                        r_value, a_value, b_value
                    );
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `int_max` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_max(
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
                    .extract_int_value(C_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_int();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let max_val = a_value.max(b_value);
            let mut violation = 0.0;
            if c_value != max_val {
                if verbose {
                    info!(
                        "Violated constraint: int_max max({},{}) = {}",
                        a_value, b_value, c_value
                    );
                }
                violation = ((c_value - max_val).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_min` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_min(
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
                    .extract_int_value(C_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_int();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let min_val = a_value.min(b_value);
            let mut violation = 0.0;
            if c_value != min_val {
                if verbose {
                    info!(
                        "Violated constraint: int_min min({},{}) = {}",
                        a_value, b_value, c_value
                    );
                }
                violation = ((c_value - min_val).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_mod` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_mod(
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
                    .extract_int_value(C_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_int();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let real_mod = a_value % b_value;
            let mut violation = 0.0;
            if c_value != real_mod {
                if verbose {
                    info!(
                        "Violated constraint: int_mod {} mod {} = {}",
                        a_value, b_value, c_value
                    );
                }
                violation = ((c_value - real_mod).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_ne` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_ne(
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

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            if a_value == b_value {
                if verbose {
                    info!("Violated constraint: int_ne {} != {}", a_value, b_value);
                }
                violation = 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_ne_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_ne_reif(
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

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let r_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM");
            }

            let mut violation = 0.0;
            if r_value != (a_value != b_value) {
                if verbose {
                    info!(
                        "Violated constraint: int_ne_reif {} <-> {} != {}",
                        r_value, a_value, b_value
                    );
                }
                violation = 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_pow` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_pow(
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
                    .extract_int_value(C_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_int();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let result = a_value.pow(b_value as u32);
            let mut violation = 0.0;
            if c_value != result {
                if verbose {
                    info!(
                        "Violated constraint: int_pow {} ^ {} = {}",
                        a_value, b_value, c_value
                    );
                }
                violation = ((c_value - result).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_times` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_times(
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
                    .extract_int_value(C_TERM_INDEX, &constraint.call),
            );
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some() {
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_int();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_int();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let result = a_value * b_value;
            let mut violation = 0.0;
            if c_value != result {
                if verbose {
                    info!(
                        "Violated constraint: int_times {} * {} = {}",
                        a_value, b_value, c_value
                    );
                }
                violation = ((c_value - result).abs()) as f64;
            }
            violation
        })
    }

    fn int_lin_left_term(
        verbose: bool,
        coeff: &Vec<i64>,
        registers: &Vec<u32>,
        solution: &[VariableValue],
        verbose_terms: &mut String,
    ) -> i64 {
        let left_side_term: i64 = coeff
            .iter()
            .zip(registers.iter())
            .map(|(c, id)| {
                if verbose {
                    write_verbose_output(verbose_terms, c, &solution[*id as usize].as_int());
                }
                c * solution[*id as usize].as_int()
            })
            .sum();
        left_side_term
    }
}

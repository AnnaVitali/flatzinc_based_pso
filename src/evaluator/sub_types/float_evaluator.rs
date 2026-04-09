use crate::data_utility::types::Register;
use crate::{args_extractor::sub_types::float_args_extractor::FloatArgsExtractor, data_utility::logger::write_verbose_output};
use crate::data_utility::types::VariableValue;
use flatzinc_serde::{Array, Literal};
use log::info;
use std::collections::HashMap;
use crate::evaluator::mini_evaluator::CallWithDefines;

pub const A_TERM_INDEX: i64 = 0;
pub const B_TERM_INDEX: i64 = 1;
pub const C_TERM_INDEX: i64 = 2;
pub const R_TERM_INDEX: i64 = 2;
pub const R_TERM_LIN_EXPR_REIF_INDEX: i64 = 3;
pub const COEFF_LIN_CONSTR_INDEX: i64 = 0;
pub const VARS_LIN_CONSTR_INDEX: i64 = 1;
pub const CONST_LIN_CONSTR_INDEX: i64 = 2;
pub const FLOAT_EQ_TOLERANCE: f64 = 0.0;

/// Evaluator for float constraints in MiniZinc models, providing functional evaluators for various float operations.
///
/// This struct stores arrays, an argument extractor, and a verbosity flag, and provides methods to generate functional evaluators for float constraints such as arithmetic, trigonometric, and comparison operations.
#[derive(Debug, Clone, Default)]
pub struct FloatEvaluator {
    /// Map of array identifiers to their values.
    arrays: HashMap<String, Array>,
    /// Map of variable identifiers to their registers, used for resolving variable references in constraints.
    variable_register_map: HashMap<String, Register>,
    /// Helper for extracting float arguments from constraints.
    args_extractor: FloatArgsExtractor,
    /// If true, enables verbose logging of constraint violations.
    verbose: bool,
}


impl FloatEvaluator {
    /// Creates a new `FloatFunctionalEvaluator` with the provided arrays and verbosity flag.
    ///
    /// # Arguments
    /// * `arrays` - Map of array identifiers to their values.
    /// * `variable_register_map` - Map of variable identifiers to their registers.
    /// * `verbose` - If true, enables verbose logging of constraint violations.
    ///
    /// # Returns
    /// A new `FloatFunctionalEvaluator` instance.
    pub fn new(arrays: HashMap<String, Array>, variable_register_map: HashMap<String, Register>, verbose: bool) -> Self {
        let args_extractor = FloatArgsExtractor::new();
        Self {
            arrays,
            variable_register_map,
            args_extractor,
            verbose,
        }
    }

    /// Returns a functional evaluator for the `array_float_element` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    ///
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn array_float_element(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let index_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied().expect("Index register not found");
        let array: Vec<f64> = self.arrays.get(vars_involved.get(&B_TERM_INDEX).unwrap()).expect("Expect a constant array for array_float_element constraint")
            .contents
            .iter()
            .map(|elem| match elem {
                Literal::Float(f) => *f,
                _ => panic!("Expected float literal in array for array_float_element constraint"),
            })
            .collect();
        let value_register = self.variable_register_map.get(vars_involved.get(&C_TERM_INDEX).unwrap()).copied().expect("Value register not found");
        let verbose = self.verbose;

        Box::new(move |solution: &[VariableValue]| {
            let array_value = array[solution[index_register as usize].as_int() as usize];
            let value = solution[value_register as usize].as_float();

            let violation =  (value - array_value).abs() as f64;

            if violation > 0.0 && verbose {
                info!("Violated constraint: array_float_element {} = {}", array_value, value);
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `array_var_float_element` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn array_var_float_element(
        &self,
        constraint: &CallWithDefines,
     ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let index_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied().expect("Index register not found");
        let array: Vec<String> = self.arrays.get(vars_involved.get(&B_TERM_INDEX).unwrap()).expect("Expect a variable array for array_var_float_element constraint")
            .contents
            .iter()
            .map(|elem| match elem {
                Literal::Identifier(i) => i.clone(),
                _ => panic!("Expected identifier in array for array_var_float_element constraint"),
            })
            .collect();
        let value_register = self.variable_register_map.get(vars_involved.get(&C_TERM_INDEX).unwrap()).copied().expect("Value register not found");
        let verbose = self.verbose;
        let variable_map = self.variable_register_map.clone();

        Box::new(move |solution: &[VariableValue]| {
            let idx_in_array = solution[index_register as usize].as_int() as usize;
            let var_name = &array[idx_in_array];
            let var_idx = variable_map.get(var_name).copied().expect("Array value not found") as usize;
            let array_value = solution[var_idx].as_float();
            let value = solution[value_register as usize].as_float();

            let violation =  (value - array_value).abs() as f64;

            if violation > 0.0 && verbose {
                info!("Violated constraint: array_var_float_element {} = {}", array_value, value);
            }

            violation
        })
    }


    /// Returns a functional evaluator for the `float_abs` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    ///
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_abs(
        &self,
        constraint: &CallWithDefines,
     ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            if (a_value.abs() - b_value).abs() > FLOAT_EQ_TOLERANCE {
                if verbose {
                    info!("Violated constraint: float_abs {} = {}", a_value.abs(), b_value);
                }
                violation = (a_value.abs() - b_value).abs();
            }
            violation
        })
    }
    

    /// Returns a functional evaluator for the `float_acos` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    ///
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_acos(
        &self,
        constraint: &CallWithDefines,
       ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let acos = a_value.acos();
            if !((acos - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_acos {} = {}", a_value, b_value);
                }
                violation = (acos - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_acosh` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_acosh(
        &self,
        constraint: &CallWithDefines,
   ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            let mut violation = 0.0;
            let acosh = a_value.acosh();
            if !((acosh - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_acosh {} = {}", a_value, b_value);
                }
                violation = (acosh - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_asin` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_asin(
        &self,
        constraint: &CallWithDefines,
   ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let asin = a_value.asin();
            if !((asin - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_asin {} = {}", a_value, b_value);
                }
                violation = (asin - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_asinh` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_asinh(
        &self,
        constraint: &CallWithDefines,
       ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let asinh = a_value.asinh();
            if !((asinh - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_asinh {} = {}", asinh, b_value);
                }
                violation = (asinh - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_atan` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_atan(
        &self,
        constraint: &CallWithDefines,
       ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let atan = a_value.atan();
            if !((atan - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_atan {} = {}", atan, b_value);
                }
                violation = (atan - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_atanh` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_atanh(
        &self,
        constraint: &CallWithDefines,
       ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            if a_value.abs() >= 1.0 || b_value.abs() >= 1.0 {
                if verbose {
                    info!("Invalid input for float_atanh {} or {} outside (-1,1)", a_value, b_value);
                }
                violation = 1.0;
            } else {
                let atanh = a_value.atanh();
                if !((atanh - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                    if verbose {
                        info!("Violated constraint: float_atanh {} = {}", atanh, b_value);
                    }
                    violation = (atanh - b_value).abs();
                }
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_cos` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_cos(
        &self,
        constraint: &CallWithDefines,
        ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let cos = a_value.cos();
            if !((cos - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                    if verbose {
                    info!("Violated constraint: float_cos {} = {}", cos, b_value);
                }
                violation = (cos - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_cosh` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_cosh(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let cosh = a_value.cosh();
                if !((cosh - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_cosh {} = {}", cosh, b_value);
                }
                violation = (cosh - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_cos` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_div(
        &self,
        constraint: &CallWithDefines,
) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
                let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else{
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
    
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else{
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }
        
        let mut c_register = None;
        let mut c_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            c_register = self.variable_register_map.get(vars_involved.get(&C_TERM_INDEX).unwrap()).copied();
        }else{
            c_const = Some(self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call));
        }


        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_float();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let mut violation = 0.0;
            if b_value == 0.0 {
                violation = 1.0;
            } else {
                let result = a_value / b_value;
                if !((c_value - result).abs() <= FLOAT_EQ_TOLERANCE) {
                    if verbose {
                        info!("Violated constraint: float_div {}/{} = {}", a_value, b_value, c_value);
                    }
                    violation = ((c_value - result).abs()) as f64;
                }
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `float_eq` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_eq(
        &self,
        constraint: &CallWithDefines,
   ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else if b_register.is_none() {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            let mut violation = 0.0;
            if !((a_value - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_eq {} = {}", a_value, b_value);
                }
                violation = ((a_value - b_value).abs()) as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `float_eq_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_eq_reif(
        &self,
        constraint: &CallWithDefines,
     ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else if b_register.is_none() {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self.variable_register_map.get(vars_involved.get(&R_TERM_INDEX).unwrap()).copied();
        }else {
            r_const = Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let r_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
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
                    info!("Violated constraint: float_eq_reif {} <-> {} = {}", a_value, b_value, r_value);
                }
                violation = ((a_value - b_value).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_exp` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_exp(
        &self,
        constraint: &CallWithDefines,
        ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let exp = a_value.exp();
            if !((exp - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                   
                    info!("Violated constraint: float_exp {} = {}", exp, b_value);
                }
                violation = (exp - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_atanh` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_le(
        &self,
        constraint: &CallWithDefines,
        ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else if b_register.is_none() {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            let mut violation = 0.0;
            if a_value > b_value {
                if verbose {
                    info!("Violated constraint: float_le {} <= {}", a_value, b_value);
                }
                violation = ((a_value - b_value).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_le_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_le_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else if b_register.is_none() {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self.variable_register_map.get(vars_involved.get(&R_TERM_INDEX).unwrap()).copied();
        }else {
            r_const = Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let r_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
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
                    info!("Violated constraint: float_le_reif {} <-> {} <= {}", r_value, a_value, b_value);
                }
                violation = 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_le_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_eq(
        &self,
        constraint: &CallWithDefines,
   ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let mut  registers = Vec::new();
        for var in &vars_involved {
            let reg = self.variable_register_map.get(var).copied().expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let constant_term: f64 = self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::float_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);

            let mut violation = 0.0;
            if !((left_side_term - constant_term).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_lin_eq {} = {}", left_side_term, constant_term);
                }
                violation = ((left_side_term - constant_term).abs()) as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `float_lin_eq_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_eq_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let literal_vars_map = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut  registers = Vec::new();
        for var in &vars_involved {
            let reg = self.variable_register_map.get(var).copied().expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let constant_term: f64 = self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call);
        let mut r_register = None;
        let mut r_const = None;
        if literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).is_some() {
            r_register = self.variable_register_map.get(literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).unwrap()).copied();
        }else {
            r_const = Some(self.args_extractor.extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call));
        }
        
        Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::float_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);
            let mut violation = 0.0;
            let r_value;
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM in float_lin_eq_reif");
            }

            if r_value != (left_side_term == constant_term) {
                if verbose {
                    info!("Violated constraint: float_lin_eq_reif {} <-> {} = {}", r_value, left_side_term, constant_term);
                }
                violation = 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_lin_eq` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_le(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let mut  registers = Vec::new();
        for var in &vars_involved {
            let reg = self.variable_register_map.get(var).copied().expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let constant_term: f64 = self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::float_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);
            let mut violation = 0.0;
            
            if left_side_term > constant_term {
                if verbose {

                    info!("Violated constraint: float_lin_le {} <= {}", left_side_term, constant_term);
                }
                violation = ((left_side_term - constant_term).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_le_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_le_reif(
        &self,
        constraint: &CallWithDefines,
        ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let literal_vars_map = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut  registers = Vec::new();
        for var in &vars_involved {
            let reg = self.variable_register_map.get(var).copied().expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let constant_term: f64 = self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call);
        let mut r_register = None;
        let mut r_const = None;
        if literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).is_some() {
            r_register = self.variable_register_map.get(literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).unwrap()).copied();
        }else {
            r_const = Some(self.args_extractor.extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call));
        }
        
        Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::float_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);
            let r_value;
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM in float_lin_eq_reif");
            }


            let mut violation = 0.0;
            if r_value != (left_side_term <= constant_term) {
                if verbose {
                    info!("Violated constraint: float_lin_le_reif {} <-> {} <= {}", r_value, left_side_term, constant_term);
                }
                violation = 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_lin_lt` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_lt(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let mut  registers = Vec::new();
        for var in &vars_involved {
            let reg = self.variable_register_map.get(var).copied().expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let constant_term: f64 = self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::float_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);
            let mut violation = 0.0;
            
            if left_side_term >= constant_term {
                if verbose {

                    info!("Violated constraint: float_lin_lt {} < {}", left_side_term, constant_term);
                }
                violation = ((left_side_term - constant_term + 1.0).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_lin_lt_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_lt_reif(
        &self,
        constraint: &CallWithDefines,
   ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let literal_vars_map = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut  registers = Vec::new();
        for var in &vars_involved {
            let reg = self.variable_register_map.get(var).copied().expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let constant_term: f64 = self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call);
        let mut r_register = None;
        let mut r_const = None;
        if literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).is_some() {
            r_register = self.variable_register_map.get(literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).unwrap()).copied();
        }else {
            r_const = Some(self.args_extractor.extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call));
        }
        
        Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::float_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);
            let r_value;
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM in float_lin_eq_reif");
            }


            let mut violation = 0.0;
            if r_value != (left_side_term < constant_term) {
                if verbose {
                    info!("Violated constraint: float_lin_lt_reif {} <-> {} < {}", r_value, left_side_term, constant_term);
                }
                violation = 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_lin_ne` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_ne(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let mut  registers = Vec::new();
        for var in &vars_involved {
            let reg = self.variable_register_map.get(var).copied().expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let constant_term: f64 = self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::float_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);

            let mut violation = 0.0;
            if left_side_term == constant_term {
                if verbose {
                    info!("Violated constraint: float_lin_ne {} == {}", left_side_term, constant_term);
                }
                violation = 1.0;
            } 

            violation
        })
    }

    /// Returns a functional evaluator for the `float_lin_ne_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_ne_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let literal_vars_map = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut  registers = Vec::new();
        for var in &vars_involved {
            let reg = self.variable_register_map.get(var).copied().expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let constant_term: f64 = self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call);
        let mut r_register = None;
        let mut r_const = None;
        if literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).is_some() {
            r_register = self.variable_register_map.get(literal_vars_map.get(&R_TERM_LIN_EXPR_REIF_INDEX).unwrap()).copied();
        }else {
            r_const = Some(self.args_extractor.extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call));
        }
        
       Box::new(move |solution: &[VariableValue]| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::float_lin_left_term(verbose, &coeff, &registers, solution, &mut verbose_terms);
            let r_value;
            if r_register.is_some() {
                r_value = solution[r_register.unwrap() as usize].as_bool();
            } else {
                r_value = r_const.expect("Expected constant value for R_TERM in float_lin_eq_reif");
            }


            let mut violation = 0.0;
            if r_value != (left_side_term != constant_term) {
                if verbose {
                    info!("Violated constraint: float_lin_ne_reif {} <-> {} != {}", r_value, left_side_term, constant_term);
                }
                violation = 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_ln` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_ln(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let ln = a_value.ln();
            if !((ln - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_ln {} = {}", ln, b_value);
                }
                violation = (ln - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_log10` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_log10(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let log10 = a_value.log10();
            if !((log10 - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_log10 {} = {}", log10, b_value);
                }
                violation = (log10 - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_log2` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_log2(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let log2 = a_value.log2();
            if !((log2 - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_log2 {} = {}", log2, b_value);
                }
                violation = (log2 - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_log2` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lt(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else if b_register.is_none() {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            if a_value >= b_value {
                if verbose {
                    info!("Violated constraint: float_lt {} < {}", a_value, b_value);
                }
                violation = (a_value - b_value + 1.0) as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `float_lt_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lt_reif(
        &self,
        constraint: &CallWithDefines,
       ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else if b_register.is_none() {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self.variable_register_map.get(vars_involved.get(&R_TERM_INDEX).unwrap()).copied();
        }else {
            r_const = Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
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
                    info!("Violated constraint: int_lt_reif {} <-> {} < {}", r_value, a_value, b_value);
                }
                violation = 1.0;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `float_max` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_max(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
                let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else{
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
    
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else{
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }
        
        let mut c_register = None;
        let mut c_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            c_register = self.variable_register_map.get(vars_involved.get(&C_TERM_INDEX).unwrap()).copied();
        }else{
            c_const = Some(self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call));
        }


        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_float();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let max_val = a_value.max(b_value);
            let mut violation = 0.0;
            if !((c_value - max_val).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_max max({},{}) = {}", a_value, b_value, c_value);
                }
                violation = ((c_value - max_val).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_min` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_min(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
                let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else{
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
    
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else{
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }
        
        let mut c_register = None;
        let mut c_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            c_register = self.variable_register_map.get(vars_involved.get(&C_TERM_INDEX).unwrap()).copied();
        }else{
            c_const = Some(self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call));
        }


        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_float();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let min_val = a_value.min(b_value);
            let mut violation = 0.0;
            if !((c_value - min_val).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_min min({},{}) = {}", a_value, b_value, c_value);
                }
                violation = ((c_value - min_val).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_ne` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn float_ne(
        &self,
        constraint: &CallWithDefines,
     ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else{
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
    
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else{
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            if a_value == b_value {
                if verbose {
                    info!("Violated constraint: float_ne {} != {}", a_value, b_value);
                }
                violation = 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_ne_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn float_ne_reif(
        &self,
        constraint: &CallWithDefines,
     ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else if b_register.is_none() {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        let mut r_register = None;
        let mut r_const = None;
        if vars_involved.get(&R_TERM_INDEX).is_some() {
            r_register = self.variable_register_map.get(vars_involved.get(&R_TERM_INDEX).unwrap()).copied();
        }else {
            r_const = Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let r_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
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
                    info!("Violated constraint: float_ne_reif {} <-> {} != {}", r_value, a_value, b_value);
                }
                violation = 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_plus` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_plus(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
                let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else{
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
    
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else{
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }
        
        let mut c_register = None;
        let mut c_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            c_register = self.variable_register_map.get(vars_involved.get(&C_TERM_INDEX).unwrap()).copied();
        }else{
            c_const = Some(self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call));
        }


        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_float();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let result = a_value + b_value;
            let mut violation = 0.0;
            if !((c_value - result).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_plus {} + {} = {}", a_value, b_value, c_value);
                }
                violation = ((c_value - result).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_pow` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_pow(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
                let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else{
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
    
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else{
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }
        
        let mut c_register = None;
        let mut c_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            c_register = self.variable_register_map.get(vars_involved.get(&C_TERM_INDEX).unwrap()).copied();
        }else{
            c_const = Some(self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call));
        }


        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_float();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let result = a_value.powf(b_value);
            let mut violation = 0.0;
            if !((c_value - result).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_pow {} ^ {} = {}", a_value, b_value, c_value);
                }
                violation = ((c_value - result).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_sin` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_sin(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let sin = a_value.sin();
                if !((sin - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_sin {} = {}", sin, b_value);
                }
                violation = (sin - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_sinh` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_sinh(
        &self,
        constraint: &CallWithDefines,
     ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let sinh = a_value.sinh();
                if !((sinh - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_sinh {} = {}", sinh, b_value);
                }
                violation = (sinh - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_sqrt` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_sqrt(
        &self,
        constraint: &CallWithDefines,
        ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let sqrt = a_value.sqrt();
                if !((sqrt - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_sqrt {} = {}", sqrt, b_value);
                }
                violation = (sqrt - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_tan` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_tan(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let tan = a_value.tan();
                if !((tan - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_tan {} = {}", tan, b_value);
                }
                violation = (tan - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_tanh` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_tanh(
        &self,
        constraint: &CallWithDefines,
     ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            let tanh = a_value.tanh();
                if !((tanh - b_value).abs() <= FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_tanh {} = {}", tanh, b_value);
                }
                violation = (tanh - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_times` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_times(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
                let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else{
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }
    
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else{
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }
        
        let mut c_register = None;
        let mut c_const = None;
        if vars_involved.get(&C_TERM_INDEX).is_some() {
            c_register = self.variable_register_map.get(vars_involved.get(&C_TERM_INDEX).unwrap()).copied();
        }else{
            c_const = Some(self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call));
        }


        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            let c_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if c_register.is_some() {
                c_value = solution[c_register.unwrap() as usize].as_float();
            } else {
                c_value = c_const.expect("Expected constant value for C_TERM");
            }

            let result = a_value * b_value;
            let mut violation = 0.0;
            if c_value != result {
                if verbose {
                    info!("Violated constraint: float_times {} * {} = {}", a_value, b_value, c_value);
                }
                violation = ((c_value - result).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int2float` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int2float(
        &self,
        constraint: &CallWithDefines,
         ) -> Box<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> {
               let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call));
        }
        let mut b_register = None;
        let mut b_const = None;
        if vars_involved.get(&B_TERM_INDEX).is_some() {
            b_register = self.variable_register_map.get(vars_involved.get(&B_TERM_INDEX).unwrap()).copied();
        }else {
            b_const = Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[VariableValue]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            let mut violation = 0.0;
            if !((a_value as f64 - b_value).abs() < FLOAT_EQ_TOLERANCE) {
                if verbose {
                    info!("Violated constraint: float_eq {} = {}", a_value, b_value);
                }
                violation = (a_value as f64 - b_value).abs();
            }
            violation
        })
    }

    fn float_lin_left_term(
        verbose: bool,
        coeff: &Vec<f64>,
        registers: &Vec<u32>,
        solution: &[VariableValue],
        verbose_terms: &mut String,
    ) -> f64 {
        let left_side_term: f64 = coeff
            .iter()
            .zip(registers.iter())
            .map(|(c, id)| {
                if verbose {
                    write_verbose_output(verbose_terms, c, &solution[*id as usize].as_float());
                }
                c * solution[*id as usize].as_float()
            })
            .sum();
        left_side_term
    }
}
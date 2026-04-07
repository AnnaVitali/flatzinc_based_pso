use crate::args_extractor::sub_types::float_args_extractor::FloatArgsExtractor;
use crate::data_utility::types::Register;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::evaluator::sub_types::float_evaluator::{
    A_TERM_INDEX, B_TERM_INDEX, COEFF_LIN_CONSTR_INDEX, CONST_LIN_CONSTR_INDEX,
    R_TERM_INDEX, VARS_LIN_CONSTR_INDEX,
};
use crate::data_utility::types::VariableValue;
use flatzinc_serde::{Array, Literal};
use std::collections::HashMap;

/// Struct responsible for assigning float variables based on constraints and solutions.
///
/// # Fields
/// * `args_extractor` - Extracts arguments for float constraints.
/// * `arrays` - Stores arrays mapped by their identifiers.
#[derive(Debug, Clone, Default)]
pub struct FloatVariableAssigner {
    /// An instance of `FloatArgsExtractor` used to extract arguments from float constraints.
    args_extractor: FloatArgsExtractor,
    /// A hashmap that maps variable identifiers to their corresponding registers, used for resolving variable references in constraints.
    variable_register_map: HashMap<String, Register>,
    /// A hashmap that maps identifiers to their corresponding arrays, used for resolving array references in constraints.
    arrays: HashMap<String, Array>,
}

impl FloatVariableAssigner {
    /// Creates a new `FloatVariableAssigner` with the provided arrays.
    ///
    /// # Arguments
    /// * `arrays` - A map from identifiers to arrays used in float constraints.
    ///
    /// # Returns
    /// A new instance of `FloatVariableAssigner`.
    pub fn new(arrays: HashMap<String, Array>, variable_map: HashMap<String, Register>) -> Self {
        let args_extractor = FloatArgsExtractor::new();

        Self {
            args_extractor,
            variable_register_map: variable_map,
            arrays,
        }
    }

    /// Returns a closure that evaluates the `array_float_element` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the float value from the array or a specific value depending on the defined variable.
    pub fn array_float_element(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let index_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied().expect("Index register not found");
        let array: Vec<f64> = self.arrays.get(vars_involved.get(&B_TERM_INDEX).unwrap()).expect("Expect a constant array for array_float_element constraint")
            .contents
            .iter()
            .map(|elem| match elem {
                Literal::Float(i) => *i,
                _ => panic!("Expected float literal in array for array_float_element constraint"),
            })
            .collect();
        Box::new(move |solution: &[Option<VariableValue>]| {
            array[solution[index_register as usize].as_ref().expect("Missing value for register").as_float() as usize]
        })
    }

    pub fn array_var_float_element(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
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
        let variable_map = self.variable_register_map.clone();

        Box::new(move |solution: &[Option<VariableValue>]| {
            let idx_in_array = solution[index_register as usize].as_ref().expect("Missing value for register").as_int() as usize;
            let var_name = &array[idx_in_array];
            let var_idx = variable_map.get(var_name).copied().expect("Array value not found") as usize;
            
            solution[var_idx].as_ref().expect("Missing value for register").as_float()
        })
    }
    /// Returns a closure that evaluates the `float_abs` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the absolute value of a float or a specific value depending on the defined variable.
    pub fn float_abs(
        &self,
        constraint: &CallWithDefines,
   ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.abs()
        })
    }

    /// Returns a closure that evaluates the `float_div` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the division of two float values or a specific value depending on the defined variable.
    pub fn float_div(
        &self,
        constraint: &CallWithDefines,
     ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
                let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            a_value / b_value
        })
    }

    /// Returns a closure that evaluates the `float_eq` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the equality of two float values or a specific value depending on the defined variable.
    pub fn float_eq(
        &self,
        constraint: &CallWithDefines,       
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value
        })
    }

    /// Returns a closure that evaluates the `float_max` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the maximum of two float values or a specific value depending on the defined variable.
    pub fn float_max(
        &self,
        constraint: &CallWithDefines,        
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            a_value.max(b_value)
        })
    }

    /// Returns a closure that evaluates the `float_min` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the minimum of two float values or a specific value depending on the defined variable.
    pub fn float_min(
        &self,
        constraint: &CallWithDefines,       
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            a_value.min(b_value)
        })
    }

    /// Returns a closure that evaluates the `float_plus` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the sum of two float values or a specific value depending on the defined variable.
    pub fn float_plus(
        &self,
        constraint: &CallWithDefines,        
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            a_value + b_value
        })
    }

    /// Returns a closure that evaluates the `float_pow` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the result of raising one float to the power of another or a specific value depending on the defined variable.
    pub fn float_pow(
        &self,
        constraint: &CallWithDefines,       
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            a_value.powf(b_value)
        })
    }

    /// Returns a closure that evaluates the `float_times` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the product of two float values or a specific value depending on the defined variable.
    pub fn float_times(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }

            a_value * b_value
        })
    }

    /// Returns a closure that evaluates the `float_acos` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the arccosine of a float value or a specific value depending on the defined variable.
    pub fn float_acos(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.acos()
        })
    }

    /// Returns a closure that evaluates the `float_acosh` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the hyperbolic arccosine of a float value or a specific value depending on the defined variable.
    pub fn float_acosh(
        &self,
        constraint: &CallWithDefines,       
   ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.acosh()
        })
    }

    /// Returns a closure that evaluates the `float_asin` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the arcsine of a float value or a specific value depending on the defined variable.
    pub fn float_asin(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.asin()
        })
    }

    /// Returns a closure that evaluates the `float_asinh` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the hyperbolic arcsine of a float value or a specific value depending on the defined variable.
    pub fn float_asinh(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.asinh()
        })
    }

    /// Returns a closure that evaluates the `float_atan` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the arctangent of a float value or a specific value depending on the defined variable.
    pub fn float_atan(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.atan()
        })
    }

    /// Returns a closure that evaluates the `float_atanh` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the hyperbolic arctangent of a float value or a specific value depending on the defined variable.
    pub fn float_atanh(
        &self,
        constraint: &CallWithDefines,
   ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.atanh()
        })
    }

    /// Returns a closure that evaluates the `float_cos` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the cosine of a float value or a specific value depending on the defined variable.
    pub fn float_cos(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.cos()
        })
    }

    /// Returns a closure that evaluates the `float_cosh` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the hyperbolic cosine of a float value or a specific value depending on the defined variable.
    pub fn float_cosh(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.cosh()
        })
    }

    /// Returns a closure that evaluates the `float_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether two float values are equal, or a boolean value depending on the defined variable.
    pub fn float_eq_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            let r_value: Option<bool>;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(solution[r_register.unwrap() as usize].as_ref().expect("Missing value for register").as_bool());
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            }else{
                r_value = None;
            }

            if r_value.is_none() {
               a_value == b_value
            } else {
                (a_value == b_value) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `float_exp` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the exponential of a float value or a specific value depending on the defined variable.
    pub fn float_exp(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.exp()
        })
    }

    /// Returns a closure that evaluates the `float_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one float value is less than or equal to another, or a boolean value depending on the defined variable.
    pub fn float_le_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            let r_value: Option<bool>;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(solution[r_register.unwrap() as usize].as_ref().expect("Missing value for register").as_bool());
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            }else{
                r_value = None;
            }

            if r_value.is_none() {
               a_value <= b_value
            } else {
                (a_value <= b_value) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `float_lin_eq` constraint for a specific variable.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    /// * `variable` - The variable to solve for in the linear equation.
    ///
    /// # Returns
    /// A closure that, given a solution, returns the value of the specified variable in the linear equation.
    pub fn float_lin_eq(
        &self,
        constraint: &CallWithDefines,
        variable: &String,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let mut registers = Vec::new();
        for var in &vars_involved {
            let reg = self.variable_register_map.get(var).copied().expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let var_idx = vars_involved.iter().position(|id| id == variable)
            .expect("Assigned variable not found in vars_involved");
        let var_coeff = coeff[var_idx];
        let constant_term: f64 = self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[Option<VariableValue>]| {
            let mut sum = 0f64;
            for (i, &reg) in registers.iter().enumerate() {
                if i != var_idx {
                    sum += coeff[i] * solution[reg as usize].as_ref().expect("Missing value for register").as_float();
                }
            }
            let result = (constant_term - sum) / var_coeff;
            result
        })
    }

    /// Returns a closure that evaluates the `float_lin_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear equation holds, or a boolean value depending on the defined variable.
    pub fn float_lin_eq_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let mut registers = Vec::new();
        for var in &vars_involved {
            let reg = self.variable_register_map.get(var).copied().expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let defines = constraint.defines.clone();
        let r_register = defines.as_ref().and_then(|r| self.variable_register_map.get(r).copied());
        let r_const = if r_register.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call))
        } else {
            None
        };
        let constant_term: f64 = self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[Option<VariableValue>]| {
            let left_side_term = {
                let mut sum = 0f64;
                for (i, &reg) in registers.iter().enumerate() {
                    sum += coeff[i] * solution[reg as usize].as_ref().expect("Missing value for register").as_float();
                }
                sum
            };
            let r_value: Option<bool>;
             if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(solution[r_register.unwrap() as usize].as_ref().expect("Missing value for register").as_bool());
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            }else{
                r_value = None;
            }
            
            if r_value.is_none() {
                left_side_term == constant_term
            } else {
                (left_side_term == constant_term) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `float_lin_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear expression is less than or equal to the term, or a boolean value depending on the defined variable.
    pub fn float_lin_le_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let mut registers = Vec::new();
        for var in &vars_involved {
            let reg = self.variable_register_map.get(var).copied().expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let defines = constraint.defines.clone();
        let r_register = defines.as_ref().and_then(|r| self.variable_register_map.get(r).copied());
        let r_const = if r_register.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call))
        } else {
            None
        };
        let constant_term: f64 = self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[Option<VariableValue>]| {
            let left_side_term = {
                let mut sum = 0f64;
                for (i, &reg) in registers.iter().enumerate() {
                    sum += coeff[i] * solution[reg as usize].as_ref().expect("Missing value for register").as_float();
                }
                sum
            };
            let r_value: Option<bool>;
             if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(solution[r_register.unwrap() as usize].as_ref().expect("Missing value for register").as_bool());
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            }else{
                r_value = None;
            }

            if r_value.is_none() {
                left_side_term <= constant_term
            } else {
                (left_side_term <= constant_term) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `float_lin_lt_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear expression is less than the term, or a boolean value depending on the defined variable.
    pub fn float_lin_lt_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let mut registers = Vec::new();
        for var in &vars_involved {
            let reg = self.variable_register_map.get(var).copied().expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let defines = constraint.defines.clone();
        let r_register = defines.as_ref().and_then(|r| self.variable_register_map.get(r).copied());
        let r_const = if r_register.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call))
        } else {
            None
        };
        let constant_term: f64 = self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[Option<VariableValue>]| {
            let left_side_term = {
                let mut sum = 0f64;
                for (i, &reg) in registers.iter().enumerate() {
                    sum += coeff[i] * solution[reg as usize].as_ref().expect("Missing value for register").as_float();
                }
                sum
            };
            let r_value: Option<bool>;
             if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(solution[r_register.unwrap() as usize].as_ref().expect("Missing value for register").as_bool());
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            }else{
                r_value = None;
            }
            if r_value.is_none() {
                left_side_term < constant_term
            } else {
                (left_side_term < constant_term) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `float_lin_ne_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether the linear expression is not equal to the term, or a boolean value depending on the defined variable.
    pub fn float_lin_ne_reif(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let mut registers = Vec::new();
        for var in &vars_involved {
            let reg = self.variable_register_map.get(var).copied().expect("Variable in linear constraint not found in variable map");
            registers.push(reg);
        }
        let defines = constraint.defines.clone();
        let r_register = defines.as_ref().and_then(|r| self.variable_register_map.get(r).copied());
        let r_const = if r_register.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call))
        } else {
            None
        };
        let constant_term: f64 = self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call);

        Box::new(move |solution: &[Option<VariableValue>]| {
            let left_side_term = {
                let mut sum = 0f64;
                for (i, &reg) in registers.iter().enumerate() {
                    sum += coeff[i] * solution[reg as usize].as_ref().expect("Missing value for register").as_float();
                }
                sum
            };
           let r_value: Option<bool>;
             if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(solution[r_register.unwrap() as usize].as_ref().expect("Missing value for register").as_bool());
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            }else{
                r_value = None;
            }
            
            if r_value.is_none() {
                left_side_term != constant_term
            } else {
                (left_side_term != constant_term) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `float_ln` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the natural logarithm of a float value or a specific value depending on the defined variable.
    pub fn float_ln(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.ln()
        })
    }

    /// Returns a closure that evaluates the `float_log10` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the base-10 logarithm of a float value or a specific value depending on the defined variable.
    pub fn float_log10(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.log10()
        })
    }

    /// Returns a closure that evaluates the `float_log2` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the base-2 logarithm of a float value or a specific value depending on the defined variable.
    pub fn float_log2(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.log2()
        })
    }

    /// Returns a closure that evaluates the `float_lt_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether one float value is less than another, or a boolean value depending on the defined variable.
    pub fn float_lt_reif(
        &self,
        constraint: &CallWithDefines,
     ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            let r_value: Option<bool>;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(solution[r_register.unwrap() as usize].as_ref().expect("Missing value for register").as_bool());
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            }else{
                r_value = None;
            }

            if r_value.is_none() {
               a_value < b_value
            } else {
                (a_value < b_value) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `float_ne_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns whether two float values are not equal, or a boolean value depending on the defined variable.
    pub fn float_ne_reif(
        &self,
        constraint: &CallWithDefines,
     ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
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

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            let b_value;
            let r_value: Option<bool>;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }
            if b_register.is_some() {
                b_value = solution[b_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                b_value = b_const.expect("Expected constant value for B_TERM");
            }
            if r_register.is_some() && solution[r_register.unwrap() as usize].is_some() {
                r_value = Some(solution[r_register.unwrap() as usize].as_ref().expect("Missing value for register").as_bool());
            } else if r_const.is_some() {
                r_value = Some(r_const.expect("Expected constant value for R_TERM"));
            }else{
                r_value = None;
            }

            if r_value.is_none() {
               a_value != b_value
            } else {
                (a_value != b_value) == r_value.unwrap()
            }
        })
    }

    /// Returns a closure that evaluates the `float_sin` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the sine of a float value or a specific value depending on the defined variable.
    pub fn float_sin(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.sin()
        })
    }

    /// Returns a closure that evaluates the `float_sinh` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the hyperbolic sine of a float value or a specific value depending on the defined variable.
    pub fn float_sinh(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.sinh()
        })
    }

    /// Returns a closure that evaluates the `float_sqrt` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the square root of a float value or a specific value depending on the defined variable.
    pub fn float_sqrt(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.sqrt()
        })
    }

    /// Returns a closure that evaluates the `float_tan` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the tangent of a float value or a specific value depending on the defined variable.
    pub fn float_tan(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.tan()
        })
    }

    /// Returns a closure that evaluates the `float_tanh` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the hyperbolic tangent of a float value or a specific value depending on the defined variable.
    pub fn float_tanh(
        &self,
        constraint: &CallWithDefines,
   ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_float();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value.tanh()
        })
    }

    /// Returns a closure that evaluates the `int2float` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint and its defines to evaluate.

    ///
    /// # Returns
    /// A closure that, given a solution, returns the float representation of an integer value or a specific value depending on the defined variable.
    pub fn int2float(
        &self,
        constraint: &CallWithDefines,
        
   ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let mut a_register = None;
        let mut a_const = None;
        if vars_involved.get(&A_TERM_INDEX).is_some() {
            a_register = self.variable_register_map.get(vars_involved.get(&A_TERM_INDEX).unwrap()).copied();
        }else {
           a_const = Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call));
        }

        Box::new(move |solution: &[Option<VariableValue>]| {
            let a_value;
            if a_register.is_some(){
                a_value = solution[a_register.unwrap() as usize].as_ref().expect("Missing value for register").as_int();
            } else {
                a_value = a_const.expect("Expected constant value for A_TERM");
            }

            a_value as f64
        })
    }
}

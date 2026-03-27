use crate::{args_extractor::sub_types::float_args_extractor::FloatArgsExtractor, data_utility::logger::write_verbose_output};
use crate::solution_provider::VariableValue;
use flatzinc_serde::Array;
use log::info;
use std::collections::HashMap;
use crate::evaluator::mini_evaluator::CallWithDefines;

pub const A_TERM_INDEX: usize = 0;
pub const B_TERM_INDEX: usize = 1;
pub const C_TERM_INDEX: usize = 2;
pub const R_TERM_INDEX: usize = 2;
pub const R_TERM_LIN_EXPR_REIF_INDEX: usize = 3;
pub const COEFF_LIN_CONSTR_INDEX: usize = 0;
pub const VARS_LIN_CONSTR_INDEX: usize = 1;
pub const CONST_LIN_CONSTR_INDEX: usize = 2;
pub const FLOAT_EQ_TOLERANCE: f64 = 1e-4;

/// Evaluator for float constraints in MiniZinc models, providing functional evaluators for various float operations.
///
/// This struct stores arrays, an argument extractor, and a verbosity flag, and provides methods to generate functional evaluators for float constraints such as arithmetic, trigonometric, and comparison operations.
#[derive(Debug, Clone, Default)]
pub struct FloatEvaluator {
    /// Map of array identifiers to their values.
    arrays: HashMap<String, Array>,
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
    /// * `verbose` - If true, enables verbose logging of constraint violations.
    ///
    /// # Returns
    /// A new `FloatFunctionalEvaluator` instance.
    pub fn new(arrays: HashMap<String, Array>, verbose: bool) -> Self {
        let args_extractor = FloatArgsExtractor::new();
        Self {
            arrays,
            args_extractor,
            verbose,
        }
    }

    /// Returns a functional evaluator for the `array_float_element` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    ///
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn array_float_element(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);
        let c_value: Option<f64> = if c_key.is_none() {
            Some(self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let array_value =
                args_extractor.extract_float_element_in_array(&call, &arrays, solution);

            let value = if let Some(const_val) = c_value {
                const_val
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if (array_value - value).abs() > FLOAT_EQ_TOLERANCE {
                if verbose {
                    let c_display = if let Some(cv) = c_value {
                        cv.to_string()
                    } else {
                        c_key.as_ref().unwrap().to_string()
                     };
                    info!("Violated constraint: array_float_element {} = {}", array_value, c_display);
                }
                violation = (array_value - value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_abs` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    ///
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_abs(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if !float_eq_tol(a_value.abs(), b_value) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_abs {} = {}", a_display, b_display);
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
    /// * `solution` - The current solution map.
    ///
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_acos(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            let lhs = a_value.acos();
            if !float_eq_tol(lhs, b_value) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_acos {} = {}", a_display, b_display);
                }
                violation = (lhs - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_acosh` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_acosh(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };
            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            let lhs = a_value.acosh();
            if !float_eq_tol(lhs, b_value) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_acosh {} = {}", a_display, b_display);
                }
                violation = (lhs - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_asin` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_asin(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            let lhs = a_value.asin();
            if !float_eq_tol(lhs, b_value) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_asin {} = {}", a_display, b_display);
                }
                violation = (lhs - b_value).abs();
            }
            violation
        })
    }

    pub fn float_asinh(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            let lhs = a_value.asinh();
            if !float_eq_tol(lhs, b_value) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_asinh {} = {}", a_display, b_display);
                }
                violation = (lhs - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_atan` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_atan(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };
            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            let lhs = a_value.atan();
            if !float_eq_tol(lhs, b_value) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_atan {} = {}", a_display, b_display);
                }
                violation = (lhs - b_value).abs();
            }
            violation
        })
    }

    pub fn float_atanh(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };


            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if a_value.abs() >= 1.0 || b_value.abs() >= 1.0 {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Invalid input for float_atanh {} or {} outside (-1,1)", a_display, b_display);
                }
                violation = 1.0;
            } else {
                let lhs = a_value.atanh();
                if !float_eq_tol(lhs, b_value) {
                    if verbose {
                        let a_display = if let Some(av) = a_const {
                            av.to_string()
                        } else {
                            a_key.as_ref().unwrap().to_string()
                        };
                        let b_display = if let Some(bv) = b_const {
                            bv.to_string()
                        } else {
                            b_key.as_ref().unwrap().to_string()
                        };
                        info!("Violated constraint: float_atanh {} = {}", a_display, b_display);
                    }
                    violation = (lhs - b_value).abs();
                }
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_cos` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_cos(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<f64> = if b_key.is_none() { 
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

           let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            let lhs = a_value.cos();
            if !float_eq_tol(lhs, b_value) {
                    if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_cos {} = {}", a_display, b_display);
                }
                violation = (lhs - b_value).abs();
            }
            violation
        })
    }

    pub fn float_cosh(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
           let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            let lhs = a_value.cosh();
                if !float_eq_tol(lhs, b_value) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_cosh {} = {}", a_display, b_display);
                }
                violation = (lhs - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_cos` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_div(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let c_const: Option<f64> = if c_key.is_none() {
            Some(self.args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;
        let float_eq_violation = |x: f64, y: f64| ((x - y).abs() - FLOAT_EQ_TOLERANCE).max(0.0);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let c_value = if let Some(cv) = c_const {
                cv
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if b_value == 0.0 {
                violation = 1.0;
            } else {
                let expected = a_value / b_value;
                if !float_eq_tol(c_value, expected) {
                    if verbose {
                        let a_display = if let Some(av) = a_const {
                            av.to_string()
                        } else {
                            a_key.as_ref().unwrap().to_string()
                        };
                        let b_display = if let Some(bv) = b_const {
                            bv.to_string()
                        } else {
                            b_key.as_ref().unwrap().to_string()
                        };
                        let c_display = if let Some(cv) = c_const {
                            cv.to_string()
                        } else {
                            c_key.as_ref().unwrap().to_string()
                        };

                        info!("Violated constraint: float_div {} / {} = {}", a_display, b_display, c_display);
                    }
                    violation = float_eq_violation(c_value, expected);
                }
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_eq` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_eq(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);

        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if !float_eq_tol(a_value, b_value) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()  
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_eq {} = {}", a_display, b_display);
                }
                violation = (a_value - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_eq_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_eq_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let r_key: Option<String> = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);

        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let r_const: Option<bool> = if r_key.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let r_value = if let Some(rv) = r_const {
                rv
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable key for R_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            let eq_res = float_eq_tol(a_value, b_value);
            if r_value != eq_res {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if let Some(rv) = r_const {
                        rv.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_eq_reif {} <-> {} = {}", r_display, a_display, b_display);
                }
                violation = (a_value - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_exp` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_exp(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);

        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let b_key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(b_key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            let lhs = a_value.exp();
            if !float_eq_tol(lhs, b_value) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };

                    info!("Violated constraint: float_exp {} = {}", a_display, b_display);
                }
                violation = (lhs - b_value).abs();
            }
            violation
        })
    }

    pub fn float_le(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let b_key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(b_key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if a_value > b_value {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_le {} <= {}", a_display, b_display);
                }
                violation = (a_value - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_le_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_le_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let r_key: Option<String> = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);

        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let r_const: Option<bool> = if r_key.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let r_value = if let Some(rv) = r_const {
                rv
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable key for R_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if r_value != (a_value <= b_value) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if let Some(rv) = r_const {
                        rv.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_le_reif{} <-> {} <= {}", r_display, a_display, b_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_eq(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let arrays = self.arrays.clone();
        let verbose = self.verbose;
         let literal_vars_map = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let term_key: Option<String> = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<f64> = if term_key.is_none() {
            Some(self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );
        let vars_involved = self.args_extractor.extract_var_values_lin_expr(
            VARS_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::float_lin_left_term(verbose, &coeff, solution, &vars_involved, &mut verbose_terms);
            let term_value = if let Some(tc) = term_const {
                tc
            } else {
                let key_ref = term_key
                    .as_ref()
                    .expect("Expected variable key for CONST_LIN_CONSTR_INDEX when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for term")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for term, found other type"),
                }
            };
            
            let mut violation = 0.0;
            if (left_side_term - term_value).abs() > FLOAT_EQ_TOLERANCE {
                if verbose {
                    let term_display = if let Some(tc) = term_const {
                        tc.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_lin_eq {} = {}", verbose_terms, term_display);
                }
                violation = ((left_side_term - term_value).abs() - FLOAT_EQ_TOLERANCE).max(0.0);
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_lin_eq_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_eq_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let arrays = self.arrays.clone();
        let verbose = self.verbose;

        let literal_vars_map = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let term_key = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<f64> = if term_key.is_none() {
            Some(self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let r_key = self.identifier_from_vars(&literal_vars_map, R_TERM_LIN_EXPR_REIF_INDEX);
        let r_const: Option<bool> = if r_key.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let coeff = self
            .args_extractor
            .extract_float_coefficients_lin_expr(COEFF_LIN_CONSTR_INDEX, &constraint.call, &arrays);

        let vars_vec = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &arrays);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
            let left_side_term =
                Self::float_lin_left_term(verbose, &coeff, solution, &vars_vec, &mut verbose_terms);
            let term_value = if let Some(tc) = term_const {
                tc
            } else {
                let key_ref = term_key
                    .as_ref()
                    .expect("Expected variable key for CONST_LIN_CONSTR_INDEX when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for term")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for term, found other type"),
                }
            };

            let r_value = if let Some(rv) = r_const {
                rv
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable key for R_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for r")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for r, found other type"),
                }
            };

            let mut violation = 0.0;
            let eq_res = (left_side_term - term_value).abs() <= FLOAT_EQ_TOLERANCE;
            if r_value != eq_res {
                if verbose {
                    let term_display = if let Some(tc) = term_const {
                        tc.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if let Some(rv) = r_const {
                        rv.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_lin_eq_reif {} <-> {} = {}", r_display, verbose_terms, term_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_le(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let arrays = self.arrays.clone();
        let verbose = self.verbose;
        let literal_vars_map = self.args_extractor.extract_literal_identifier_with_index(&constraint.call.args);
        let term_key = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<f64> = if term_key.is_none() {
            Some(self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );
        let vars_involved = self.args_extractor.extract_var_values_lin_expr(
            VARS_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
            let left_side_term =
                Self::float_lin_left_term(verbose, &coeff, solution, &vars_involved, &mut verbose_terms);

            let term_value = if let Some(tc) = term_const {
                tc
            } else {
                let key_ref = term_key
                    .as_ref()
                    .expect("Expected variable key for CONST_LIN_CONSTR_INDEX when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for term")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for term, found other type"),
                }
            };

            let mut violation = 0.0;
            if left_side_term > term_value {
                if verbose {
                    let term_display = if let Some(tc) = term_const {
                        tc.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: {} <= {}", verbose_terms, term_display);
                }
                violation = (left_side_term - term_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_le_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_le_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let arrays = self.arrays.clone();
        let verbose = self.verbose;
        let literal_vars_map = self.args_extractor.extract_literal_identifier_with_index(&constraint.call.args);
        let term_key = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<f64> = if term_key.is_none() {
            Some(self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let r_key = self.identifier_from_vars(&literal_vars_map, R_TERM_LIN_EXPR_REIF_INDEX);
        let r_const: Option<bool> = if r_key.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );
        let vars_involved = self.args_extractor.extract_var_values_lin_expr(
            VARS_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
            let left_side_term =
                Self::float_lin_left_term(verbose, &coeff, solution, &vars_involved, &mut verbose_terms);

            let term_value = if let Some(tc) = term_const {
                tc
            } else {
                let key_ref = term_key
                    .as_ref()
                    .expect("Expected variable key for CONST_LIN_CONSTR_INDEX when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for term")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for term, found other type"),
                }
            };
            let r_value = if let Some(rv) = r_const {
                rv
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable key for R_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for r")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for r, found other type"),
                }
            };

            let mut violation = 0.0;
            if r_value != (left_side_term <= term_value) {
                if verbose {
                    let term_display = if let Some(tc) = term_const {
                        tc.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if let Some(rv) = r_const {
                        rv.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: lin_le_reif {} <-> {} <= {}", r_display, verbose_terms, term_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_lt(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let arrays = self.arrays.clone();
        let verbose = self.verbose;
        let literal_vars_map = self.args_extractor.extract_literal_identifier_with_index(&constraint.call.args);
        let term_key = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<f64> = if term_key.is_none() {
            Some(self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );
        let vars_involved = self.args_extractor.extract_var_values_lin_expr(
            VARS_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
            let left_side_term =
                Self::float_lin_left_term(verbose, &coeff, solution, &vars_involved, &mut verbose_terms);
            let term_value = if let Some(tc) = term_const {
                tc
            } else {
                let key_ref = term_key
                    .as_ref()
                    .expect("Expected variable key for CONST_LIN_CONSTR_INDEX when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for term")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for term, found other type"),
                }
            };

            let mut violation = 0.0;
            if left_side_term >= term_value {
                if verbose {
                    let term_display = if let Some(tc) = term_const {
                        tc.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_lin_lt {} < {}", verbose_terms, term_display);
                }
                violation = (left_side_term - term_value).abs() + 1.0;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_lin_lt_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_lt_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let arrays = self.arrays.clone();
        let verbose = self.verbose;
        let literal_vars_map = self.args_extractor.extract_literal_identifier_with_index(&constraint.call.args);
        let term_key = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<f64> = if term_key.is_none() {
            Some(self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let r_key = self.identifier_from_vars(&literal_vars_map, R_TERM_LIN_EXPR_REIF_INDEX);
        let r_const: Option<bool> = if r_key.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );
        let vars_involved = self.args_extractor.extract_var_values_lin_expr(
            VARS_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
            let left_side_term =
                Self::float_lin_left_term(verbose, &coeff, solution, &vars_involved, &mut verbose_terms);
            let term_value = if let Some(tc) = term_const {
                tc
            } else {
                let key_ref = term_key
                    .as_ref()
                    .expect("Expected variable key for CONST_LIN_CONSTR_INDEX when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for term")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for term, found other type"),
                }
            };
            let r_value = if let Some(rv) = r_const {
                rv
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable key for R_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for r")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for r, found other type"),
                }
            };

            let mut violation = 0.0;
            if r_value != (left_side_term < term_value) {
                if verbose {
                    let term_display = if let Some(tc) = term_const {
                        tc.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if let Some(rv) = r_const {
                        rv.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_lin_lt_reif {} <-> {} < {}", r_display, verbose_terms, term_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_ne(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let arrays = self.arrays.clone();
        let verbose = self.verbose;

        let literal_vars_map = self.args_extractor.extract_literal_identifier_with_index(&constraint.call.args);
        let term_key = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<f64> = if term_key.is_none() {
            Some(self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );
        let vars_involved = self.args_extractor.extract_var_values_lin_expr(
            VARS_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
            let left_side_term =
                Self::float_lin_left_term(verbose, &coeff, solution, &vars_involved, &mut verbose_terms);

            let term_value = if let Some(tc) = term_const {
                tc
            } else {
                let key_ref = term_key
                    .as_ref()
                    .expect("Expected variable key for CONST_LIN_CONSTR_INDEX when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for term")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for term, found other type"),
                }
            };

            let mut violation = 0.0;
            if (left_side_term - term_value).abs() <= FLOAT_EQ_TOLERANCE {
                if verbose {
                    let term_display = if let Some(tc) = term_const {
                        tc.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_lin_ne {} != {}", verbose_terms, term_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lin_ne_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let arrays = self.arrays.clone();
        let verbose = self.verbose;
        let literal_vars_map = self.args_extractor.extract_literal_identifier_with_index(&constraint.call.args);
        let term_key = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<f64> = if term_key.is_none() {
            Some(self.args_extractor.extract_float_value(CONST_LIN_CONSTR_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let r_key = self.identifier_from_vars(&literal_vars_map, R_TERM_LIN_EXPR_REIF_INDEX);
        let r_const: Option<bool> = if r_key.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let coeff = self.args_extractor.extract_float_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );
        let vars_involved = self.args_extractor.extract_var_values_lin_expr(
            VARS_LIN_CONSTR_INDEX,
            &constraint.call,
            &arrays,
        );

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
            let left_side_term =
                Self::float_lin_left_term(verbose, &coeff, solution, &vars_involved, &mut verbose_terms);
            let term_value = if let Some(tc) = term_const {
                tc
            } else {
                let key_ref = term_key
                    .as_ref()
                    .expect("Expected variable key for CONST_LIN_CONSTR_INDEX when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for term")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for term, found other type"),
                }
            };
            let r_value = if let Some(rv) = r_const {
                rv
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable key for R_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for r")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for r, found other type"),
                }
            };

            let mut violation = 0.0;
            let lhs_ne_term = (left_side_term - term_value).abs() > FLOAT_EQ_TOLERANCE;
            if r_value != lhs_ne_term {
                if verbose {
                    let term_display = if let Some(tc) = term_const {
                        tc.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if let Some(rv) = r_const {
                        rv.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_lin_ne {} <-> {} != {}", r_display, verbose_terms, term_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_ln(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);

        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if !float_eq_tol(a_value.ln(), b_value) {
                if verbose {
                    let a_display = if let Some(ac) = a_const {
                        ac.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bc) = b_const {
                        bc.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_ln {} = {}", a_display, b_display);
                }
                violation = (a_value - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_log10` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_log10(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);

        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if !float_eq_tol(a_value.log10(), b_value) {
                if verbose {
                    let a_display = if let Some(ac) = a_const {
                        ac.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bc) = b_const {
                        bc.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_log10 {} = {}", a_display, b_display);
                }
                violation = (a_value - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_log2` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_log2(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);

        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if !float_eq_tol(a_value.log2(), b_value) {
                if verbose {
                    let a_display = if let Some(ac) = a_const {
                        ac.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bc) = b_const {
                        bc.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_log2 {} = {}", a_display, b_display);
                }
                violation = (a_value - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_log2` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lt(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if a_value >= b_value {
                if verbose {
                    let a_display = if let Some(ac) = a_const {
                        ac.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bc) = b_const {
                        bc.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };

                    info!("Violated constraint: float lt {} < {}", a_display, b_display);
                }
                violation = (a_value - b_value).abs();
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_lt_reif` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_lt_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let literal_vars_map = self.args_extractor.extract_literal_identifier_with_index(&constraint.call.args);
        let verbose = self.verbose;

        let a_key: Option<String> = self.identifier_from_vars(&literal_vars_map, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&literal_vars_map, B_TERM_INDEX);
        let r_key: Option<String> = self.identifier_from_vars(&literal_vars_map, R_TERM_INDEX);

        let a_const: Option<f64> = if a_key.is_none() {
            Some(self.args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<f64> = if b_key.is_none() {
            Some(self.args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let r_const: Option<bool> = if r_key.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for A_TERM, found other type"),
                }
            };

            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type"),
                }
            };

            let r_value = if let Some(rv) = r_const {
                rv
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable key for R_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for R_TERM, found other type"),
                }
            };

            let mut violation = 0.0;
            if r_value != (a_value < b_value) {
                if verbose {
                   let a_dsiplay = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if let Some(rv) = r_const {
                        rv.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };

                    info!("Violated constraint: float_lt_reif {} <-> {} < {}", r_display, a_dsiplay, b_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_max(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let verbose = self.verbose;
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let c_const: Option<f64> = if c_key.is_none() {
            Some(args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;
        let float_eq_violation = |x: f64, y: f64| ((x - y).abs() - FLOAT_EQ_TOLERANCE).max(0.0);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for A_TERM, found other type"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type"),
                }
            };

            let c_value = if let Some(cc) = c_const {
                cc
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for C_TERM, found other type"),
                }
            };

            let mut violation = 0.0;
            let expected = a_value.max(b_value);
            if !float_eq_tol(c_value, expected) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let c_display = if let Some(cv) = c_const {
                        cv.to_string()
                    } else {
                        c_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_max max({},{}) = {}", a_display, b_display, c_display);
                }
                violation = float_eq_violation(c_value, expected);
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_min` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_min(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let verbose = self.verbose;
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);
        let a_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let c_const: Option<f64> = if c_key.is_none() {
            Some(args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;
        let float_eq_violation = |x: f64, y: f64| ((x - y).abs() - FLOAT_EQ_TOLERANCE).max(0.0);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for A_TERM, found other type"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type"),
                }
            };

            let c_value = if let Some(cc) = c_const {
                cc
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for C_TERM, found other type"),
                }
            };

            let mut violation = 0.0;
            let expected = a_value.min(b_value);
            if !float_eq_tol(c_value, expected) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let c_display = if let Some(cv) = c_const {
                        cv.to_string()
                    } else {
                        c_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_min min({}, {}) = {}", a_display, b_display, c_display);
                }
                violation = float_eq_violation(c_value, expected);
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_ne` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn float_ne(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let verbose = self.verbose;
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);

        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);

        let a_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for A_TERM, found other type"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type"),
                }
            };

            let mut violation = 0.0;
            if float_eq_tol(a_value, b_value) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_ne {} != {}", a_display, b_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn float_ne_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let verbose = self.verbose;
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let r_key: Option<String> = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);

        let a_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let r_const: Option<bool> = if r_key.is_none() {
            Some(args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for A_TERM") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for A_TERM, found other type"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for B_TERM") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type"),
                }
            };

            let r_value = if let Some(rv) = r_const {
                rv
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable key for R_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for R_TERM") {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for R_TERM, found other type"),
                }
            };

            let mut violation = 0.0;
            let eq_res = float_eq_tol(a_value, b_value);
            let ne_res = !eq_res;
            if r_value != ne_res {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if let Some(rv) = r_const {
                        rv.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_ne_reif {} <-> {} != {}", r_display, a_display, b_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_plus(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let verbose = self.verbose;
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);

        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let c_const: Option<f64> = if c_key.is_none() {
            Some(args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;
        let float_eq_violation = |x: f64, y: f64| ((x - y).abs() - FLOAT_EQ_TOLERANCE).max(0.0);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for A_TERM") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for A_TERM, found other type"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for B_TERM") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type"),
                }
            };

            let c_value = if let Some(cc) = c_const {
                cc
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for C_TERM") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for C_TERM, found other type"),
                }
            };

            let result = a_value + b_value;
            let mut violation = 0.0;
            if !float_eq_tol(c_value, result) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let c_display = if let Some(cv) = c_const {
                        cv.to_string()
                    } else {
                        c_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_plus {} + {} = {}", a_display, b_display, c_display);
                }
                violation = float_eq_violation(c_value, result);
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_pow` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_pow(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let verbose = self.verbose;
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);

        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let c_const: Option<f64> = if c_key.is_none() {
            Some(args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;
        let float_eq_violation = |x: f64, y: f64| ((x - y).abs() - FLOAT_EQ_TOLERANCE).max(0.0);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for A_TERM") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for A_TERM, found other type"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for B_TERM") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type"),
                }
            };

            let c_value = if let Some(cc) = c_const {
                cc
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for C_TERM") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for C_TERM, found other type"),
                }
            };

            let result = a_value.powf(b_value);
            let mut violation = 0.0;
            if !float_eq_tol(c_value, result) {
                if verbose {
                    let a_display = if let Some(av) = a_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bv) = b_const {
                        bv.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let c_display = if let Some(cv) = c_const {
                        cv.to_string()
                    } else {
                        c_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_pow {} ^ {} = {}", a_display, b_display, c_display);
                }
                violation = float_eq_violation(c_value, result);
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_sin` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_sin(
        &self,
        constraint: &CallWithDefines,
         solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let verbose = self.verbose;
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_value_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);

        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;
        let float_eq_violation = |x: f64, y: f64| ((x - y).abs() - FLOAT_EQ_TOLERANCE).max(0.0);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_value_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for A_TERM, found other type"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type"),
                }
            };

            let mut violation = 0.0;
            let expected = a_value.sin();
            if !float_eq_tol(b_value, expected) {
                if verbose {
                    let a_display = if let Some(av) = a_value_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bc) = b_const {
                        bc.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };

                    info!("Violated constraint: float_sin {} = {}", a_display, b_display);
                }
                violation = float_eq_violation(b_value, expected);
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_sinh` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_sinh(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let verbose = self.verbose;
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);

          let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_value_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);

        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;
        let float_eq_violation = |x: f64, y: f64| ((x - y).abs() - FLOAT_EQ_TOLERANCE).max(0.0);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
           let a_value = if let Some(ac) = a_value_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for A_TERM, found other type"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type"),
                }
            };

            let mut violation = 0.0;
            let expected = a_value.sinh();
            if !float_eq_tol(b_value, expected) {
                if verbose {
                    let a_display = if let Some(av) = a_value_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bc) = b_const {
                        bc.to_string() 
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_sinh {} = {}", a_display, b_display);
                }
                violation = float_eq_violation(b_value, expected);
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_sqrt` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_sqrt(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let verbose = self.verbose;
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);

         let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_value_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);

        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;
        let float_eq_violation = |x: f64, y: f64| ((x - y).abs() - FLOAT_EQ_TOLERANCE).max(0.0);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_value_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for A_TERM, found other type"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type"),
                }
            };

            let mut violation = 0.0;
            let expected = a_value.sqrt();
            if !float_eq_tol(b_value, expected) {
                if verbose {
                    let a_display = if let Some(av) = a_value_const {
                        av.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bc) = b_const {
                        bc.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    
                    info!("Violated constraint: float_sqrt {} = {}", a_display, b_display);
                }
                violation = float_eq_violation(b_value, expected);
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_tan` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_tan(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let verbose = self.verbose;
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_value_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);

        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;
        let float_eq_violation = |x: f64, y: f64| ((x - y).abs() - FLOAT_EQ_TOLERANCE).max(0.0);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
           let a_value = if let Some(ac) = a_value_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for A_TERM, found other type"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type"),
                }
            };

            let mut violation = 0.0;
            let expected = a_value.tan();
            if !float_eq_tol(b_value, expected) {
                if verbose {
                    let a_display = if let Some(ac) = a_value_const {
                        ac.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bc) = b_const {
                        bc.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_tan {} = {}", a_display, b_display);
                }
                violation = float_eq_violation(b_value, expected);
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_tanh` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_tanh(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let verbose = self.verbose;
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);

        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_value_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);

        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;
        let float_eq_violation = |x: f64, y: f64| ((x - y).abs() - FLOAT_EQ_TOLERANCE).max(0.0);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_value_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for A_TERM, found other type"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type"),
                }
            };

            let mut violation = 0.0;
            let expected = a_value.tanh();
            if !float_eq_tol(b_value, expected) {
                if verbose {
                    let a_display = if let Some(ac) = a_value_const {
                        ac.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bc) = b_const {
                        bc.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_tanh {} = {}", a_display, b_display);
                }
                violation = float_eq_violation(b_value, expected);
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `float_times` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn float_times(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let verbose = self.verbose;
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_value_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_float_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);
        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let c_const: Option<f64> = if c_key.is_none() {
            Some(args_extractor.extract_float_value(C_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };


        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;
        let float_eq_violation = |x: f64, y: f64| ((x - y).abs() - FLOAT_EQ_TOLERANCE).max(0.0);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_value_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for A_TERM") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for A_TERM, found other type"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for B_TERM") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type"),
                }
            };

            let c_value = if let Some(cc) = c_const {
                cc
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for C_TERM") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for C_TERM, found other type"),
                }
            };

            let result = a_value * b_value;
            let mut violation = 0.0;
            if !float_eq_tol(c_value, result) {
                if verbose {
                    let a_display = if let Some(ac) = a_value_const {
                        ac.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bc) = b_const {
                        bc.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let c_display = if let Some(cc) = c_const {
                        cc.to_string()
                    } else {
                        c_key.as_ref().unwrap().to_string()
                    };

                    info!("Violated constraint: float_times {} * {} = {}", a_display, b_display, c_display);
                }
                violation = float_eq_violation(c_value, result);
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int2float` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int2float(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let verbose = self.verbose;
        let vars_involved = args_extractor.extract_literal_identifier_with_index(&constraint.call.args);

        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);

        let a_const: Option<f64> = if a_key.is_none() {
            Some(args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution) as f64)
        } else {
            None
        };
        let b_const: Option<f64> = if b_key.is_none() {
            Some(args_extractor.extract_float_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let float_eq_tol = |x: f64, y: f64| (x - y).abs() <= FLOAT_EQ_TOLERANCE;
        let float_eq_violation = |x: f64, y: f64| ((x - y).abs() - FLOAT_EQ_TOLERANCE).max(0.0);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(ac) = a_const {
                ac
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for A_TERM") {
                    VariableValue::Int(i) => *i as f64,
                    _ => panic!("Expected int variable for A_TERM, found other type variable"),
                }
            };

            let b_value = if let Some(bc) = b_const {
                bc
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for B_TERM") {
                    VariableValue::Float(f) => *f,
                    _ => panic!("Expected float variable for B_TERM, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if !float_eq_tol(a_value, b_value) {
                if verbose {
                    let a_display = if let Some(ac) = a_const {
                        ac.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if let Some(bc) = b_const {
                        bc.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: float_eq {} = {}", a_display, b_display);
                }
                violation = float_eq_violation(a_value, b_value);
            }
            violation
        })
    }

    fn float_lin_left_term(
        verbose: bool,
        coeff: &Vec<f64>,
        solution: &HashMap<String, VariableValue>,
        vars_involved: &Vec<String>,
        verbose_terms: &mut String,
    ) -> f64 {
        let left_side_term: f64 = coeff
            .iter()
            .zip(vars_involved.iter())
            .map(|(c, id)| {
                let var_val = solution
                    .get(id)
                    .and_then(|int_val| match int_val {
                        VariableValue::Float(int_val) => Some(*int_val),
                        _ => None,
                    })
                    .unwrap_or_else(|| panic!("No value defined for the variable {}", id));

                if verbose {
                    write_verbose_output(verbose_terms, c, &var_val);
                }

                c * var_val
            })
            .sum();
        left_side_term
    }

    #[inline]
    fn identifier_from_vars(&self, vars: &HashMap<i64, String>, index: usize) -> Option<String> {
        vars.get(&(index as i64)).map(|id| id.to_string())
    }
}
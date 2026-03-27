use crate::args_extractor::sub_types::int_args_extractor::IntArgsExtractor;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Identifier};
use crate::data_utility::logger::write_verbose_output;
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

#[derive(Debug, Clone, Default)]
/// Evaluator for integer constraints, providing methods to evaluate various integer operations and constraints.
///
/// This struct contains methods for evaluating integer constraints such as equality, inequality, linear expressions,
/// and arithmetic operations. It uses argument extraction utilities and supports verbose output for debugging.
pub struct IntEvaluator {
    /// A map of identifiers to arrays used in constraint evaluation.
    arrays: HashMap<Identifier, Array>,
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
    /// * `verbose` - Boolean flag to enable verbose output for debugging.
    ///
    /// # Returns
    /// A new `IntFunctionalEvaluator` instance.
    pub fn new(arrays: HashMap<Identifier, Array>, verbose: bool) -> Self {
        let args_extractor = IntArgsExtractor::new();
        Self {
            arrays,
            args_extractor,
            verbose,
        }
    }

    /// Returns a functional evaluator for the `array_int_element` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn array_int_element(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let vars_involved = args_extractor.extract_literal_identifiers_with_index(&call.args);
        let verbose = self.verbose;
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);
        let c_value: Option<i64> = if c_key.is_none() {
            Some(self.args_extractor.extract_int_value(C_TERM_INDEX, &call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let array_value = args_extractor.extract_int_element_array(&call, &arrays, solution);

            let value = if let Some(cv) = c_value {
                cv
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            println!("array_value in array_int_element: {}", array_value);
            if array_value != value {
                if verbose {
                    let value_display = if c_key.is_none() {
                        value.to_string()
                    } else {
                        c_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: array_int_element {} = {}", array_value, value_display);
                }
                violation = ((array_value - value).abs()) as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `int_abs` constraint.
    /// 
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_abs(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
   
         .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const = if a_key.is_none() {
            Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<i64> = if b_key.is_none() {
            Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution))
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
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable, found other type variable"),
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
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if b_value != a_value.abs() {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_value.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_value.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_abs abs({}) = {}", a_display, b_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_div(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<i64> = if a_key.is_none() {
            Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<i64> = if b_key.is_none() {
            Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let c_const: Option<i64> = if c_key.is_none() {
            Some(self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, solution))
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
                    .expect("Expected variable in solution for A_TERM")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for A_TERM, found other type variable"),
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
                    .expect("Expected variable in solution for B_TERM")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for B_TERM, found other type variable"),
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
                    .expect("Expected variable in solution for C_TERM")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for C_TERM, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if b_value == 0 {
                violation = 1.0;
            } else {
                let result = a_value / b_value;
                if c_value != result {
                    if verbose {
                        let a_display = if a_key.is_none() {
                            a_value.to_string()
                        } else {
                            a_key.as_ref().unwrap().to_string()
                        };
                        let b_display = if b_key.is_none() {
                            b_value.to_string()
                        } else {
                            b_key.as_ref().unwrap().to_string()
                        };
                        let c_display = if c_key.is_none() {
                            c_value.to_string()
                        } else {
                            c_key.as_ref().unwrap().to_string()
                        };
                        info!("Violated constraint: int_div {}/{} = {} ({} / {} -> {})", a_display, b_display, c_display, a_value, b_value, result);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_eq(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
            .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<i64> = if a_key.is_none() {
            Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);

        let b_const: Option<i64> = if b_key.is_none() {
            Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution))
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
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for A_TERM, found other type variable"),
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
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if a_value != b_value {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_value.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_value.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_eq {} = {}", a_display, b_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_eq_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let r_key: Option<String> = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);

        let a_const: Option<i64> = if a_key.is_none() {
            Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<i64> = if b_key.is_none() {
            Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution))
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
                    .expect("Expected variable in solution for A_TERM")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for A_TERM, found other type variable"),
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
                    .expect("Expected variable in solution for B_TERM")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for B_TERM, found other type variable"),
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
                    .expect("Expected variable in solution for R_TERM")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for R_TERM, found other type variable"),
                }
            };

            let mut violation = 0.0;
            let eq_res = a_value == b_value;
            if r_value != eq_res {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_value.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_value.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r_value.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_eq_reif {} <-> {} = {}", a_display, b_display, r_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_le(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const = if a_key.is_none() {
            Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<i64> = if b_key.is_none() {
            Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution))
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
                    .expect("Expected variable in solution for A_TERM")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for A_TERM, found other type variable"),
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
                    .expect("Expected variable in solution for B_TERM")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for B_TERM, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if a_value > b_value {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_value.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_value.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_le {} <= {}", a_display, b_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_le_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let r_key: Option<String> = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);

        let a_const: Option<i64> = if a_key.is_none() {
            Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<i64> = if b_key.is_none() {
            Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution))
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
                    .expect("Expected variable in solution for A_TERM")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for A_TERM, found other type variable"),
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
                    .expect("Expected variable in solution for B_TERM")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for B_TERM, found other type variable"),
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
                    .expect("Expected variable in solution for R_TERM")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for R_TERM, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if r_value != (a_value <= b_value) {
                if verbose {
                   let a_display = if a_key.is_none() {
                        a_value.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_value.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r_value.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: {} <-> {} <= {}", r_display, a_display, b_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_lin_eq(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);
        let literal_vars_map = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let term_key: Option<String> = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<i64> = if term_key.is_none() {
            Some(self.args_extractor.extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::int_lin_left_term(verbose, &coeff, solution, &vars_involved, &mut verbose_terms);

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
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for term, found other type"),
                }
            };

            let mut violation = 0.0;
            if left_side_term != term_value {
                if verbose {
                    let term_display = if term_key.is_none() {
                        term_value.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_lin_eq {} = {}", left_side_term, term_display);
                }
                violation = ((left_side_term - term_value).abs()) as f64;
            }

            violation
        })
    }

    /// Returns a functional evaluator for the `int_lin_eq_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_lin_eq_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let literal_vars_map = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let term_key: Option<String> = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<i64> = if term_key.is_none() {
            Some(self.args_extractor.extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let r_key: Option<String> = self.identifier_from_vars(&literal_vars_map, R_TERM_LIN_EXPR_REIF_INDEX);
        let r_const: Option<bool> = if r_key.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_vec = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::int_lin_left_term(verbose, &coeff, solution, &vars_vec, &mut verbose_terms);
            let term_value = if let Some(tc) = term_const {
                tc
            } else {
                let key_ref = term_key
                    .as_ref()
                    .expect("Expected variable key for term when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for term")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for term"),
                }
            };
            let r_value = if let Some(rv) = r_const {
                rv
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable key for r when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for r")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for r"),
                }
            };

            let mut violation = 0.0;
            if r_value != (left_side_term == term_value) {
                if verbose {
                    let term_display = if term_key.is_none() {
                        term_value.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r_value.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_lin_eq_reif {} <-> {} = {}", r_display, left_side_term, term_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_lin_le(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;

        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let literal_vars_map = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let term_key: Option<String> = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<i64> = if term_key.is_none() {
            Some(self.args_extractor.extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let vars_vec = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
              let left_side_term = Self::int_lin_left_term(verbose, &coeff, solution, &vars_vec, &mut verbose_terms);
            
            let mut violation = 0.0;
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
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for term, found other type"),
                }
            };
            if left_side_term > term_value {
                if verbose {
                    let term_display = if term_key.is_none() {
                        term_value.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_lin_le {} <= {}", left_side_term, term_display);
                }
                violation = ((left_side_term - term_value).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_lin_le_reif` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_lin_le_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let literal_vars_map = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let term_key: Option<String> = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<i64> = if term_key.is_none() {
            Some(self.args_extractor.extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let r_key: Option<String> = self.identifier_from_vars(&literal_vars_map, R_TERM_LIN_EXPR_REIF_INDEX);
        let r_const: Option<bool> = if r_key.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_vec = self.args_extractor.extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::int_lin_left_term(verbose, &coeff, solution, &vars_vec, &mut verbose_terms);

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
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for term, found other type"),
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
                    let term_display = if term_key.is_none() {
                        term_value.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r_value.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_lin_le_reif {} <-> {} <= {}", r_display, left_side_term, term_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_lin_ne(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let literal_vars_map = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let term_key: Option<String> = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<i64> = if term_key.is_none() {
            Some(self.args_extractor.extract_int_value(CONST_LIN_CONSTR_INDEX, &constraint.call, solution))
        } else {
            None
        };

        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &constraint.call,
            &self.arrays,
        );
        let vars_involved = self
            .args_extractor
            .extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &constraint.call, &self.arrays);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::int_lin_left_term(verbose, &coeff, solution, &vars_involved, &mut verbose_terms);

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
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for term, found other type"),
                }
            };

            let mut violation = 0.0;
            if left_side_term == term_value {
                if verbose {
                    let term_display = if term_key.is_none() {
                        term_value.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_lin_ne {} == {}", left_side_term, term_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_lin_ne_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let call = constraint.call.clone();
        let verbose = self.verbose;
        let literal_vars_map = self.args_extractor.extract_literal_identifiers_with_index(&call.args);
        let term_key: Option<String> = self.identifier_from_vars(&literal_vars_map, CONST_LIN_CONSTR_INDEX);
        let term_const: Option<i64> = if term_key.is_none() {
            Some(self.args_extractor.extract_int_value(CONST_LIN_CONSTR_INDEX, &call, solution))
        } else {
            None
        };
        let r_key: Option<String> = self.identifier_from_vars(&literal_vars_map, R_TERM_LIN_EXPR_REIF_INDEX);
        let r_const: Option<bool> = if r_key.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_LIN_EXPR_REIF_INDEX, &call, solution))
        } else {
            None
        };

        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &call,
            &self.arrays,
        );
        let vars_vec =
            self.args_extractor.extract_var_values_lin_expr(VARS_LIN_CONSTR_INDEX, &call, &self.arrays);

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut verbose_terms = String::new();
            let left_side_term = Self::int_lin_left_term(verbose, &coeff, solution, &vars_vec, &mut verbose_terms);

            let term_value = if let Some(tc) = term_const {
                tc
            } else {
                let key_ref = term_key
                    .as_ref()
                    .expect("Expected variable key for term when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for term")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for term"),
                }
            };

            let r_value = if let Some(rv) = r_const {
                rv
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable key for r when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for r")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for r"),
                }
            };

            let mut violation = 0.0;
            let lhs_ne_term = left_side_term != term_value;
            if r_value != lhs_ne_term {
                if verbose {
                    let term_display = if term_key.is_none() {
                        term_value.to_string()
                    } else {
                        term_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r_value.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_lin_ne_reif {} <-> {} != {}", r_display, left_side_term, term_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_lt(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const = if a_key.is_none() {
            Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<i64> = if b_key.is_none() {
            Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution))
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
                    .expect("Expected variable in solution for A_TERM")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for A_TERM, found other type variable"),
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
                    .expect("Expected variable in solution for B_TERM")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for B_TERM, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if a_value >= b_value {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_value.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_value.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_lt {} < {}", a_display, b_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_lt_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let r_key: Option<String> = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);

        let a_const: Option<i64> = if a_key.is_none() {
            Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<i64> = if b_key.is_none() {
            Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let r_const: Option<bool> = if r_key.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key
                    .as_ref()
                    .expect("Expected variable key for A_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for A_TERM")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for A_TERM, found other type variable"),
                }
            };

            let b = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key
                    .as_ref()
                    .expect("Expected variable key for B_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for B_TERM")
                {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for B_TERM, found other type variable"),
                }
            };

            let r = if let Some(rv) = r_const {
                rv
            } else {
                let key_ref = r_key
                    .as_ref()
                    .expect("Expected variable key for R_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for R_TERM")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for R_TERM, found other type variable"),
                }
            };

            let mut violation = 0.0;
            if r != (a < b) {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_lt_reif {} <-> {} < {}", r_display, a_display, b_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_max(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<i64> = if a_key.is_none() {
            Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution))
        } else { None };
        let b_const: Option<i64> = if b_key.is_none() {
            Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution))
        } else { None };
        let c_const: Option<i64> = if c_key.is_none() {
            Some(self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, solution))
        } else { None };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key.as_ref().expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for A_TERM") {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for A_TERM, found other type variable"),
                }
            };
            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key.as_ref().expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for B_TERM") {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for B_TERM, found other type variable"),
                }
            };
            let c_value = if let Some(cv) = c_const {
                cv
            } else {
                let key_ref = c_key.as_ref().expect("Expected variable key for C_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for C_TERM") {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for C_TERM, found other type variable"),
                }
            };

            let max_val = a_value.max(b_value);
            let mut violation = 0.0;
            if c_value != max_val {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_value.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_value.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let c_display = if c_key.is_none() {
                        c_value.to_string()
                    } else {
                        c_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_max max({},{}) = {}", a_display, b_display, c_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_min(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<i64> = if a_key.is_none() {
            Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution))
        } else { None };
        let b_const: Option<i64> = if b_key.is_none() {
            Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution))
        } else { None };
        let c_const: Option<i64> = if c_key.is_none() {
            Some(self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, solution))
        } else { None };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const {
                av
            } else {
                let key_ref = a_key.as_ref().expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for A_TERM") {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for A_TERM, found other type variable"),
                }
            };
            let b_value = if let Some(bv) = b_const {
                bv
            } else {
                let key_ref = b_key.as_ref().expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for B_TERM") {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for B_TERM, found other type variable"),
                }
            };
            let c_value = if let Some(cv) = c_const {
                cv
            } else {
                let key_ref = c_key.as_ref().expect("Expected variable key for C_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for C_TERM") {
                    VariableValue::Int(i) => *i,
                    _ => panic!("Expected int variable for C_TERM, found other type variable"),
                }
            };

            let min_val = a_value.min(b_value);
            let mut violation = 0.0;
            if c_value != min_val {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_value.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_value.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let c_display = if c_key.is_none() {
                        c_value.to_string()
                    } else {
                        c_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_min min({},{}) = {}", a_display, b_display, c_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_mod(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<i64> = if a_key.is_none() { Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution)) } else { None };
        let b_const: Option<i64> = if b_key.is_none() { Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution)) } else { None };
        let c_const: Option<i64> = if c_key.is_none() { Some(self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, solution)) } else { None };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const { av } else {
                let key_ref = a_key.as_ref().expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for A_TERM") { VariableValue::Int(i) => *i, _ => panic!("Expected int variable for A_TERM, found other type variable") }
            };
            let b_value = if let Some(bv) = b_const { bv } else {
                let key_ref = b_key.as_ref().expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for B_TERM") { VariableValue::Int(i) => *i, _ => panic!("Expected int variable for B_TERM, found other type variable") }
            };
            let c_value = if let Some(cv) = c_const { cv } else {
                let key_ref = c_key.as_ref().expect("Expected variable key for C_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for C_TERM") { VariableValue::Int(i) => *i, _ => panic!("Expected int variable for C_TERM, found other type variable") }
            };

            let real_mod = a_value % b_value;
            let mut violation = 0.0;
            if c_value != real_mod {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_value.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_value.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let c_display = if c_key.is_none() {
                        c_value.to_string()
                    } else {
                        c_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_mod {} mod {} = {}", a_display, b_display, c_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_ne(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let a_const: Option<i64> = if a_key.is_none() { Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution)) } else { None };
        let b_const: Option<i64> = if b_key.is_none() { Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution)) } else { None };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a_value = if let Some(av) = a_const { av } else {
                let key_ref = a_key.as_ref().expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for A_TERM") { VariableValue::Int(i) => *i, _ => panic!("Expected int variable for A_TERM, found other type variable") }
            };
            let b_value = if let Some(bv) = b_const { bv } else {
                let key_ref = b_key.as_ref().expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for B_TERM") { VariableValue::Int(i) => *i, _ => panic!("Expected int variable for B_TERM, found other type variable") }
            };

            let mut violation = 0.0;
            if a_value == b_value {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_value.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_value.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_ne {} != {}", a_display, b_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns 1.0 if violated, 0.0 otherwise.
    pub fn int_ne_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let r_key: Option<String> = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);

        let a_const: Option<i64> = if a_key.is_none() { Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution)) } else { None };
        let b_const: Option<i64> = if b_key.is_none() { Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution)) } else { None };
        let r_const: Option<bool> = if r_key.is_none() { Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution)) } else { None };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a = if let Some(av) = a_const { av } else {
                let key_ref = a_key.as_ref().expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for A_TERM") { VariableValue::Int(i) => *i, _ => panic!("Expected int variable for A_TERM, found other type variable") }
            };
            let b = if let Some(bv) = b_const { bv } else {
                let key_ref = b_key.as_ref().expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for B_TERM") { VariableValue::Int(i) => *i, _ => panic!("Expected int variable for B_TERM, found other type variable") }
            };
            let r = if let Some(rv) = r_const { rv } else {
                let key_ref = r_key.as_ref().expect("Expected variable key for R_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for R_TERM") { VariableValue::Bool(b) => *b, _ => panic!("Expected bool variable for R_TERM, found other type variable") }
            };

            let mut violation = 0.0;
            if r != (a != b) {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let r_display = if r_key.is_none() {
                        r.to_string()
                    } else {
                        r_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_ne_reif {} <-> {} != {}", r_display, a_display, b_display);
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
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_pow(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<i64> = if a_key.is_none() { Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution)) } else { None };
        let b_const: Option<i64> = if b_key.is_none() { Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution)) } else { None };
        let c_const: Option<i64> = if c_key.is_none() { Some(self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, solution)) } else { None };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a = if let Some(av) = a_const { av } else {
                let key_ref = a_key.as_ref().expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for A_TERM") { VariableValue::Int(i) => *i, _ => panic!("Expected int variable for A_TERM, found other type variable") }
            };
            let b = if let Some(bv) = b_const { bv } else {
                let key_ref = b_key.as_ref().expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for B_TERM") { VariableValue::Int(i) => *i, _ => panic!("Expected int variable for B_TERM, found other type variable") }
            };
            let c = if let Some(cv) = c_const { cv } else {
                let key_ref = c_key.as_ref().expect("Expected variable key for C_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for C_TERM") { VariableValue::Int(i) => *i, _ => panic!("Expected int variable for C_TERM, found other type variable") }
            };

            let result = a.pow(b as u32);
            let mut violation = 0.0;
            if c != result {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let c_display = if c_key.is_none() {
                        c.to_string()
                    } else {
                        c_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_pow {} ^ {} = {}", a_display, b_display, c_display);
                }
                violation = ((c - result).abs()) as f64;
            }
            violation
        })
    }

    /// Returns a functional evaluator for the `int_times` constraint.
    ///
    /// # Arguments
    /// * `constraint` - The constraint call with defines.
    /// * `solution` - The current solution map.
    /// # Returns
    /// A closure that evaluates the constraint and returns the absolute difference if violated, 0.0 otherwise.
    pub fn int_times(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<i64> = if a_key.is_none() { Some(self.args_extractor.extract_int_value(A_TERM_INDEX, &constraint.call, solution)) } else { None };
        let b_const: Option<i64> = if b_key.is_none() { Some(self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, solution)) } else { None };
        let c_const: Option<i64> = if c_key.is_none() { Some(self.args_extractor.extract_int_value(C_TERM_INDEX, &constraint.call, solution)) } else { None };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let a = if let Some(av) = a_const { av } else {
                let key_ref = a_key.as_ref().expect("Expected variable key for A_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for A_TERM") { VariableValue::Int(i) => *i, _ => panic!("Expected int variable for A_TERM, found other type variable") }
            };
            let b = if let Some(bv) = b_const { bv } else {
                let key_ref = b_key.as_ref().expect("Expected variable key for B_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for B_TERM") { VariableValue::Int(i) => *i, _ => panic!("Expected int variable for B_TERM, found other type variable") }
            };
            let c = if let Some(cv) = c_const { cv } else {
                let key_ref = c_key.as_ref().expect("Expected variable key for C_TERM when not a literal");
                match solution.get(key_ref).expect("Expected variable in solution for C_TERM") { VariableValue::Int(i) => *i, _ => panic!("Expected int variable for C_TERM, found other type variable") }
            };

            let result = a * b;
            let mut violation = 0.0;
            if c != result {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a.to_string()
                    } else {
                        a_key.as_ref().unwrap().to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b.to_string()
                    } else {
                        b_key.as_ref().unwrap().to_string()
                    };
                    let c_display = if c_key.is_none() {
                        c.to_string()
                    } else {
                        c_key.as_ref().unwrap().to_string()
                    };
                    info!("Violated constraint: int_times {} * {} = {}", a_display, b_display, c_display);
                }
                violation = ((c - result).abs()) as f64;
            }
            violation
        })
    }


    fn int_lin_left_term(
        verbose: bool,
        coeff: &Vec<i64>,
        solution: &HashMap<String, VariableValue>,
        vars_involved: &Vec<Identifier>,
        verbose_terms: &mut String,
    ) -> i64 {
        let left_side_term: i64 = coeff
            .iter()
            .zip(vars_involved.iter())
            .map(|(c, id)| {
                let var_val = solution
                    .get(id)
                    .and_then(|int_val| match int_val {
                        VariableValue::Int(int_val) => Some(*int_val),
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


    fn identifier_from_vars(
        &self,
        vars: &HashMap<i64, Identifier>,
        index: usize,
    ) -> Option<String> {
        vars.get(&(index as i64)).map(|id| id.to_string())
    }
}

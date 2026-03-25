use crate::args_extractor::sub_types::bool_args_extractor::BoolArgsExtractor;
use crate::data_utility::logger::write_verbose_output;
use crate::evaluator::evaluator::CallWithDefines;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Identifier, Call};
use log::info;
use std::{backtrace, collections::HashMap};

pub const A_TERM_INDEX: usize = 0;
pub const B_TERM_INDEX: usize = 1;
pub const C_TERM_INDEX: usize = 2;
pub const AS_ARRAY_INDEX: usize = 0;
pub const BS_ARRAY_INDEX: usize = 1;
pub const R_TERM_INDEX: usize = 1;
pub const COEFF_LIN_CONSTR_INDEX: usize = 0;

#[derive(Debug, Clone, Default)]
pub struct BoolFunctionalEvaluator {
    arrays: HashMap<Identifier, Array>,
    args_extractor: BoolArgsExtractor,
    verbose: bool,
}

impl BoolFunctionalEvaluator {
    pub fn new(arrays: HashMap<Identifier, Array>, verbose: bool) -> Self {
        let args_extractor = BoolArgsExtractor::new();

        Self {
            arrays,
            args_extractor,
            verbose,
        }
    }

    fn prepare_env(&self, constraint: &CallWithDefines) -> (BoolArgsExtractor, Call, bool) {
        (self.args_extractor.clone(), constraint.call.clone(), self.verbose)
    }

    fn prepare_env_with_arrays(&self, constraint: &CallWithDefines) -> (BoolArgsExtractor, HashMap<Identifier, Array>, Call, bool) {
        (self.args_extractor.clone(), self.arrays.clone(), constraint.call.clone(), self.verbose)
    }

    pub fn array_bool_and(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&call.args);

        let r_key: Option<String> = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);
        let r_const: Option<bool> = if r_key.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &call, solution))
        } else {
            None
        };


        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let as_array = args_extractor.extract_bool_array(AS_ARRAY_INDEX, &arrays, &call, solution);
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

            if as_array.iter().all(|&item| item) != r_value {
                if verbose {
                    let joined = as_array
                        .iter()
                        .map(|b| if *b { "true" } else { "false" })
                        .collect::<Vec<_>>()
                        .join(" /\\ ");
                    let r_name = r_key.as_deref().unwrap_or("<const>");
                    info!("Violated: {} <-> {}", joined, r_name);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn array_bool_element(
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
        let c_value: Option<bool> = if c_key.is_none() {
            Some(self.args_extractor.extract_bool_value(C_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let array_value = args_extractor.extract_bool_element_array(AS_ARRAY_INDEX, &call, &arrays, solution);
            let c_value = if let Some(cv) = c_value {
                cv
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable, found other type variable"),
                }
            };

            if array_value != c_value {
                if verbose {
                    let array_display = if array_value { "true".to_string() } else { "false".to_string() };
                    let value_display = if c_key.is_none() {
                        c_value.to_string()
                    } else {
                        c_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    info!("Violated constraint: array_bool_element {} = {}", array_display, value_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn array_bool_xor(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let verbose = self.verbose;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let array_value = args_extractor.extract_bool_array(AS_ARRAY_INDEX, &arrays, &call, solution);

            if !array_value.iter().fold(false, |acc, &item| acc ^ item) {
                if verbose {
                    let joined = array_value
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

    pub fn bool_and(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<bool> = if a_key.is_none() {
            Some(self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<bool> = if b_key.is_none() {
            Some(self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let c_const: Option<bool> = if c_key.is_none() {
            Some(self.args_extractor.extract_bool_value(C_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for A_TERM, found other type variable"),
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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for B_TERM, found other type variable"),
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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for C_TERM, found other type variable"),
                }
            };

            if c_value != (a_value && b_value) {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_const.unwrap().to_string()
                    } else {
                        a_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_const.unwrap().to_string()
                    } else {
                        b_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let c_display = if c_key.is_none() {
                        c_const.unwrap().to_string()
                    } else {
                        c_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    info!("Violated constraint: array_bool_and {} <-> {} /\\ {}", c_display, a_display, b_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn bool_clause(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let verbose = self.verbose;

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

            let as_array = args_extractor.extract_bool_array(AS_ARRAY_INDEX, &arrays, &call, solution);
            let bs_array = args_extractor.extract_bool_array(BS_ARRAY_INDEX, &arrays, &call, solution);

            let or_as_array = as_array.iter().map(|&b| b as i64).sum::<i64>();
            let or_bs_array = bs_array.iter().map(|&b| b as i64).sum::<i64>();

            let result = or_as_array + or_bs_array;

            if result == 0 {
                if verbose {
                    let mut interleaved: Vec<&str> = Vec::new();
                    let max_len = std::cmp::max(as_array.len(), bs_array.len());
                    for i in 0..max_len {
                        if let Some(a) = as_array.get(i) {
                            interleaved.push(if *a { "true" } else { "false" });
                        }
                        if let Some(b) = bs_array.get(i) {
                            interleaved.push(if *b { "not(true)" } else { "not(false)" });
                        }
                    }
                    let joined = interleaved.join(r" \/ ");
                    info!("constraint{:?}", call);
                    info!("Violated constraint: bool_clause {}", joined);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn bool_eq(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let vars_involved = self
            .args_extractor
   
         .extract_literal_identifiers_with_index(&constraint.call.args);
        let verbose = self.verbose;
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<bool> = if a_key.is_none() {
            Some(self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<bool> = if b_key.is_none() {
            Some(self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for A_TERM, found other type variable"),
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
                    VariableValue::Bool(i) => *i,
                    _ => panic!("Expected bool variable, found other type variable"),
                }
            };

            if a_value != b_value {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_const.unwrap().to_string()
                    } else {
                        a_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_const.unwrap().to_string()
                    } else {
                        b_key.as_deref().unwrap_or("<const>").to_string()
                    };  
                    info!("Violated constraint: bool_eq {} = {}", a_display, b_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn bool_eq_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<bool> = if a_key.is_none() {
            Some(self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<bool> = if b_key.is_none() {
            Some(self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let c_const: Option<bool> = if c_key.is_none() {
            Some(self.args_extractor.extract_bool_value(C_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for A_TERM, found other type variable"),
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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for B_TERM, found other type variable"),
                }
            };

            let c_value = if let Some(rv) = c_const {
                rv
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for C_TERM")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for C_TERM, found other type variable"),
                }
            };
            if c_value != (a_value == b_value) {
                if verbose {
                   let a_display = if a_key.is_none() {
                        a_const.unwrap().to_string()
                    } else {
                        a_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_const.unwrap().to_string()
                    } else {
                        b_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let c_display = if c_key.is_none() {
                        c_const.unwrap().to_string()
                    } else {
                        c_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    info!("Violated constraint: bool_eq_reif {} <-> {} = {}", c_display, a_display, b_display);       
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn bool_le(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<bool> = if a_key.is_none() {
            Some(self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<bool> = if b_key.is_none() {
            Some(self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for A_TERM, found other type variable"),
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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for B_TERM, found other type variable"),
                }
            };

            if a_value > b_value {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_const.unwrap().to_string()
                    } else {
                        a_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_const.unwrap().to_string()
                    } else {
                        b_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    info!("Violated constraint: bool_le {} <= {}", a_display, b_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn bool_le_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, R_TERM_INDEX);

        let a_const: Option<bool> = if a_key.is_none() {
            Some(self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<bool> = if b_key.is_none() {
            Some(self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let c_const: Option<bool> = if c_key.is_none() {
            Some(self.args_extractor.extract_bool_value(R_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };


        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for A_TERM, found other type variable"),
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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for B_TERM, found other type variable"),
                }
            };

            let c_value = if let Some(rv) = c_const {
                rv
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for C_TERM")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for C_TERM, found other type variable"),
                }
            };

            if c_value != (a_value <= b_value) {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_const.unwrap().to_string()
                    } else {
                        a_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_const.unwrap().to_string()
                    } else {
                        b_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let c_display = if c_key.is_none() {
                        c_const.unwrap().to_string()
                    } else {
                        c_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    info!("Violated constraint: bool_le_reif {} <-> {} <= {}", c_display, a_display, b_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn bool_lin_eq(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let verbose = self.verbose;
        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &call,
            &self.arrays,
        );
        let literal_vars_map = self.args_extractor.extract_literal_identifiers_with_index(&call.args);
        let term_key: Option<String> = self.identifier_from_vars(&literal_vars_map, C_TERM_INDEX);
        let term_const: Option<i64> = if term_key.is_none() {
            Some(self.args_extractor.extract_int_constant_term_lin_expr( &call))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;
            let bs_array = args_extractor.extract_bool_array(BS_ARRAY_INDEX, &arrays, &call, solution);
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


            let mut verbose_terms = String::new();

            let left_side_term: i64 = coeff
                .iter()
                .zip(bs_array.iter())
                .map(|(c, b)| {
                    let val = if *b { 1_i64 } else { 0_i64 };
                    if verbose {
                        write_verbose_output(&mut verbose_terms, c, &val);
                    }
                    c * val
                })
                .sum();

            if left_side_term != term_value {
                if verbose {
                    let term_display = if term_key.is_none() {
                        term_const.unwrap().to_string()
                    } else {
                        term_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    info!("Violated constraint: bool_lin_eq {} = {}", left_side_term, term_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn bool_lin_le(
        &self,
        constraint: &CallWithDefines,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
       let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let verbose = self.verbose;
        let coeff = self.args_extractor.extract_int_coefficients_lin_expr(
            COEFF_LIN_CONSTR_INDEX,
            &call,
            &self.arrays,
        );
        let literal_vars_map = self.args_extractor.extract_literal_identifiers_with_index(&call.args);
        let term_key: Option<String> = self.identifier_from_vars(&literal_vars_map, C_TERM_INDEX);
        let term_const: Option<i64> = if term_key.is_none() {
            Some(self.args_extractor.extract_int_constant_term_lin_expr( &call))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;
            let bs_array = args_extractor.extract_bool_array(BS_ARRAY_INDEX, &arrays, &call, solution);
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

            let mut verbose_terms = String::new();

            let left_side_term: i64 = coeff
                .iter()
                .zip(bs_array.iter())
                .map(|(c, b)| {
                    let val = if *b { 1_i64 } else { 0_i64 };
                    if verbose {
                        write_verbose_output(&mut verbose_terms, c, &val);
                    }
                    c * val
                })
                .sum();

            if left_side_term > term_value {
                if verbose {
                    let term_display = if term_key.is_none() {
                        term_const.unwrap().to_string()
                    } else {
                        term_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    info!("Violated constraint: bool_lin_le {} <= {}", verbose_terms, term_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn bool_lt(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
              let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<bool> = if a_key.is_none() {
            Some(self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<bool> = if b_key.is_none() {
            Some(self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };

        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for A_TERM, found other type variable"),
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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for B_TERM, found other type variable"),
                }
            };

            if a_value >= b_value {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_const.unwrap().to_string()
                    } else {
                        a_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_const.unwrap().to_string()
                    } else {
                        b_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    info!("Violated constraint: bool_lt {} < {}", a_display, b_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn bool_lt_reif(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<bool> = if a_key.is_none() {
            Some(self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<bool> = if b_key.is_none() {
            Some(self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let c_const: Option<bool> = if c_key.is_none() {
            Some(self.args_extractor.extract_bool_value(C_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };


        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for A_TERM, found other type variable"),
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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for B_TERM, found other type variable"),
                }
            };

            let c_value = if let Some(rv) = c_const {
                rv
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for C_TERM")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for C_TERM, found other type variable"),
                }
            };

            if c_value != (a_value < b_value) {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_const.unwrap().to_string()
                    } else {
                        a_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_const.unwrap().to_string()
                    } else {
                        b_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let c_display = if c_key.is_none() {
                        c_const.unwrap().to_string()
                    } else {
                        c_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    info!("Violated constraint: bool_lt_reif {} <-> {} < {}", c_display, a_display, b_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn bool_not(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<bool> = if a_key.is_none() {
            Some(self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let b_const: Option<bool> = if b_key.is_none() {
            Some(self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };


        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for A_TERM, found other type variable"),
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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for B_TERM, found other type variable"),
                }
            };

            if a_value == b_value {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_const.unwrap().to_string()
                    } else {
                        a_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_const.unwrap().to_string()
                    } else {
                        b_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    info!("Violated constraint: bool_not {} = not({})", a_display, b_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn bool_or(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<bool> = if a_key.is_none() {
            Some(self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<bool> = if b_key.is_none() {
            Some(self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let c_const: Option<bool> = if c_key.is_none() {
            Some(self.args_extractor.extract_bool_value(C_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };


        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for A_TERM, found other type variable"),
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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for B_TERM, found other type variable"),
                }
            };

            let c_value = if let Some(rv) = c_const {
                rv
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for C_TERM")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for C_TERM, found other type variable"),
                }
            };

            if c_value != (a_value || b_value) {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_const.unwrap().to_string()
                    } else {
                        a_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_const.unwrap().to_string()
                    } else {
                        b_key.as_deref().unwrap_or("<const>").to_string()
                    };

                    let c_display = if c_key.is_none() {
                        c_const.unwrap().to_string()
                    } else {
                        c_key.as_deref().unwrap_or("<const>").to_string()
                    };

                    info!(r"Violated constraint: bool_or {} <-> {} \/ {}", c_display, a_display, b_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn bool_xor(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;

        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self.identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let b_key: Option<String> = self.identifier_from_vars(&vars_involved, B_TERM_INDEX);
        let c_key: Option<String> = self.identifier_from_vars(&vars_involved, C_TERM_INDEX);

        let a_const: Option<bool> = if a_key.is_none() {
            Some(self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let b_const: Option<bool> = if b_key.is_none() {
            Some(self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };
        let c_const: Option<bool> = if c_key.is_none() {
            Some(self.args_extractor.extract_bool_value(C_TERM_INDEX, &constraint.call, solution))
        } else {
            None
        };


        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let mut violation = 0.0;

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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for A_TERM, found other type variable"),
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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for B_TERM, found other type variable"),
                }
            };

            let c_value = if let Some(rv) = c_const {
                rv
            } else {
                let key_ref = c_key
                    .as_ref()
                    .expect("Expected variable key for C_TERM when not a literal");
                match solution
                    .get(key_ref)
                    .expect("Expected variable in solution for C_TERM")
                {
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for C_TERM, found other type variable"),
                }
            };

            if c_value != (a_value ^ b_value) {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_const.unwrap().to_string()
                    } else {
                        a_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_const.unwrap().to_string()
                    } else {
                        b_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let c_display = if c_key.is_none() {
                        c_const.unwrap().to_string()
                    } else {
                        c_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    info!("Violated constraint: bool_xor {} <-> {} xor {}", c_display, a_display, b_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    pub fn bool2int(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        let verbose = self.verbose;
        let vars_involved = self.args_extractor.extract_literal_identifiers_with_index(&constraint.call.args);
        let a_key: Option<String> = self
            .identifier_from_vars(&vars_involved, A_TERM_INDEX);
        let a_const: Option<bool> = if a_key.is_none() {
            Some(self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, solution))
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
            let mut violation = 0.0;

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
                    VariableValue::Bool(b) => *b,
                    _ => panic!("Expected bool variable for A_TERM, found other type variable"),
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
                    _ => panic!("Expected int variable for B_TERM, found other type variable"),
                }
            };

            if a_value as i64 != b_value {
                if verbose {
                    let a_display = if a_key.is_none() {
                        a_const.unwrap().to_string()
                    } else {
                        a_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    let b_display = if b_key.is_none() {
                        b_const.unwrap().to_string()
                    } else {
                        b_key.as_deref().unwrap_or("<const>").to_string()
                    };
                    info!("Violated constraint: bool2int {} = {}", a_display, b_display);
                }
                violation = 1.0;
            }

            violation
        })
    }

    fn int_lin_left_term(
        &self,
        coeff: &[i64],
        bs_array: &[bool],
        verbose_terms: &mut String,
    ) -> i64 {
        let left_side_term: i64 = coeff
            .iter()
            .zip(bs_array.iter())
            .map(|(c, b)| {
                let val = if *b { 1_i64 } else { 0_i64 };
                if self.verbose {
                    write_verbose_output(verbose_terms, c, &val);
                }
                c * val
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

use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Identifier};
use std::collections::{HashMap, HashSet};
use crate::args_extractor::args_extractor::ArgsExtractor;
use crate::evaluator::evaluator::CallWithDefines;
use crate::variable_assigner::sub_types::bool_variable_assigner::BoolVariableAssigner;
use crate::variable_assigner::sub_types::float_variable_assigner::FloatVariableAssigner;
use crate::variable_assigner::sub_types::int_variable_assigner::IntVariableAssigner;
use crate::variable_assigner::sub_types::set_variable_assigner::SetVariableAssigner;

#[derive(Debug, Clone, Default)]
pub struct VariableAssigner {
    defined_variable: Vec<String>,
    constraints: Vec<CallWithDefines>,
    args_extractor: ArgsExtractor,
    complete_solution: HashMap<String, VariableValue>,
    arrays: HashMap<Identifier, Array>,
    int_variable_assigner: IntVariableAssigner,
    float_variable_assigner: FloatVariableAssigner,
    bool_variable_assigner: BoolVariableAssigner,
    set_variable_assigner: SetVariableAssigner,
}
impl VariableAssigner {
    pub fn new(
        partial_solution: HashMap<String, VariableValue>,
        defined_variable: Vec<String>,
        constraints: Vec<CallWithDefines>,
        arrays: HashMap<Identifier, Array>,
    ) -> Self {
        let complete_solution = partial_solution.clone();
        let int_variable_assigner = IntVariableAssigner::new(arrays.clone());
        let float_variable_assigner = FloatVariableAssigner::new(arrays.clone());
        let bool_variable_assigner = BoolVariableAssigner::new(arrays.clone());
        let set_variable_assigner = SetVariableAssigner::new(arrays.clone());
        let args_extractor = ArgsExtractor::new();

        Self {
            defined_variable,
            constraints,
            args_extractor,
            complete_solution,
            arrays,
            int_variable_assigner,
            float_variable_assigner,
            bool_variable_assigner,
            set_variable_assigner,
        }
    }

    pub fn search_defined_var_in_constraints(&mut self) -> HashMap<String, VariableValue> {
        let mut unknow = self.defined_variable.clone(); 
        let array_keys: HashSet<String> = self
            .arrays
            .keys()
            .cloned()
            .collect();
        
        let mut constraint_deps: Vec<(usize, Option<String>, HashSet<String>)> = Vec::new();
        for (idx, cw) in self.constraints.iter().enumerate() {
            if let Some(def) = cw.defines.as_ref() {
                let deps = self
                    .args_extractor
                    .extract_literal_identifiers(&cw.call.args)
                    .into_iter()
                    .collect::<HashSet<_>>();
                constraint_deps.push((idx, Some(def.clone()), deps));
            }
        }

        while !unknow.is_empty() {
            let mut ready: Vec<(usize, String)> = Vec::new();
            
            for (idx, defined_var, deps) in &constraint_deps {
                if let Some(def) = defined_var {
                    if unknow.contains(def) {
                        let dependencies_ready = deps.iter().all(|id| {
                            id == def
                                || self.complete_solution.contains_key(id)
                                || (array_keys.contains(id) && self.array_is_ready(id, def))
                        });

                        if dependencies_ready && !self.complete_solution.contains_key(def) {
                            ready.push((*idx, def.clone()));
                        }
                    }
                }
            }

            if ready.is_empty() {
                break;
            }

            for (idx, var) in ready {
                let cw = self.constraints[idx].clone();
                let assigned = self.assign_value(&cw, &var);
                if assigned {
                    unknow.retain(|v| v != &var);
                }
            }
        }
        self.complete_solution.clone()
    }

    fn array_is_ready(&self, array_id: &String, defined_var: &str) -> bool {
         if let Some(array) = self.arrays.get(array_id) {
                for element in &array.contents {
                 match element {
                    flatzinc_serde::Literal::Int(_) => continue,
                    flatzinc_serde::Literal::Float(_) => continue,
                    flatzinc_serde::Literal::Identifier(id) => 
                    if id == defined_var {
                        continue;
                    } else if !self.complete_solution.contains_key(id) {
                        return false;
                    },
                    _ => continue
                    }
                }
                true
          } else {
                false
          }
    }

    fn assign_value(&mut self, constraint: &CallWithDefines, variable: &String) -> bool {
        let id = constraint.call.id.as_str();
        if id == "array_int_element" || id == "array_var_int_element" || id.starts_with("int_") {
            self.assign_value_int(constraint, variable);
        } else if id == "array_float_element"
            || id == "array_var_float_element"
            || id.starts_with("float_")
            || id == "int2float"
        {
            self.assign_value_float(constraint, variable);
        } else if id == "array_bool_element"
            || id == "array_var_bool_element"
            || id.starts_with("bool_")
            || id.starts_with("array_bool_")
            || id == "bool2int"
        {
            self.assign_value_bool(constraint, variable);
        } else if id == "array_set_element"
            || id == "array_var_set_element"
            || id.starts_with("set_")
        {
            self.assign_value_set(constraint, variable);
        } else {
            panic!("Missing predicate: {}", id);
        }

        self.complete_solution.contains_key(variable)
    }

    fn assign_value_int(&mut self, constraint: &CallWithDefines, variable: &String) {
        match constraint.call.id.as_str() {
            "array_int_element" => {
                let value = self
                    .int_variable_assigner
                    .array_int_element(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            "array_var_int_element" => {
                let value = self
                    .int_variable_assigner
                    .array_int_element(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            "int_abs" => {
                let value = self
                    .int_variable_assigner
                    .int_abs(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            "int_div" => {
                let value = self
                    .int_variable_assigner
                    .int_div(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            "float_eq" => {
                let value = self
                    .int_variable_assigner
                    .int_eq(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            "int_eq_reif" => {
                let value = self
                    .int_variable_assigner
                    .int_eq_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "int_le_reif" => {
                let value = self
                    .int_variable_assigner
                    .int_le_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "int_lin_eq" => {
                let value = self.int_variable_assigner.int_lin_eq(
                    &constraint,
                    &self.complete_solution,
                    variable,
                );
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            "int_lin_eq_reif" => {
                let value = self
                    .int_variable_assigner
                    .int_lin_eq_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "int_lin_le_reif" => {
                let value = self
                    .int_variable_assigner
                    .int_lin_le_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "int_lin_ne_reif" => {
                let value = self
                    .int_variable_assigner
                    .int_lin_ne_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "int_lt_reif" => {
                let value = self
                    .int_variable_assigner
                    .int_lt_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "int_max" => {
                let value = self
                    .int_variable_assigner
                    .int_max(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            "int_min" => {
                let value = self
                    .int_variable_assigner
                    .int_min(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            "int_mod" => {
                let value = self
                    .int_variable_assigner
                    .int_mod(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            "int_ne_reif" => {
                let value = self
                    .int_variable_assigner
                    .int_ne_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "int_plus" => {
                let value = self
                    .int_variable_assigner
                    .int_plus(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            "int_pow" => {
                let value = self
                    .int_variable_assigner
                    .int_pow(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            "int_minus" => {
                let value = self
                    .int_variable_assigner
                    .int_minus(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            "int_times" => {
                let value = self
                    .int_variable_assigner
                    .int_times(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            _ => panic!(
                "Not assignment found for constraint {}",
                constraint.call.id.as_str()
            ),
        }
    }

    fn assign_value_float(&mut self, constraint: &CallWithDefines, variable: &String) {
        match constraint.call.id.as_str() {
            "array_float_element" => {
                let value = self
                    .float_variable_assigner
                    .array_float_element(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "array_var_float_element" => {
                let value = self
                    .float_variable_assigner
                    .array_float_element(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_abs" => {
                let value = self
                    .float_variable_assigner
                    .float_abs(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_acos" => {
                let value = self
                    .float_variable_assigner
                    .float_acos(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_acosh" => {
                let value = self
                    .float_variable_assigner
                    .float_acosh(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_asin" => {
                let value = self
                    .float_variable_assigner
                    .float_asin(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_asinh" => {
                let value = self
                    .float_variable_assigner
                    .float_asinh(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_atan" => {
                let value = self
                    .float_variable_assigner
                    .float_atan(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_atanh" => {
                let value = self
                    .float_variable_assigner
                    .float_atanh(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_cos" => {
                let value = self
                    .float_variable_assigner
                    .float_cos(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_cosh" => {
                let value = self
                    .float_variable_assigner
                    .float_cosh(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_div" => {
                let value = self
                    .float_variable_assigner
                    .float_div(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_eq" => {
                let value = self
                    .float_variable_assigner
                    .float_eq(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_eq_reif" => {
                let value = self
                    .float_variable_assigner
                    .float_eq_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "float_exp" => {
                let value = self
                    .float_variable_assigner
                    .float_exp(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_le_reif" => {
                let value = self
                    .float_variable_assigner
                    .float_le_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "float_lin_eq" => {
                let value = self.float_variable_assigner.float_lin_eq(
                    &constraint,
                    &self.complete_solution,
                    variable,
                );
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_lin_eq_reif" => {
                let value = self
                    .float_variable_assigner
                    .float_lin_eq_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "float_lin_le_reif" => {
                let value = self
                    .float_variable_assigner
                    .float_lin_le_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "float_lin_lt_reif" => {
                let value = self
                    .float_variable_assigner
                    .float_lin_lt_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "float_lin_ne_reif" => {
                let value = self
                    .float_variable_assigner
                    .float_lin_ne_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "float_ln" => {
                let value = self
                    .float_variable_assigner
                    .float_ln(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_log10" => {
                let value = self
                    .float_variable_assigner
                    .float_log10(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_log2" => {
                let value = self
                    .float_variable_assigner
                    .float_log2(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_lt_reif" => {
                let value = self
                    .float_variable_assigner
                    .float_lt_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "float_max" => {
                let value = self
                    .float_variable_assigner
                    .float_max(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_min" => {
                let value = self
                    .float_variable_assigner
                    .float_min(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_ne_reif" => {
                let value = self
                    .float_variable_assigner
                    .float_ne_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "float_plus" => {
                let value = self
                    .float_variable_assigner
                    .float_plus(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_pow" => {
                let value = self
                    .float_variable_assigner
                    .float_pow(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_sin" => {
                let value = self
                    .float_variable_assigner
                    .float_sin(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_sinh" => {
                let value = self
                    .float_variable_assigner
                    .float_sinh(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_sqrt" => {
                let value = self
                    .float_variable_assigner
                    .float_sqrt(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_tan" => {
                let value = self
                    .float_variable_assigner
                    .float_tan(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_tanh" => {
                let value = self
                    .float_variable_assigner
                    .float_tanh(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "float_times" => {
                let value = self
                    .float_variable_assigner
                    .float_times(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            "int2float" => {
                let value = self
                    .float_variable_assigner
                    .int2float(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Float(value));
            }
            _ => panic!(
                "Not assignment found for constraint {}",
                constraint.call.id.as_str()
            ),
        }
    }

    fn assign_value_bool(&mut self, constraint: &CallWithDefines, variable: &String) {
        match constraint.call.id.as_str() {
            "array_bool_and" => {
                let value = self
                    .bool_variable_assigner
                    .array_bool_and(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "array_bool_element" => {
                let value = self
                    .bool_variable_assigner
                    .array_bool_element(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "array_var_bool_element" => {
                let value = self
                    .bool_variable_assigner
                    .array_bool_element(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "bool_and" => {
                let value = self
                    .bool_variable_assigner
                    .bool_and(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "bool_clause" => {
                let value = self
                    .bool_variable_assigner
                    .bool_clause(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "bool_eq" => {
                let value = self
                    .bool_variable_assigner
                    .bool_eq(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "bool_not" => {
                let value = self
                    .bool_variable_assigner
                    .bool_not(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "bool_eq_reif" => {
                let value = self
                    .bool_variable_assigner
                    .bool_eq_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "bool_le_reif" => {
                let value = self
                    .bool_variable_assigner
                    .bool_le_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "bool_lt_reif" => {
                let value = self
                    .bool_variable_assigner
                    .bool_lt_reif(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "bool_or" => {
                let value = self
                    .bool_variable_assigner
                    .bool_or(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "bool_xor" => {
                let value = self
                    .bool_variable_assigner
                    .bool_xor(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "bool2int" => {
                let value = self
                    .bool_variable_assigner
                    .bool2int(&constraint, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Int(value));
            }
            _ => panic!(
                "Not assignment found for constraint {}",
                constraint.call.id.as_str()
            ),
        }
    }

    fn assign_value_set(&mut self, constraint: &CallWithDefines, variable: &String) {
        match constraint.call.id.as_str() {
            "array_set_element" => {
                let value = self
                    .set_variable_assigner
                    .array_set_element(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Set(value));
            }
            "array_var_set_element" => {
                let value = self
                    .set_variable_assigner
                    .array_set_element(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Set(value));
            }
            "set_diff" => {
                let value = self
                    .set_variable_assigner
                    .set_diff(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Set(value));
            }
            "set_eq" => {
                let value = self
                    .set_variable_assigner
                    .set_eq(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Set(value));
            }
            "set_eq_reif" => {
                let value = self
                    .set_variable_assigner
                    .set_eq_reif(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "set_in_reif" => {
                let value = self
                    .set_variable_assigner
                    .set_in_reif(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "set_intersect" => {
                let value = self
                    .set_variable_assigner
                    .set_intersect(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Set(value));
            }
            "set_le_reif" => {
                let value = self
                    .set_variable_assigner
                    .set_le_reif(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "set_lt_reif" => {
                let value = self
                    .set_variable_assigner
                    .set_lt_reif(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "set_ne_reif" => {
                let value = self
                    .set_variable_assigner
                    .set_ne_reif(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "set_subset_reif" => {
                let value = self
                    .set_variable_assigner
                    .set_subset_reif(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "set_superset_reif" => {
                let value = self
                    .set_variable_assigner
                    .set_superset_reif(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Bool(value));
            }
            "set_symmetric_difference" => {
                let value = self
                    .set_variable_assigner
                    .set_symmetric_difference(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Set(value));
            }
            "set_union" => {
                let value = self
                    .set_variable_assigner
                    .set_union(&constraint.call, &self.complete_solution);
                self.complete_solution
                    .insert(variable.to_string(), VariableValue::Set(value));
            }
            _ => {}
        }
    }
}

use crate::args_extractor::args_extractor::ArgsExtractor;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::solution_provider::VariableValue;
use crate::variable_assigner::sub_types::bool_variable_assigner::BoolVariableAssigner;
use crate::variable_assigner::sub_types::float_variable_assigner::FloatVariableAssigner;
use crate::variable_assigner::sub_types::int_variable_assigner::IntVariableAssigner;
use crate::variable_assigner::sub_types::set_variable_assigner::SetVariableAssigner;

use flatzinc_serde::{Array, Identifier};

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

type AssignFn = Arc<dyn Fn(&HashMap<String, VariableValue>) -> VariableValue + Send + Sync>;

#[derive(Clone, Default)]
pub struct VariableAssigner {
    defined_variable: Vec<String>,
    constraints: Vec<CallWithDefines>,
    assigned_functions: Vec<(String, AssignFn)>,
    args_extractor: ArgsExtractor,
    complete_solution: HashMap<String, VariableValue>,
    arrays: HashMap<Identifier, Array>,
    int_variable_assigner: IntVariableAssigner,
    float_variable_assigner: FloatVariableAssigner,
    bool_variable_assigner: BoolVariableAssigner,
    set_variable_assigner: SetVariableAssigner,
}

impl fmt::Debug for VariableAssigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VariableAssigner")
            .field("defined_variable", &self.defined_variable)
            .field("constraints", &self.constraints)
            .field("assigned_functions", &self.assigned_functions.iter().map(|(var, _)| var).collect::<Vec<_>>())
            .field("args_extractor", &self.args_extractor)
            .field("complete_solution", &self.complete_solution)
            .field("arrays", &self.arrays)
            .field("int_variable_assigner", &self.int_variable_assigner)
            .field("float_variable_assigner", &self.float_variable_assigner)
            .field("bool_variable_assigner", &self.bool_variable_assigner)
            .field("set_variable_assigner", &self.set_variable_assigner)
            .finish()
    }
}

impl VariableAssigner {
    pub fn new(
        defined_variable: Vec<String>,
        constraints: Vec<CallWithDefines>,
        arrays: HashMap<Identifier, Array>,
    ) -> Self {
        Self {
            defined_variable,
            constraints,
            assigned_functions: Vec::new(),
            args_extractor: ArgsExtractor::new(),
            complete_solution: HashMap::new(),
            arrays: arrays.clone(),
            int_variable_assigner: IntVariableAssigner::new(arrays.clone()),
            float_variable_assigner: FloatVariableAssigner::new(arrays.clone()),
            bool_variable_assigner: BoolVariableAssigner::new(arrays.clone()),
            set_variable_assigner: SetVariableAssigner::new(arrays),
        }
    }

    pub fn assign_defined_variables(
        &mut self,
        partial_solution: &HashMap<String, VariableValue>,
    ) -> HashMap<String, VariableValue> {
        self.complete_solution = partial_solution.clone();

        if self.assigned_functions.is_empty() {
            self.build_assigned_functions();
        }

        for (var, func) in &self.assigned_functions {
            if !self.complete_solution.contains_key(var) {
                let value = func(&self.complete_solution);
                self.complete_solution.insert(var.clone(), value);
            }
        }

        self.complete_solution.clone()
    }

    fn build_assigned_functions(&mut self) {
        for constraint in &self.constraints {
            let Some(var) = constraint.defines.clone() else {
                continue;
            };

            let closure = self.build_closure(constraint, &var);
            self.assigned_functions.push((var, closure));
        }
    }

    fn build_closure(&self, constraint: &CallWithDefines, variable: &String) -> AssignFn {
        let id = constraint.call.id.as_str();
        if id.ends_with("_reif") {
            return self.wrap_bool(constraint, variable);
        }
        if id == "bool2int" {
            return self.wrap_int(constraint, variable);
        }
        if id == "int2float" {
            return self.wrap_float(constraint, variable);
        }

        if id.starts_with("int_") || id.contains("array_int") || id.contains("array_var_int") {
            self.wrap_int(constraint, variable)
        } else if id.starts_with("float_") || id.contains("array_float") || id.contains("array_var_float"){
            self.wrap_float(constraint, variable)
        } else if id.starts_with("bool_") || id.contains("array_bool") || id.contains("array_var_bool") {
            self.wrap_bool(constraint, variable)
        } else if id.starts_with("set_") || id.contains("array_set") || id.contains("array_var_set") {
            self.wrap_set(constraint, variable)
        } else {
            panic!("Unknown predicate: {}", id);
        }
    }

    fn wrap_int(&self, constraint: &CallWithDefines, variable: &String) -> AssignFn {
        let f = self.build_int_closure(constraint, variable);
        Arc::new(move |sol| VariableValue::Int(f(sol)))
    }

    fn wrap_float(&self, constraint: &CallWithDefines, variable: &String) -> AssignFn {
        let f = self.build_float_closure(constraint, variable);
        Arc::new(move |sol| VariableValue::Float(f(sol)))
    }

    fn wrap_bool(&self, constraint: &CallWithDefines, variable: &String) -> AssignFn {
        let f = self.build_bool_closure(constraint, variable);
        Arc::new(move |sol| VariableValue::Bool(f(sol)))
    }

    fn wrap_set(&self, constraint: &CallWithDefines, variable: &String) -> AssignFn {
        let f = self.build_set_closure(constraint, variable);
        Arc::new(move |sol| VariableValue::Set(f(sol)))
    }

    fn build_int_closure(
        &self,
        constraint: &CallWithDefines,
        variable: &String,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> i64 + Send + Sync> {
        match constraint.call.id.as_str() {
            "array_int_element" | "array_var_int_element" => self
                .int_variable_assigner
                .array_int_element(constraint, &self.complete_solution),
            "int_abs" => self
                .int_variable_assigner
                .int_abs(constraint, &self.complete_solution),
            "int_div" => self
                .int_variable_assigner
                .int_div(constraint, &self.complete_solution),
            "int_eq" => self
                .int_variable_assigner
                .int_eq(constraint, &self.complete_solution),
            "int_lin_eq" => {
                self.int_variable_assigner
                    .int_lin_eq(constraint, &self.complete_solution, variable)
            }
            "int_max" => self
                .int_variable_assigner
                .int_max(constraint, &self.complete_solution),
            "int_min" => self
                .int_variable_assigner
                .int_min(constraint, &self.complete_solution),
            "int_mod" => self
                .int_variable_assigner
                .int_mod(constraint, &self.complete_solution),
            "int_pow" => self
                .int_variable_assigner
                .int_pow(constraint, &self.complete_solution),
            "int_times" => self
                .int_variable_assigner
                .int_times(constraint, &self.complete_solution),
            "bool2int" => self
                .bool_variable_assigner
                .bool2int(constraint, &self.complete_solution),
            _ => panic!("Unhandled int constraint {}", constraint.call.id),
        }
    }

    fn build_float_closure(
        &self,
        constraint: &CallWithDefines,
        variable: &String,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> {
        match constraint.call.id.as_str() {
            "array_float_element" | "array_var_float_element" => self
                .float_variable_assigner
                .array_float_element(constraint, &self.complete_solution),
            "float_abs" => self
                .float_variable_assigner
                .float_abs(constraint, &self.complete_solution),
            "float_div" => self
                .float_variable_assigner
                .float_div(constraint, &self.complete_solution),
            "float_eq" => self
                .float_variable_assigner
                .float_eq(constraint, &self.complete_solution),
            "float_max" => self
                .float_variable_assigner
                .float_max(constraint, &self.complete_solution),
            "float_min" => self
                .float_variable_assigner
                .float_min(constraint, &self.complete_solution),
            "float_plus" => self
                .float_variable_assigner
                .float_plus(constraint, &self.complete_solution),
            "float_pow" => self
                .float_variable_assigner
                .float_pow(constraint, &self.complete_solution),
            "float_times" => self
                .float_variable_assigner
                .float_times(constraint, &self.complete_solution),
            "float_acos" => self
                .float_variable_assigner
                .float_acos(constraint, &self.complete_solution),
            "float_acosh" => self
                .float_variable_assigner
                .float_acosh(constraint, &self.complete_solution),
            "float_asin" => self
                .float_variable_assigner
                .float_asin(constraint, &self.complete_solution),
            "float_asinh" => self
                .float_variable_assigner
                .float_asinh(constraint, &self.complete_solution),
            "float_atan" => self
                .float_variable_assigner
                .float_atan(constraint, &self.complete_solution),
            "float_atanh" => self
                .float_variable_assigner
                .float_atanh(constraint, &self.complete_solution),
            "float_cos" => self
                .float_variable_assigner
                .float_cos(constraint, &self.complete_solution),
            "float_cosh" => self
                .float_variable_assigner
                .float_cosh(constraint, &self.complete_solution),
            "float_exp" => self
                .float_variable_assigner
                .float_exp(constraint, &self.complete_solution),
            "float_lin_eq" => self.float_variable_assigner.float_lin_eq(
                constraint,
                &self.complete_solution,
                variable,
            ),
            "float_ln" => self
                .float_variable_assigner
                .float_ln(constraint, &self.complete_solution),
            "float_log10" => self
                .float_variable_assigner
                .float_log10(constraint, &self.complete_solution),
            "float_log2" => self
                .float_variable_assigner
                .float_log2(constraint, &self.complete_solution),
            "float_sin" => self
                .float_variable_assigner
                .float_sin(constraint, &self.complete_solution),
            "float_sinh" => self
                .float_variable_assigner
                .float_sinh(constraint, &self.complete_solution),
            "float_sqrt" => self
                .float_variable_assigner
                .float_sqrt(constraint, &self.complete_solution),
            "float_tan" => self
                .float_variable_assigner
                .float_tan(constraint, &self.complete_solution),
            "float_tanh" => self
                .float_variable_assigner
                .float_tanh(constraint, &self.complete_solution),
            "int2float" => self
                .float_variable_assigner
                .int2float(constraint, &self.complete_solution),
            _ => panic!("Unhandled float constraint {}", constraint.call.id),
        }
    }

    fn build_bool_closure(
        &self,
        constraint: &CallWithDefines,
        _variable: &String,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        match constraint.call.id.as_str() {
            "array_bool_and" => self
                .bool_variable_assigner
                .array_bool_and(constraint, &self.complete_solution),
            "array_bool_element" | "array_var_bool_element" => self
                .bool_variable_assigner
                .array_bool_element(constraint, &self.complete_solution),
            "bool_and" => self
                .bool_variable_assigner
                .bool_and(constraint, &self.complete_solution),
            "bool_clause" => self
                .bool_variable_assigner
                .bool_clause(constraint, &self.complete_solution),
            "bool_eq" => self
                .bool_variable_assigner
                .bool_eq(constraint, &self.complete_solution),
            "bool_not" => self
                .bool_variable_assigner
                .bool_not(constraint, &self.complete_solution),
            "bool_eq_reif" => self
                .bool_variable_assigner
                .bool_eq_reif(constraint, &self.complete_solution),
            "bool_le_reif" => self
                .bool_variable_assigner
                .bool_le_reif(constraint, &self.complete_solution),
            "bool_lt_reif" => self
                .bool_variable_assigner
                .bool_lt_reif(constraint, &self.complete_solution),
            "bool_or" => self
                .bool_variable_assigner
                .bool_or(constraint, &self.complete_solution),
            "bool_xor" => self
                .bool_variable_assigner
                .bool_xor(constraint, &self.complete_solution),
            "float_eq_reif" => self
                .float_variable_assigner
                .float_eq_reif(constraint, &self.complete_solution),
            "float_le_reif" => self
                .float_variable_assigner
                .float_le_reif(constraint, &self.complete_solution),
            "float_lin_eq_reif" => self
                .float_variable_assigner
                .float_lin_eq_reif(constraint, &self.complete_solution),
            "float_lin_le_reif" => self
                .float_variable_assigner
                .float_lin_le_reif(constraint, &self.complete_solution),
            "float_lin_ne_reif" => self
                .float_variable_assigner
                .float_lin_ne_reif(constraint, &self.complete_solution),
            "float_lin_lt_reif" => self
                .float_variable_assigner
                .float_lin_lt_reif(constraint, &self.complete_solution),
            "float_lt_reif" => self
                .float_variable_assigner
                .float_lt_reif(constraint, &self.complete_solution),
            "float_ne_reif" => self
                .float_variable_assigner
                .float_ne_reif(constraint, &self.complete_solution),
            "int_eq_reif" => self
                .int_variable_assigner
                .int_eq_reif(constraint, &self.complete_solution),
            "int_le_reif" => self
                .int_variable_assigner
                .int_le_reif(constraint, &self.complete_solution),
            "int_lin_eq_reif" => self
                .int_variable_assigner
                .int_lin_eq_reif(constraint, &self.complete_solution),
            "int_lin_le_reif" => self
                .int_variable_assigner
                .int_lin_le_reif(constraint, &self.complete_solution),
            "int_lin_ne_reif" => self
                .int_variable_assigner
                .int_lin_ne_reif(constraint, &self.complete_solution),
            "int_lt_reif" => self
                .int_variable_assigner
                .int_lt_reif(constraint, &self.complete_solution),
            "int_ne_reif" => self
                .int_variable_assigner
                .int_ne_reif(constraint, &self.complete_solution),
            "set_eq_reif" => self
                .set_variable_assigner
                .set_eq_reif(constraint, &self.complete_solution),
            "set_in_reif" => self
                .set_variable_assigner
                .set_in_reif(constraint, &self.complete_solution),
            "set_le_reif" => self
                .set_variable_assigner
                .set_le_reif(constraint, &self.complete_solution),
            "set_lt_reif" => self
                .set_variable_assigner
                .set_lt_reif(constraint, &self.complete_solution),
            "set_ne_reif" => self
                .set_variable_assigner
                .set_ne_reif(constraint, &self.complete_solution),
            "set_subset_reif" => self
                .set_variable_assigner
                .set_subset_reif(constraint, &self.complete_solution),
            "set_superset_reif" => self
                .set_variable_assigner
                .set_superset_reif(constraint, &self.complete_solution),

            _ => panic!("Unhandled bool constraint {}", constraint.call.id),
        }
    }

    fn build_set_closure(
        &self,
        constraint: &CallWithDefines,
        _variable: &String,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> HashSet<i64> + Send + Sync> {
        match constraint.call.id.as_str() {
            "array_set_element" | "array_var_set_element" => self
                .set_variable_assigner
                .array_set_element(&constraint, &self.complete_solution),
            "set_diff" => self
                .set_variable_assigner
                .set_diff(constraint, &self.complete_solution),
            "set_eq" => self
                .set_variable_assigner
                .set_eq(constraint, &self.complete_solution),
            "set_intersect" => self
                .set_variable_assigner
                .set_intersect(constraint, &self.complete_solution),
            "set_symdiff" => self
                .set_variable_assigner
                .set_symmetric_difference(constraint, &self.complete_solution),
            "set_union" => self
                .set_variable_assigner
                .set_union(constraint, &self.complete_solution),
            _ => panic!("Unhandled set constraint {}", constraint.call.id),
        }
    }
}

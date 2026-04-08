use crate::args_extractor::args_extractor::ArgsExtractor;
use crate::data_utility::types::Register;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::data_utility::types::VariableValue;
use crate::variable_assigner::sub_types::bool_variable_assigner::BoolVariableAssigner;
use crate::variable_assigner::sub_types::float_variable_assigner::FloatVariableAssigner;
use crate::variable_assigner::sub_types::int_variable_assigner::IntVariableAssigner;
use crate::variable_assigner::sub_types::set_variable_assigner::SetVariableAssigner;

use flatzinc_serde::Array;

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

type AssignFn = Arc<dyn Fn(&[Option<VariableValue>]) -> VariableValue + Send + Sync>;

#[derive(Clone, Default)]
/// The `VariableAssigner` struct is responsible for assigning values to defined variables based on the constraints and a partial solution.
/// It maintains a list of defined variables, constraints that define these variables, and a mapping of assigned functions
/// that compute the values of these variables based on the constraints and the current solution.
pub struct VariableAssigner {
    /// A vector of variable names that are defined by the constraints and need to be assigned values.
    defined_variable: Vec<String>,
    /// A vector of constraints (with their defines) that are used to determine the values of the defined variables.
    constraints: Vec<CallWithDefines>,
    /// A hashmap that maps variable names to their indices in the solution vector.
    variable_register_map: HashMap<String, Register>,
    /// A vector of tuples where each tuple contains a variable register and a corresponding function that computes its value based on the current solution.
    assigned_functions: Vec<(Register, AssignFn)>,
    /// An instance of `ArgsExtractor` used to extract arguments from constraints when building the assigned functions.
    args_extractor: ArgsExtractor,
    /// A hashmap that represents the complete solution, mapping variable names to their assigned values. This is updated as variables are assigned.
    complete_solution: HashMap<String, VariableValue>,
    /// A hashmap that maps identifiers to their corresponding arrays, used for resolving array references in constraints when building assigned functions.
    arrays: HashMap<String, Array>,
    /// An instance of `IntVariableAssigner` used to build functions for assigning integer variables based on integer constraints.
    int_variable_assigner: IntVariableAssigner,
    /// An instance of `FloatVariableAssigner` used to build functions for assigning float variables based on float constraints.
    float_variable_assigner: FloatVariableAssigner,
    /// An instance of `BoolVariableAssigner` used to build functions for assigning boolean variables based on boolean constraints.
    bool_variable_assigner: BoolVariableAssigner,
    /// An instance of `SetVariableAssigner` used to build functions for assigning set variables based on set constraints.
    set_variable_assigner: SetVariableAssigner,
}

/// Custom implementation of the `Debug` trait for `VariableAssigner` to provide a more concise and informative debug output,
/// especially for the `assigned_functions` field which contains closures that cannot be directly printed.
impl fmt::Debug for VariableAssigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VariableAssigner")
            .field("defined_variable", &self.defined_variable)
            .field("constraints", &self.constraints)
            .field(
                "assigned_functions",
                &self
                    .assigned_functions
                    .iter()
                    .map(|(reg, _)| reg)
                    .collect::<Vec<_>>(),
            )
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

/// Implementation of the `VariableAssigner` struct, providing methods to create a new assigner, assign values to defined variables based on a partial solution,
/// and build the assigned functions based on the constraints that define the variables.
impl VariableAssigner {
    /// Creates a new `VariableAssigner` with the provided defined variables, constraints, and arrays.
    /// The constructor initializes the internal state of the assigner, including the assigned functions and variable assigners for
    /// integers, floats, booleans, and sets, which will be used to build the functions for assigning variable values based on the constraints.
    pub fn new(
        defined_variable: Vec<String>,
        constraints: Vec<CallWithDefines>,
        arrays: HashMap<String, Array>,
        variable_map: HashMap<String, Register>,
    ) -> Self {
        Self {
            defined_variable,
            constraints,
            assigned_functions: Vec::new(),
            args_extractor: ArgsExtractor::new(),
            complete_solution: HashMap::new(),
            arrays: arrays.clone(),
            variable_register_map: variable_map.clone(),
            int_variable_assigner: IntVariableAssigner::new(arrays.clone(), variable_map.clone()),
            float_variable_assigner: FloatVariableAssigner::new(
                arrays.clone(),
                variable_map.clone(),
            ),
            bool_variable_assigner: BoolVariableAssigner::new(arrays.clone(), variable_map.clone()),
            set_variable_assigner: SetVariableAssigner::new(arrays, variable_map.clone()),
        }
    }

    /// Assigns values to the defined variables based on the provided partial solution and the constraints that define these variables.
    ///
    /// # Arguments
    /// * `partial_solution_vec` - A vector of optional `VariableValue` representing the current partial solution, where each index corresponds to a variable register.
    /// This is used as the basis for computing the values of the defined variables.
    /// # Returns
    /// A vector of optional `VariableValue` representing the complete solution, where the defined variables have been assigned values based on the constraints and the partial solution.
    pub fn assign_defined_variables(
        &mut self,
        partial_solution_vec: &[Option<VariableValue>],
    ) -> Vec<Option<VariableValue>> {
        let mut complete_solution_vec = partial_solution_vec.to_vec();
        if self.assigned_functions.is_empty() {
            self.build_assigned_functions();
        }

        for (register_inserted_var, func) in &self.assigned_functions {
            let idx = *register_inserted_var as usize;
            if complete_solution_vec[idx].is_none() {
                let value = func(&complete_solution_vec);
                complete_solution_vec[idx] = Some(value);
            }
        }

        complete_solution_vec
    }

    fn build_assigned_functions(&mut self) {
        for constraint in &self.constraints {
            let Some(var) = constraint.defines.clone() else {
                continue;
            };
            let register = *self
                .variable_register_map
                .get(&var)
                .expect("Variable not found in variable_map");
            let closure = self.build_closure(constraint, &var);
            self.assigned_functions.push((register, closure));
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
        } else if id.starts_with("float_")
            || id.contains("array_float")
            || id.contains("array_var_float")
        {
            self.wrap_float(constraint, variable)
        } else if id.starts_with("bool_")
            || id.contains("array_bool")
            || id.contains("array_var_bool")
        {
            self.wrap_bool(constraint, variable)
        } else if id.starts_with("set_") || id.contains("array_set") || id.contains("array_var_set")
        {
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
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> i64 + Send + Sync> {
        match constraint.call.id.as_str() {
            "array_int_element" => self.int_variable_assigner.array_int_element(constraint),
            "array_var_int_element" => self.int_variable_assigner.array_var_int_element(constraint),
            "int_abs" => self.int_variable_assigner.int_abs(constraint),
            "int_div" => self.int_variable_assigner.int_div(constraint),
            "int_eq" => self.int_variable_assigner.int_eq(constraint),
            "int_lin_eq" => self.int_variable_assigner.int_lin_eq(constraint, variable),
            "int_max" => self.int_variable_assigner.int_max(constraint),
            "int_min" => self.int_variable_assigner.int_min(constraint),
            "int_mod" => self.int_variable_assigner.int_mod(constraint),
            "int_pow" => self.int_variable_assigner.int_pow(constraint),
            "int_times" => self.int_variable_assigner.int_times(constraint),
            "bool2int" => self.bool_variable_assigner.bool2int(constraint),
            _ => panic!("Unhandled int constraint {}", constraint.call.id),
        }
    }

    fn build_float_closure(
        &self,
        constraint: &CallWithDefines,
        variable: &String,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> f64 + Send + Sync> {
        match constraint.call.id.as_str() {
            "array_float_element" => self.float_variable_assigner.array_float_element(constraint),
            "array_var_float_element" => self
                .float_variable_assigner
                .array_var_float_element(constraint),
            "float_abs" => self.float_variable_assigner.float_abs(constraint),
            "float_div" => self.float_variable_assigner.float_div(constraint),
            "float_eq" => self.float_variable_assigner.float_eq(constraint),
            "float_max" => self.float_variable_assigner.float_max(constraint),
            "float_min" => self.float_variable_assigner.float_min(constraint),
            "float_plus" => self.float_variable_assigner.float_plus(constraint),
            "float_pow" => self.float_variable_assigner.float_pow(constraint),
            "float_times" => self.float_variable_assigner.float_times(constraint),
            "float_acos" => self.float_variable_assigner.float_acos(constraint),
            "float_acosh" => self.float_variable_assigner.float_acosh(constraint),
            "float_asin" => self.float_variable_assigner.float_asin(constraint),
            "float_asinh" => self.float_variable_assigner.float_asinh(constraint),
            "float_atan" => self.float_variable_assigner.float_atan(constraint),
            "float_atanh" => self.float_variable_assigner.float_atanh(constraint),
            "float_cos" => self.float_variable_assigner.float_cos(constraint),
            "float_cosh" => self.float_variable_assigner.float_cosh(constraint),
            "float_exp" => self.float_variable_assigner.float_exp(constraint),
            "float_lin_eq" => self
                .float_variable_assigner
                .float_lin_eq(constraint, variable),
            "float_ln" => self.float_variable_assigner.float_ln(constraint),
            "float_log10" => self.float_variable_assigner.float_log10(constraint),
            "float_log2" => self.float_variable_assigner.float_log2(constraint),
            "float_sin" => self.float_variable_assigner.float_sin(constraint),
            "float_sinh" => self.float_variable_assigner.float_sinh(constraint),
            "float_sqrt" => self.float_variable_assigner.float_sqrt(constraint),
            "float_tan" => self.float_variable_assigner.float_tan(constraint),
            "float_tanh" => self.float_variable_assigner.float_tanh(constraint),
            "int2float" => self.float_variable_assigner.int2float(constraint),
            _ => panic!("Unhandled float constraint {}", constraint.call.id),
        }
    }

    fn build_bool_closure(
        &self,
        constraint: &CallWithDefines,
        _variable: &String,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> bool + Send + Sync> {
        match constraint.call.id.as_str() {
            "array_bool_and" => self.bool_variable_assigner.array_bool_and(constraint),
            "array_bool_element" => self.bool_variable_assigner.array_bool_element(constraint),
            "array_var_bool_element" => self
                .bool_variable_assigner
                .array_var_bool_element(constraint),
            "bool_and" => self.bool_variable_assigner.bool_and(constraint),
            "bool_clause" => self
                .bool_variable_assigner
                .bool_clause(constraint, constraint.defines.as_ref().unwrap()),
            "bool_eq" => self.bool_variable_assigner.bool_eq(constraint),
            "bool_not" => self.bool_variable_assigner.bool_not(constraint),
            "bool_eq_reif" => self.bool_variable_assigner.bool_eq_reif(constraint),
            "bool_le_reif" => self.bool_variable_assigner.bool_le_reif(constraint),
            "bool_lt_reif" => self.bool_variable_assigner.bool_lt_reif(constraint),
            "bool_or" => self.bool_variable_assigner.bool_or(constraint),
            "bool_xor" => self.bool_variable_assigner.bool_xor(constraint),
            "float_eq_reif" => self.float_variable_assigner.float_eq_reif(constraint),
            "float_le_reif" => self.float_variable_assigner.float_le_reif(constraint),
            "float_lin_eq_reif" => self.float_variable_assigner.float_lin_eq_reif(constraint),
            "float_lin_le_reif" => self.float_variable_assigner.float_lin_le_reif(constraint),
            "float_lin_ne_reif" => self.float_variable_assigner.float_lin_ne_reif(constraint),
            "float_lin_lt_reif" => self.float_variable_assigner.float_lin_lt_reif(constraint),
            "float_lt_reif" => self.float_variable_assigner.float_lt_reif(constraint),
            "float_ne_reif" => self.float_variable_assigner.float_ne_reif(constraint),
            "int_eq_reif" => self.int_variable_assigner.int_eq_reif(constraint),
            "int_le_reif" => self.int_variable_assigner.int_le_reif(constraint),
            "int_lin_eq_reif" => self.int_variable_assigner.int_lin_eq_reif(constraint),
            "int_lin_le_reif" => self.int_variable_assigner.int_lin_le_reif(constraint),
            "int_lin_ne_reif" => self.int_variable_assigner.int_lin_ne_reif(constraint),
            "int_lt_reif" => self.int_variable_assigner.int_lt_reif(constraint),
            "int_ne_reif" => self.int_variable_assigner.int_ne_reif(constraint),
            "set_eq_reif" => self.set_variable_assigner.set_eq_reif(constraint),
            "set_in_reif" => self.set_variable_assigner.set_in_reif(constraint),
            "set_le_reif" => self.set_variable_assigner.set_le_reif(constraint),
            "set_lt_reif" => self.set_variable_assigner.set_lt_reif(constraint),
            "set_ne_reif" => self.set_variable_assigner.set_ne_reif(constraint),
            "set_subset_reif" => self.set_variable_assigner.set_subset_reif(constraint),
            "set_superset_reif" => self.set_variable_assigner.set_superset_reif(constraint),
            _ => panic!("Unhandled bool constraint {}", constraint.call.id),
        }
    }

    fn build_set_closure(
        &self,
        constraint: &CallWithDefines,
        _variable: &String,
    ) -> Box<dyn Fn(&[Option<VariableValue>]) -> HashSet<i64> + Send + Sync> {
        match constraint.call.id.as_str() {
            "array_set_element" => self.set_variable_assigner.array_set_element(&constraint),
            "array_var_set_element" => self
                .set_variable_assigner
                .array_var_set_element(&constraint),
            "set_diff" => self.set_variable_assigner.set_diff(constraint),
            "set_eq" => self.set_variable_assigner.set_eq(constraint),
            "set_intersect" => self.set_variable_assigner.set_intersect(constraint),
            "set_symdiff" => self
                .set_variable_assigner
                .set_symmetric_difference(constraint),
            "set_union" => self.set_variable_assigner.set_union(constraint),
            _ => panic!("Unhandled set constraint {}", constraint.call.id),
        }
    }
}

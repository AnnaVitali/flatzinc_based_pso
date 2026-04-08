use crate::data_utility::types::Register;
use crate::data_utility::types::VariableValue;
use crate::evaluator::sub_types::bool_evaluator::BoolEvaluator;
use crate::evaluator::sub_types::float_evaluator::FloatEvaluator;
use crate::evaluator::sub_types::int_evaluator::IntEvaluator;
use crate::evaluator::sub_types::set_evaluator::SetEvaluator;
use crate::invariant_graph::InvariantGraph;
use crate::solution_provider::SolutionProvider;
use crate::variable_assigner::variable_assigner::VariableAssigner;
use env_logger::Env;
use flatzinc_serde::{Array, Constraint, Domain, FlatZinc, Literal, Type};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::{fmt, vec};

#[derive(Debug, Clone)]
/// A struct representing a constraint call along with its defines (if any).
pub struct CallWithDefines {
    pub(crate) id: usize,
    pub(crate) call: Constraint,
    pub(crate) defines: Option<String>,
}
#[derive(Clone, Default)]
/// An evaluator for FlatZinc constraints that evaluates the constraints based on a provided solution map.
/// This struct maintains the FlatZinc model, variable bounds, constraints, and functional evaluators for different types of constraints (float, int, bool, set).
/// It also includes a variable assigner to assign values to defined variables based on the constraints and arrays in the model.
pub struct MiniEvaluator {
    /// The FlatZinc model being evaluated.
    fzn: FlatZinc,
    /// A map from variable names to their (min, max) bounds as `VariableValue` pairs.
    variable_bounds: HashMap<String, (VariableValue, VariableValue)>,
    /// A map from variable names to their corresponding register IDs.
    variable_register_map: HashMap<String, Register>,
    /// A vector representing the current solution values for each variable, indexed by their register IDs, used for efficient access during constraint evaluation.
    solution_vec: Vec<VariableValue>,
    /// A vector of constraints along with their defines (if any) to be evaluated.
    constraints: Vec<CallWithDefines>,
    /// A vector of functional evaluators corresponding to the constraints, used to compute violation values.
    violation_functions: Vec<Arc<dyn Fn(&[VariableValue]) -> f64 + Send + Sync>>,
    /// A flag to enable verbose logging during evaluation.
    verbose: bool,
    /// A map from array identifiers to their corresponding `Array` definitions from the FlatZinc model.
    arrays_hashmap: HashMap<String, Array>,
    /// Functional evaluators for different types of constraints (float, int, bool, set) that provide methods to evaluate specific constraint types.
    float_functional_evaluator: FloatEvaluator,
    /// Functional evaluator for integer constraints.
    int_functional_evaluator: IntEvaluator,
    /// Functional evaluator for boolean constraints.
    bool_functional_evaluator: BoolEvaluator,
    /// Functional evaluator for set constraints.
    set_functional_evaluator: SetEvaluator,
    /// A variable assigner that assigns values to defined variables based on the constraints and arrays in the model.
    variable_assigner: VariableAssigner,
    /// The current solution map that is being evaluated, mapping variable names to their assigned values.
    solution: HashMap<String, VariableValue>,
}

/// Custom implementation of the `Debug` trait for `MiniEvaluator` to provide a more concise and relevant debug output.
impl fmt::Debug for MiniEvaluator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionalEvaluator")
            .field("constraints_len", &self.constraints.len())
            .field("violation_functions_len", &self.violation_functions.len())
            .field("verbose", &self.verbose)
            .field("arrays_hashmap_len", &self.arrays_hashmap.len())
            .field("solution_len", &self.solution.len())
            .finish()
    }
}

/// Implementation of the `MiniEvaluator` struct, providing methods to create a new evaluator,
///  evaluate constraints based on a solution provider, and populate violation functions for the constraints.
impl MiniEvaluator {
    /// Creates a new `MiniEvaluator` instance by loading constraints from the provided FlatZinc model and building an invariant graph.
    pub fn new(path: &Path, fzn: FlatZinc, option: Option<&str>) -> Self {
        let mut constraints = Self::load_constraints_with_defines(path, &fzn);
        let arrays_hashmap: HashMap<String, Array> = fzn
            .arrays
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let graph = InvariantGraph::build(&constraints, &arrays_hashmap, false);
        constraints = graph.topologically_sorted_constraints(&constraints);
        let variable_register_map: HashMap<String, Register> = fzn
            .variables
            .iter()
            .enumerate()
            .map(|(i, var)| (var.0.clone(), (i) as Register))
            .collect();

        let mut verbose = false;
        if option.is_some() {
            match option.unwrap() {
                "verbose" => {
                    verbose = true;
                    let _ = env_logger::Builder::from_env(Env::default().default_filter_or("info"))
                        .try_init();
                }
                _ => {}
            }
        }

        let defined_variables: Vec<String> = fzn
            .variables
            .iter()
            .filter_map(|(id, var)| if var.defined { Some(id.clone()) } else { None })
            .collect();

        let variable_assigner = VariableAssigner::new(
            defined_variables,
            constraints.clone(),
            arrays_hashmap.clone(),
            variable_register_map.clone(),
        );

        let float_functional_evaluator = FloatEvaluator::new(
            arrays_hashmap.clone(),
            variable_register_map.clone(),
            verbose,
        );
        let int_functional_evaluator = IntEvaluator::new(
            arrays_hashmap.clone(),
            variable_register_map.clone(),
            verbose,
        );
        let bool_functional_evaluator = BoolEvaluator::new(
            arrays_hashmap.clone(),
            variable_register_map.clone(),
            verbose,
        );
        let set_functional_evaluator = SetEvaluator::new(
            arrays_hashmap.clone(),
            variable_register_map.clone(),
            verbose,
        );

        let solution_vec: Vec<VariableValue> =
            vec![VariableValue::Int(0); variable_register_map.len()];

        let mut evaluator = Self {
            fzn,
            variable_bounds: HashMap::new(),
            variable_register_map,
            solution_vec,
            constraints,
            violation_functions: Vec::new(),
            verbose,
            arrays_hashmap,
            float_functional_evaluator,
            int_functional_evaluator,
            bool_functional_evaluator,
            set_functional_evaluator,
            variable_assigner,
            solution: HashMap::new(),
        };

        evaluator.get_variable_bounds();

        evaluator
    }

    /// Evaluates the constraints based on the provided solution map from the `SolutionProvider`, returning the objective value (if any) and the total violation value.
    ///
    /// # Arguments
    /// * `solution_provider` - A reference to a `SolutionProvider` that provides the current solution map for evaluation.
    ///
    /// # Returns
    /// A tuple containing an optional objective value (as `f64`) and the total violation value (as `f64`), which is the sum of constraint violations and domain violations.
    pub fn evaluate_invariants_graph(
        &mut self,
        solution_provider: &SolutionProvider,
    ) -> (Option<f64>, f64) {
        let partial_solution = solution_provider.solution_map();
        let mut partial_solution_vec: Vec<Option<VariableValue>> =
            vec![None; self.variable_register_map.len()];

        for (var_id, index) in &self.variable_register_map {
            if let Some(value) = partial_solution.get(var_id) {
                partial_solution_vec[*index as usize] = Some(value.clone());
            }
        }

        let complete_solution_vec = self
            .variable_assigner
            .assign_defined_variables(&partial_solution_vec);

        for (i, val_opt) in complete_solution_vec.iter().enumerate() {
            if let Some(val) = val_opt {
                self.solution_vec[i] = val.clone();
            }
        }

        self.solution.clear();
        for (var_id, index) in &self.variable_register_map {
            if let Some(val) = complete_solution_vec
                .get(*index as usize)
                .and_then(|v| v.as_ref())
            {
                self.solution.insert(var_id.clone(), val.clone());
            }
        }

        let domain_violation = self.evaluate_domain_constraints();
        let (objective, constraint_violation) =
            self.evaluate_constraint_list(&self.constraints.clone());

        (objective, constraint_violation + domain_violation)
    }

    fn evaluate_constraint_list(
        &mut self,
        constraint_to_evaluate: &[CallWithDefines],
    ) -> (Option<f64>, f64) {
        if self.violation_functions.is_empty() {
            self.populate_violation_functions();
        }

        let mut constraint_violation = 0.0;

        let len = constraint_to_evaluate.len();
        if len == 0 {
            let mut objective: Option<f64> = None;
            if let Some(objective_lit) = self.fzn.solve.objective.as_ref() {
                let objective_id: &str = match objective_lit {
                    Literal::Identifier(id) => id.as_str(),
                    Literal::String(id) => id.as_str(),
                    _ => panic!("Objective literal must be an identifier or string"),
                };
                if let Some(obj_val) = self.solution.get(objective_id) {
                    match obj_val {
                        VariableValue::Int(val) => objective = Some(*val as f64),
                        VariableValue::Float(val) => objective = Some(*val),
                        _ => log::error!("Objective variable must be numeric"),
                    }
                }
            }
            return (objective, constraint_violation);
        }

        let vf_ref: &Vec<Arc<dyn Fn(&[VariableValue]) -> f64 + Send + Sync>> =
            &self.violation_functions;

        for constraint in constraint_to_evaluate.iter() {
            if constraint.call.ann.is_empty() {
                let idx = constraint.id;
                let violation = (vf_ref[idx])(&self.solution_vec);
                constraint_violation += violation;
            }
        }

        let mut objective: Option<f64> = None;
        if let Some(objective_lit) = self.fzn.solve.objective.as_ref() {
            let objective_id: &str = match objective_lit {
                Literal::Identifier(id) => id.as_str(),
                _ => panic!("Objective must be an identifier or string"),
            };
            if let Some(obj_val) = self.solution.get(objective_id) {
                match obj_val {
                    VariableValue::Int(val) => objective = Some(*val as f64),
                    VariableValue::Float(val) => objective = Some(*val),
                    _ => log::error!("Objective variable must be numeric"),
                }
            }
        }

        (objective, constraint_violation)
    }

    fn populate_violation_functions(&mut self) {
        if !self.violation_functions.is_empty() {
            return;
        }

        self.violation_functions = Vec::with_capacity(self.constraints.len());

        for constraint in &self.constraints {
            if !constraint.call.ann.is_empty() {
                self.violation_functions
                    .push(Arc::new(|_sol: &[VariableValue]| 0.0));
                continue;
            }

            let id = constraint.call.id.as_str();

            let func_arc: Arc<dyn Fn(&[VariableValue]) -> f64 + Send + Sync> = match id {
                "array_int_element" => {
                    Arc::from(self.int_functional_evaluator.array_int_element(constraint))
                }
                "array_var_int_element" => Arc::from(
                    self.int_functional_evaluator
                        .array_var_int_element(constraint),
                ),
                "int_abs" => Arc::from(self.int_functional_evaluator.int_abs(constraint)),
                "int_div" => Arc::from(self.int_functional_evaluator.int_div(constraint)),
                "int_eq" => Arc::from(self.int_functional_evaluator.int_eq(constraint)),
                "int_eq_reif" => Arc::from(self.int_functional_evaluator.int_eq_reif(constraint)),
                "int_le" => Arc::from(self.int_functional_evaluator.int_le(constraint)),
                "int_le_reif" => Arc::from(self.int_functional_evaluator.int_le_reif(constraint)),
                "int_lin_eq" => Arc::from(self.int_functional_evaluator.int_lin_eq(constraint)),
                "int_lin_eq_reif" => {
                    Arc::from(self.int_functional_evaluator.int_lin_eq_reif(constraint))
                }
                "int_lin_le" => Arc::from(self.int_functional_evaluator.int_lin_le(constraint)),
                "int_lin_le_reif" => {
                    Arc::from(self.int_functional_evaluator.int_lin_le_reif(constraint))
                }
                "int_lin_ne" => Arc::from(self.int_functional_evaluator.int_lin_ne(constraint)),
                "int_lin_ne_reif" => {
                    Arc::from(self.int_functional_evaluator.int_lin_ne_reif(constraint))
                }
                "int_lt" => Arc::from(self.int_functional_evaluator.int_lt(constraint)),
                "int_lt_reif" => Arc::from(self.int_functional_evaluator.int_lt_reif(constraint)),
                "int_max" => Arc::from(self.int_functional_evaluator.int_max(constraint)),
                "int_min" => Arc::from(self.int_functional_evaluator.int_min(constraint)),
                "int_mod" => Arc::from(self.int_functional_evaluator.int_mod(constraint)),
                "int_ne" => Arc::from(self.int_functional_evaluator.int_ne(constraint)),
                "int_ne_reif" => Arc::from(self.int_functional_evaluator.int_ne_reif(constraint)),
                "int_pow" => Arc::from(self.int_functional_evaluator.int_pow(constraint)),
                "int_times" => Arc::from(self.int_functional_evaluator.int_times(constraint)),
                "array_float_element" => Arc::from(
                    self.float_functional_evaluator
                        .array_float_element(constraint),
                ),
                "array_var_float_element" => Arc::from(
                    self.float_functional_evaluator
                        .array_var_float_element(constraint),
                ),
                "float_abs" => Arc::from(self.float_functional_evaluator.float_abs(constraint)),
                "float_acos" => Arc::from(self.float_functional_evaluator.float_acos(constraint)),
                "float_acosh" => Arc::from(self.float_functional_evaluator.float_acosh(constraint)),
                "float_asin" => Arc::from(self.float_functional_evaluator.float_asin(constraint)),
                "float_asinh" => Arc::from(self.float_functional_evaluator.float_asinh(constraint)),
                "float_atan" => Arc::from(self.float_functional_evaluator.float_atan(constraint)),
                "float_atanh" => Arc::from(self.float_functional_evaluator.float_atanh(constraint)),
                "float_cos" => Arc::from(self.float_functional_evaluator.float_cos(constraint)),
                "float_cosh" => Arc::from(self.float_functional_evaluator.float_cosh(constraint)),
                "float_div" => Arc::from(self.float_functional_evaluator.float_div(constraint)),
                "float_eq" => Arc::from(self.float_functional_evaluator.float_eq(constraint)),
                "float_eq_reif" => {
                    Arc::from(self.float_functional_evaluator.float_eq_reif(constraint))
                }
                "float_exp" => Arc::from(self.float_functional_evaluator.float_exp(constraint)),
                "float_le" => Arc::from(self.float_functional_evaluator.float_le(constraint)),
                "float_le_reif" => {
                    Arc::from(self.float_functional_evaluator.float_le_reif(constraint))
                }
                "float_lin_eq" => {
                    Arc::from(self.float_functional_evaluator.float_lin_eq(constraint))
                }
                "float_lin_eq_reif" => Arc::from(
                    self.float_functional_evaluator
                        .float_lin_eq_reif(constraint),
                ),
                "float_lin_le" => {
                    Arc::from(self.float_functional_evaluator.float_lin_le(constraint))
                }
                "float_lin_le_reif" => Arc::from(
                    self.float_functional_evaluator
                        .float_lin_le_reif(constraint),
                ),
                "float_lin_lt" => {
                    Arc::from(self.float_functional_evaluator.float_lin_lt(constraint))
                }
                "float_lin_lt_reif" => Arc::from(
                    self.float_functional_evaluator
                        .float_lin_lt_reif(constraint),
                ),
                "float_lin_ne" => {
                    Arc::from(self.float_functional_evaluator.float_lin_ne(constraint))
                }
                "float_lin_ne_reif" => Arc::from(
                    self.float_functional_evaluator
                        .float_lin_ne_reif(constraint),
                ),
                "float_ln" => Arc::from(self.float_functional_evaluator.float_ln(constraint)),
                "float_log10" => Arc::from(self.float_functional_evaluator.float_log10(constraint)),
                "float_log2" => Arc::from(self.float_functional_evaluator.float_log2(constraint)),
                "float_lt" => Arc::from(self.float_functional_evaluator.float_lt(constraint)),
                "float_lt_reif" => {
                    Arc::from(self.float_functional_evaluator.float_lt_reif(constraint))
                }
                "float_max" => Arc::from(self.float_functional_evaluator.float_max(constraint)),
                "float_min" => Arc::from(self.float_functional_evaluator.float_min(constraint)),
                "float_ne" => Arc::from(self.float_functional_evaluator.float_ne(constraint)),
                "float_ne_reif" => {
                    Arc::from(self.float_functional_evaluator.float_ne_reif(constraint))
                }
                "float_plus" => Arc::from(self.float_functional_evaluator.float_plus(constraint)),
                "float_pow" => Arc::from(self.float_functional_evaluator.float_pow(constraint)),
                "float_sin" => Arc::from(self.float_functional_evaluator.float_sin(constraint)),
                "float_sinh" => Arc::from(self.float_functional_evaluator.float_sinh(constraint)),
                "float_sqrt" => Arc::from(self.float_functional_evaluator.float_sqrt(constraint)),
                "float_tan" => Arc::from(self.float_functional_evaluator.float_tan(constraint)),
                "float_tanh" => Arc::from(self.float_functional_evaluator.float_tanh(constraint)),
                "float_times" => Arc::from(self.float_functional_evaluator.float_times(constraint)),
                "int2float" => Arc::from(self.float_functional_evaluator.int2float(constraint)),
                "array_bool_and" => {
                    Arc::from(self.bool_functional_evaluator.array_bool_and(constraint))
                }
                "array_bool_element" => Arc::from(
                    self.bool_functional_evaluator
                        .array_bool_element(constraint),
                ),
                "array_var_bool_element" => Arc::from(
                    self.bool_functional_evaluator
                        .array_var_bool_element(constraint),
                ),
                "array_bool_xor" => {
                    Arc::from(self.bool_functional_evaluator.array_bool_xor(constraint))
                }
                "bool_and" => Arc::from(self.bool_functional_evaluator.bool_and(constraint)),
                "bool_clause" => Arc::from(self.bool_functional_evaluator.bool_clause(constraint)),
                "bool_eq" => Arc::from(self.bool_functional_evaluator.bool_eq(constraint)),
                "bool_eq_reif" => {
                    Arc::from(self.bool_functional_evaluator.bool_eq_reif(constraint))
                }
                "bool_le" => Arc::from(self.bool_functional_evaluator.bool_le(constraint)),
                "bool_le_reif" => {
                    Arc::from(self.bool_functional_evaluator.bool_le_reif(constraint))
                }
                "bool_lin_eq" => Arc::from(self.bool_functional_evaluator.bool_lin_eq(constraint)),
                "bool_lin_le" => Arc::from(self.bool_functional_evaluator.bool_lin_le(constraint)),
                "bool2int" => Arc::from(self.bool_functional_evaluator.bool2int(constraint)),
                "bool_lt" => Arc::from(self.bool_functional_evaluator.bool_lt(constraint)),
                "bool_lt_reif" => {
                    Arc::from(self.bool_functional_evaluator.bool_lt_reif(constraint))
                }
                "bool_not" => Arc::from(self.bool_functional_evaluator.bool_not(constraint)),
                "bool_or" => Arc::from(self.bool_functional_evaluator.bool_or(constraint)),
                "bool_xor" => Arc::from(self.bool_functional_evaluator.bool_xor(constraint)),
                "array_set_element" => {
                    Arc::from(self.set_functional_evaluator.array_set_element(constraint))
                }
                "array_var_set_element" => Arc::from(
                    self.set_functional_evaluator
                        .array_var_set_element(constraint),
                ),
                "set_card" => Arc::from(self.set_functional_evaluator.set_card(constraint)),
                "set_diff" => Arc::from(self.set_functional_evaluator.set_diff(constraint)),
                "set_eq" => Arc::from(self.set_functional_evaluator.set_eq(constraint)),
                "set_eq_reif" => Arc::from(self.set_functional_evaluator.set_eq_reif(constraint)),
                "set_in" => Arc::from(self.set_functional_evaluator.set_in(constraint)),
                "set_in_reif" => Arc::from(self.set_functional_evaluator.set_in_reif(constraint)),
                "set_intersect" => {
                    Arc::from(self.set_functional_evaluator.set_intersect(constraint))
                }
                "set_le" => Arc::from(self.set_functional_evaluator.set_le(constraint)),
                "set_le_reif" => Arc::from(self.set_functional_evaluator.set_le_reif(constraint)),
                "set_lt" => Arc::from(self.set_functional_evaluator.set_lt(constraint)),
                "set_lt_reif" => Arc::from(self.set_functional_evaluator.set_lt_reif(constraint)),
                "set_ne" => Arc::from(self.set_functional_evaluator.set_ne(constraint)),
                "set_ne_reif" => Arc::from(self.set_functional_evaluator.set_ne_reif(constraint)),
                "set_subset" => Arc::from(self.set_functional_evaluator.set_subset(constraint)),
                "set_subset_reif" => {
                    Arc::from(self.set_functional_evaluator.set_subset_reif(constraint))
                }
                "set_superset" => Arc::from(self.set_functional_evaluator.set_superset(constraint)),
                "set_superset_reif" => {
                    Arc::from(self.set_functional_evaluator.set_superset_reif(constraint))
                }
                "set_symdiff" => Arc::from(self.set_functional_evaluator.set_symdiff(constraint)),
                "set_union" => Arc::from(self.set_functional_evaluator.set_union(constraint)),
                _ => panic!("Unsupported constraint type: {}", id),
            };

            self.violation_functions.push(func_arc);
        }
    }

    fn evaluate_domain_constraints(&mut self) -> f64 {
        let mut violation = 0.0;

        for (var_id, _var) in &self.fzn.variables {
            if let Some((lower_bound, upper_bound)) = self.variable_bounds.get(var_id) {
                if let Some(value) = self.solution.get(var_id) {
                    match (value, lower_bound, upper_bound) {
                        (
                            VariableValue::Int(val),
                            VariableValue::Int(lb),
                            VariableValue::Int(ub),
                        ) => {
                            let mut v_amt = 0.0;
                            if *val < *lb {
                                v_amt = (*lb - *val) as f64;
                            } else if *val > *ub {
                                v_amt = (*val - *ub) as f64;
                            }
                            if v_amt > 0.0 {
                                violation += v_amt;
                                if self.verbose {
                                    log::info!(
                                        "Domain violation: variable `{}` value={} not in [{}, {}] -> +{}",
                                        var_id,
                                        val,
                                        lb,
                                        ub,
                                        v_amt
                                    );
                                }
                            }
                        }
                        (
                            VariableValue::Float(val),
                            VariableValue::Float(lb),
                            VariableValue::Float(ub),
                        ) => {
                            let mut v_amt = 0.0;
                            if *val < *lb {
                                v_amt = (*lb - *val).abs();
                            } else if *val > *ub {
                                v_amt = (*val - *ub).abs();
                            }
                            if v_amt > 0.0 {
                                violation += v_amt;
                                if self.verbose {
                                    log::info!(
                                        "Domain violation: variable `{}` value={} not in [{}, {}] -> +{}",
                                        var_id,
                                        val,
                                        lb,
                                        ub,
                                        v_amt
                                    );
                                }
                            }
                        }
                        (
                            VariableValue::Bool(val),
                            VariableValue::Int(lb),
                            VariableValue::Int(ub),
                        ) => {
                            let int_val = if *val { 1 } else { 0 };
                            let mut v_amt = 0.0;
                            if int_val < *lb {
                                v_amt = (*lb - int_val) as f64;
                            } else if int_val > *ub {
                                v_amt = (int_val - *ub) as f64;
                            }
                            if v_amt > 0.0 {
                                violation += v_amt;
                                if self.verbose {
                                    log::info!(
                                        "Domain violation: variable `{}` value={} not in [{}, {}] -> +{}",
                                        var_id,
                                        val,
                                        lb,
                                        ub,
                                        v_amt
                                    );
                                }
                            }
                        }
                        _ => log::error!(
                            "Mismatched variable and bounds types for variable `{}`",
                            var_id
                        ),
                    }
                }
            }
        }

        violation
    }

    fn get_variable_bounds(&mut self) {
        let variables = self.fzn.variables.iter();
        for (identifier, variable) in variables {
            let key = identifier.to_string();

            match variable.ty {
                Type::Int => {
                    let domain = variable.domain.as_ref();
                    if domain.is_none() {
                        if variable.defined || (variable.introduced && variable.defined) {
                            continue;
                        } else {
                            if self.verbose {
                                log::warn!("Int variable `{}` is unbounded", identifier);
                            }
                            continue;
                        }
                    } else {
                        match domain.unwrap() {
                            Domain::Int(range) => {
                                let min_v = *range.lower_bound().unwrap();
                                let max_v = *range.upper_bound().unwrap();
                                self.variable_bounds.insert(
                                    key.clone(),
                                    (VariableValue::Int(min_v), VariableValue::Int(max_v)),
                                );
                            }
                            _ => {
                                log::error!("Non-integer domain for int variable `{}`", identifier)
                            }
                        };
                    }
                }
                Type::Bool => {
                    self.variable_bounds
                        .insert(key, (VariableValue::Int(0), VariableValue::Int(1)));
                }
                Type::Float => {
                    let domain = variable.domain.as_ref();
                    if domain.is_none() {
                        if variable.defined || (variable.introduced && variable.defined) {
                            continue;
                        } else {
                            if self.verbose {
                                log::warn!("Float variable `{}` is unbounded", identifier);
                            }
                            continue;
                        }
                    } else {
                        match domain.unwrap() {
                            Domain::Float(range) => {
                                let min_v = *range.lower_bound().unwrap();
                                let max_v = *range.upper_bound().unwrap();
                                self.variable_bounds.insert(
                                    key,
                                    (VariableValue::Float(min_v), VariableValue::Float(max_v)),
                                );
                            }
                            _ => log::error!(
                                "Non-floating domain for float variable `{}`",
                                identifier
                            ),
                        };
                    }
                }
                _ => continue,
            }
        }
    }

    fn load_constraints_with_defines(path: &Path, fzn: &FlatZinc) -> Vec<CallWithDefines> {
        let mut s = String::new();
        if let Err(e) = File::open(path).and_then(|mut f| f.read_to_string(&mut s)) {
            eprintln!("Could not read {}: {}", path.display(), e);
            return Vec::new();
        }

        let raw: Value = match serde_json::from_str(&s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Could not parse JSON {}: {}", path.display(), e);
                return Vec::new();
            }
        };

        let defines_vec: Vec<Option<String>> = raw
            .get("constraints")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|item| match item.get("defines") {
                        Some(Value::String(st)) => Some(st.clone()),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        fzn.constraints
            .iter()
            .enumerate()
            .map(|(i, call)| CallWithDefines {
                id: i,
                call: call.clone(),
                defines: defines_vec.get(i).cloned().unwrap_or(None),
            })
            .collect()
    }

    pub fn solution(&self) -> &HashMap<String, VariableValue> {
        &self.solution
    }
}

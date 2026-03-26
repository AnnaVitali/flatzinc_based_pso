use crate::evaluator::sub_types::bool_functional_evaluator::BoolFunctionalEvaluator;
use crate::evaluator::sub_types::float_functional_evaluator::FloatFunctionalEvaluator;
use crate::evaluator::sub_types::int_functional_evaluator::IntFunctionalEvaluator;
use crate::evaluator::sub_types::set_functional_evaluator::SetFunctionalEvaluator;
use crate::invariant_graph::InvariantGraph;
use crate::solution_provider::{SolutionProvider, VariableValue};
use crate::variable_assigner::variable_assigner::VariableAssigner;
use env_logger::Env;
use flatzinc_serde::{Array, Call, Domain, FlatZinc, Identifier, Literal, Type, Variable};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::hash::RandomState;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CallWithDefines {
    pub(crate) id: usize,
    pub(crate) call: Call,
    pub(crate) defines: Option<Identifier>,
}
#[derive(Clone, Default)]
pub struct MiniEvaluator {
    fzn: FlatZinc,
    variable_bounds: HashMap<String, (VariableValue, VariableValue)>,
    constraints: Vec<CallWithDefines>,
    violation_functions: Vec<Arc<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync>>,
    verbose: bool,
    arrays_hashmap: HashMap<Identifier, Array>,
    float_functional_evaluator: FloatFunctionalEvaluator,
    int_functional_evaluator: IntFunctionalEvaluator,
    bool_functional_evaluator: BoolFunctionalEvaluator,
    set_functional_evaluator: SetFunctionalEvaluator,
    variable_assigner: VariableAssigner,
    solution: HashMap<String, VariableValue>,
}

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

impl MiniEvaluator {
    pub fn new(path: &Path, fzn: FlatZinc, option: Option<&str>) -> Self {
        let mut constraints = Self::load_constraints_with_defines(path, &fzn);
        let arrays_hashmap: HashMap<Identifier, Array> = fzn
            .arrays
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let graph = InvariantGraph::build(&constraints, &arrays_hashmap, false);
        constraints = graph.topologically_sorted_constraints(&constraints);

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
        );

        let float_functional_evaluator =
            FloatFunctionalEvaluator::new(arrays_hashmap.clone(), verbose);
        let int_functional_evaluator = IntFunctionalEvaluator::new(arrays_hashmap.clone(), verbose);
        let bool_functional_evaluator =
            BoolFunctionalEvaluator::new(arrays_hashmap.clone(), verbose);
        let set_functional_evaluator = SetFunctionalEvaluator::new(arrays_hashmap.clone(), verbose);

        let mut evaluator = Self {
            fzn,
            variable_bounds: HashMap::new(),
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

    pub fn evaluate_invariants_graph(
        &mut self,
        solution_provider: &SolutionProvider,
    ) -> (Option<f64>, f64) {
        self.solution = self
            .variable_assigner
            .assign_defined_variables(solution_provider.solution_map());

        let domain_violation = self.evaluate_domain_constraints();
        let (objective, constraint_violation) =
            self.evaluate_constraint_list(&self.constraints.clone());

        (objective, constraint_violation + domain_violation)
    }

    fn populate_violation_functions(&mut self) {
        if !self.violation_functions.is_empty() {
            return;
        }

        self.violation_functions = Vec::with_capacity(self.constraints.len());

        for constraint in &self.constraints {
            if !constraint.call.ann.is_empty() {
                self.violation_functions
                    .push(Arc::new(|_sol: &HashMap<String, VariableValue>| 0.0));
                continue;
            }

            let id = constraint.call.id.as_str();
            let func_arc: Arc<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync> =
                match id {
                    "array_float_element" => Arc::from(
                        self.float_functional_evaluator
                            .array_float_element(constraint, &self.solution),
                    ),
                    "array_var_float_element" => Arc::from(
                        self.float_functional_evaluator
                            .array_float_element(constraint, &self.solution),
                    ),
                    "float_abs" => Arc::from(
                        self.float_functional_evaluator
                            .float_abs(constraint, &self.solution),
                    ),
                    "float_acos" => Arc::from(
                        self.float_functional_evaluator
                            .float_acos(constraint, &self.solution),
                    ),
                    "float_acosh" => Arc::from(
                        self.float_functional_evaluator
                            .float_acosh(constraint, &self.solution),
                    ),
                    "float_asin" => Arc::from(
                        self.float_functional_evaluator
                            .float_asin(constraint, &self.solution),
                    ),
                    "float_asinh" => Arc::from(
                        self.float_functional_evaluator
                            .float_asinh(constraint, &self.solution),
                    ),
                    "float_atan" => Arc::from(
                        self.float_functional_evaluator
                            .float_atan(constraint, &self.solution),
                    ),
                    "float_atanh" => Arc::from(
                        self.float_functional_evaluator
                            .float_atanh(constraint, &self.solution),
                    ),
                    "float_cos" => Arc::from(
                        self.float_functional_evaluator
                            .float_cos(constraint, &self.solution),
                    ),
                    "float_cosh" => Arc::from(
                        self.float_functional_evaluator
                            .float_cosh(constraint, &self.solution),
                    ),
                    "float_div" => Arc::from(
                        self.float_functional_evaluator
                            .float_div(constraint, &self.solution),
                    ),
                    "float_eq" => Arc::from(
                        self.float_functional_evaluator
                            .float_eq(constraint, &self.solution),
                    ),
                    "float_eq_reif" => Arc::from(
                        self.float_functional_evaluator
                            .float_eq_reif(constraint, &self.solution),
                    ),
                    "float_exp" => Arc::from(
                        self.float_functional_evaluator
                            .float_exp(constraint, &self.solution),
                    ),
                    "float_le" => Arc::from(
                        self.float_functional_evaluator
                            .float_le(constraint, &self.solution),
                    ),
                    "float_le_reif" => Arc::from(
                        self.float_functional_evaluator
                            .float_le_reif(constraint, &self.solution),
                    ),
                    "float_lin_eq" => Arc::from(
                        self.float_functional_evaluator
                            .float_lin_eq(constraint, &self.solution),
                    ),
                    "float_lin_eq_reif" => Arc::from(
                        self.float_functional_evaluator
                            .float_lin_eq_reif(constraint, &self.solution),
                    ),
                    "float_lin_le" => Arc::from(
                        self.float_functional_evaluator
                            .float_lin_le(constraint, &self.solution),
                    ),
                    "float_lin_le_reif" => Arc::from(
                        self.float_functional_evaluator
                            .float_lin_le_reif(constraint, &self.solution),
                    ),
                    "float_lin_lt" => Arc::from(
                        self.float_functional_evaluator
                            .float_lin_lt(constraint, &self.solution),
                    ),
                    "float_lin_lt_reif" => Arc::from(
                        self.float_functional_evaluator
                            .float_lin_lt_reif(constraint, &self.solution),
                    ),
                    "float_lin_ne" => Arc::from(
                        self.float_functional_evaluator
                            .float_lin_ne(constraint, &self.solution),
                    ),
                    "float_lin_ne_reif" => Arc::from(
                        self.float_functional_evaluator
                            .float_lin_ne_reif(constraint, &self.solution),
                    ),
                    "float_ln" => Arc::from(
                        self.float_functional_evaluator
                            .float_ln(constraint, &self.solution),
                    ),
                    "float_log10" => Arc::from(
                        self.float_functional_evaluator
                            .float_log10(constraint, &self.solution),
                    ),
                    "float_log2" => Arc::from(
                        self.float_functional_evaluator
                            .float_log2(constraint, &self.solution),
                    ),
                    "float_lt" => Arc::from(
                        self.float_functional_evaluator
                            .float_lt(constraint, &self.solution),
                    ),
                    "float_lt_reif" => Arc::from(
                        self.float_functional_evaluator
                            .float_lt_reif(constraint, &self.solution),
                    ),
                    "float_max" => Arc::from(
                        self.float_functional_evaluator
                            .float_max(constraint, &self.solution),
                    ),
                    "float_min" => Arc::from(
                        self.float_functional_evaluator
                            .float_min(constraint, &self.solution),
                    ),
                    "float_ne" => Arc::from(
                        self.float_functional_evaluator
                            .float_ne(constraint, &self.solution),
                    ),
                    "float_ne_reif" => Arc::from(
                        self.float_functional_evaluator
                            .float_ne_reif(constraint, &self.solution),
                    ),
                    "float_plus" => Arc::from(
                        self.float_functional_evaluator
                            .float_plus(constraint, &self.solution),
                    ),
                    "float_pow" => Arc::from(
                        self.float_functional_evaluator
                            .float_pow(constraint, &self.solution),
                    ),
                    "float_sin" => Arc::from(
                        self.float_functional_evaluator
                            .float_sin(constraint, &self.solution),
                    ),
                    "float_sinh" => Arc::from(
                        self.float_functional_evaluator
                            .float_sinh(constraint, &self.solution),
                    ),
                    "float_sqrt" => Arc::from(
                        self.float_functional_evaluator
                            .float_sqrt(constraint, &self.solution),
                    ),
                    "float_tan" => Arc::from(
                        self.float_functional_evaluator
                            .float_tan(constraint, &self.solution),
                    ),
                    "float_tanh" => Arc::from(
                        self.float_functional_evaluator
                            .float_tanh(constraint, &self.solution),
                    ),
                    "float_times" => Arc::from(
                        self.float_functional_evaluator
                            .float_times(constraint, &self.solution),
                    ),
                    "int2float" => Arc::from(
                        self.float_functional_evaluator
                            .int2float(constraint, &self.solution),
                    ),

                    "array_int_element" => Arc::from(
                        self.int_functional_evaluator
                            .array_int_element(constraint, &self.solution),
                    ),
                    "array_var_int_element" => Arc::from(
                        self.int_functional_evaluator
                            .array_int_element(constraint, &self.solution),
                    ),
                    "int_abs" => Arc::from(
                        self.int_functional_evaluator
                            .int_abs(constraint, &self.solution),
                    ),
                    "int_div" => Arc::from(
                        self.int_functional_evaluator
                            .int_div(constraint, &self.solution),
                    ),
                    "int_eq" => Arc::from(
                        self.int_functional_evaluator
                            .int_eq(constraint, &self.solution),
                    ),
                    "int_eq_reif" => Arc::from(
                        self.int_functional_evaluator
                            .int_eq_reif(constraint, &self.solution),
                    ),
                    "int_le" => Arc::from(
                        self.int_functional_evaluator
                            .int_le(constraint, &self.solution),
                    ),
                    "int_le_reif" => Arc::from(
                        self.int_functional_evaluator
                            .int_le_reif(constraint, &self.solution),
                    ),
                    "int_lin_eq" => Arc::from(
                        self.int_functional_evaluator
                            .int_lin_eq(constraint, &self.solution),
                    ),
                    "int_lin_eq_reif" => Arc::from(
                        self.int_functional_evaluator
                            .int_lin_eq_reif(constraint, &self.solution),
                    ),
                    "int_lin_le" => Arc::from(
                        self.int_functional_evaluator
                            .int_lin_le(constraint, &self.solution),
                    ),
                    "int_lin_le_reif" => Arc::from(
                        self.int_functional_evaluator
                            .int_lin_le_reif(constraint, &self.solution),
                    ),
                    "int_lin_ne" => Arc::from(
                        self.int_functional_evaluator
                            .int_lin_ne(constraint, &self.solution),
                    ),
                    "int_lin_ne_reif" => Arc::from(
                        self.int_functional_evaluator
                            .int_lin_ne_reif(constraint, &self.solution),
                    ),
                    "int_lt" => Arc::from(
                        self.int_functional_evaluator
                            .int_lt(constraint, &self.solution),
                    ),
                    "int_lt_reif" => Arc::from(
                        self.int_functional_evaluator
                            .int_lt_reif(constraint, &self.solution),
                    ),
                    "int_max" => Arc::from(
                        self.int_functional_evaluator
                            .int_max(constraint, &self.solution),
                    ),
                    "int_min" => Arc::from(
                        self.int_functional_evaluator
                            .int_min(constraint, &self.solution),
                    ),
                    "int_mod" => Arc::from(
                        self.int_functional_evaluator
                            .int_mod(constraint, &self.solution),
                    ),
                    "int_ne" => Arc::from(
                        self.int_functional_evaluator
                            .int_ne(constraint, &self.solution),
                    ),
                    "int_ne_reif" => Arc::from(
                        self.int_functional_evaluator
                            .int_ne_reif(constraint, &self.solution),
                    ),
                    "int_pow" => Arc::from(
                        self.int_functional_evaluator
                            .int_pow(constraint, &self.solution),
                    ),
                    "int_times" => Arc::from(
                        self.int_functional_evaluator
                            .int_times(constraint, &self.solution),
                    ),
                    "array_bool_element" => Arc::from(
                        self.bool_functional_evaluator
                            .array_bool_element(constraint, &self.solution),
                    ),
                    "array_var_bool_element" => Arc::from(
                        self.bool_functional_evaluator
                            .array_bool_element(constraint, &self.solution),
                    ),
                    "array_bool_xor" => {
                        Arc::from(self.bool_functional_evaluator.array_bool_xor(constraint))
                    }
                    "bool_and" => Arc::from(
                        self.bool_functional_evaluator
                            .bool_and(constraint, &self.solution),
                    ),
                    "bool_clause" => {
                        Arc::from(self.bool_functional_evaluator.bool_clause(constraint))
                    }
                    "bool_eq" => Arc::from(
                        self.bool_functional_evaluator
                            .bool_eq(constraint, &self.solution),
                    ),
                    "bool_eq_reif" => Arc::from(
                        self.bool_functional_evaluator
                            .bool_eq_reif(constraint, &self.solution),
                    ),
                    "bool_le" => Arc::from(
                        self.bool_functional_evaluator
                            .bool_le(constraint, &self.solution),
                    ),
                    "bool_le_reif" => Arc::from(
                        self.bool_functional_evaluator
                            .bool_le_reif(constraint, &self.solution),
                    ),
                    "bool_lin_eq" => {
                        Arc::from(self.bool_functional_evaluator.bool_lin_eq(constraint))
                    }
                    "bool_lin_le" => {
                        Arc::from(self.bool_functional_evaluator.bool_lin_le(constraint))
                    }
                    "bool_lt" => Arc::from(
                        self.bool_functional_evaluator
                            .bool_lt(constraint, &self.solution),
                    ),
                    "bool_lt_reif" => Arc::from(
                        self.bool_functional_evaluator
                            .bool_lt_reif(constraint, &self.solution),
                    ),
                    "bool_not" => Arc::from(
                        self.bool_functional_evaluator
                            .bool_not(constraint, &self.solution),
                    ),
                    "bool_or" => Arc::from(
                        self.bool_functional_evaluator
                            .bool_or(constraint, &self.solution),
                    ),
                    "bool_xor" => Arc::from(
                        self.bool_functional_evaluator
                            .bool_xor(constraint, &self.solution),
                    ),
                    "bool2int" => Arc::from(
                        self.bool_functional_evaluator
                            .bool2int(constraint, &self.solution),
                    ),
                    "array_set_element" => Arc::from(
                        self.set_functional_evaluator
                            .array_set_element(constraint, &self.solution),
                    ),
                    "set_card" => Arc::from(
                        self.set_functional_evaluator
                            .set_card(constraint, &self.solution),
                    ),
                    "set_diff" => Arc::from(
                        self.set_functional_evaluator
                            .set_diff(constraint, &self.solution),
                    ),
                    "set_eq" => Arc::from(
                        self.set_functional_evaluator
                            .set_eq(constraint, &self.solution),
                    ),
                    "set_eq_reif" => Arc::from(
                        self.set_functional_evaluator
                            .set_eq_reif(constraint, &self.solution),
                    ),
                    "set_in" => Arc::from(
                        self.set_functional_evaluator
                            .set_in(constraint, &self.solution),
                    ),
                    "set_in_reif" => Arc::from(
                        self.set_functional_evaluator
                            .set_in_reif(constraint, &self.solution),
                    ),
                    "set_intersect" => Arc::from(
                        self.set_functional_evaluator
                            .set_intersect(constraint, &self.solution),
                    ),
                    "set_le" => Arc::from(
                        self.set_functional_evaluator
                            .set_le(constraint, &self.solution),
                    ),
                    "set_le_reif" => Arc::from(
                        self.set_functional_evaluator
                            .set_le_reif(constraint, &self.solution),
                    ),
                    "set_lt" => Arc::from(
                        self.set_functional_evaluator
                            .set_lt(constraint, &self.solution),
                    ),
                    "set_lt_reif" => Arc::from(
                        self.set_functional_evaluator
                            .set_lt_reif(constraint, &self.solution),
                    ),
                    "set_ne" => Arc::from(
                        self.set_functional_evaluator
                            .set_ne(constraint, &self.solution),
                    ),
                    "set_ne_reif" => Arc::from(
                        self.set_functional_evaluator
                            .set_ne_reif(constraint, &self.solution),
                    ),
                    "set_subset" => Arc::from(
                        self.set_functional_evaluator
                            .set_subset(constraint, &self.solution),
                    ),
                    "set_subset_reif" => Arc::from(
                        self.set_functional_evaluator
                            .set_subset_reif(constraint, &self.solution),
                    ),
                    "set_superset" => Arc::from(
                        self.set_functional_evaluator
                            .set_superset(constraint, &self.solution),
                    ),
                    "set_superset_reif" => Arc::from(
                        self.set_functional_evaluator
                            .set_superset_reif(constraint, &self.solution),
                    ),
                    "set_symdiff" => Arc::from(
                        self.set_functional_evaluator
                            .set_symdiff(constraint, &self.solution),
                    ),
                    "set_union" => Arc::from(
                        self.set_functional_evaluator
                            .set_union(constraint, &self.solution),
                    ),
                    _ => Arc::new(|_sol: &HashMap<String, VariableValue>| 0.0),
                };

            self.violation_functions.push(func_arc);
        }
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
                        _ => panic!("Objective variable must be numeric"),
                    }
                }
            }
            return (objective, constraint_violation);
        }

        let vf_ref: &Vec<Arc<dyn Fn(&HashMap<String, VariableValue>) -> f64 + Send + Sync>> =
            &self.violation_functions;
        let solution_ref: &HashMap<String, VariableValue> = &self.solution;
        for constraint in constraint_to_evaluate.iter() {
            if constraint.call.ann.is_empty() {
                let idx = constraint.id;
                let violation = (vf_ref[idx])(solution_ref);
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
                    _ => panic!("Objective variable must be numeric"),
                }
            }
        }

        (objective, constraint_violation)
    }

    fn evaluate_domain_constraints(&mut self) -> f64 {
        let mut violation = 0.0;

        for (var_id, var) in &self.fzn.variables {
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
                        _ => panic!(
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
                            panic!("No domain for int variable `{}`", identifier);
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
                            _ => panic!("Non-integer domain for int variable `{}`", identifier),
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
                            panic!("No domain for float variable `{}`", identifier);
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
                            _ => panic!("Non-floating domain for float variable `{}`", identifier),
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

        let defines_vec: Vec<Option<Identifier>> = raw
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

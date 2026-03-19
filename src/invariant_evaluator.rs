use crate::data_utility::data_utility::ConstraintEvaluation;
use crate::invariant_evaluator_sub_types::bool_invariant_evaluator::BoolInvariantEvaluator;
use crate::invariant_evaluator_sub_types::float_invariant_evaluator::FloatInvariantEvaluator;
use crate::invariant_evaluator_sub_types::int_invariant_evaluator::IntInvariantEvaluator;
use crate::invariant_evaluator_sub_types::set_invariant_evaluator::SetInvariantEvaluator;
use crate::invariant_graph::InvariantGraph;
use crate::solution_provider::{SolutionProvider, VariableValue};
use crate::variable_assigner::VariableAssigner;
use env_logger::Env;
use flatzinc_serde::{Array, Call, FlatZinc, Identifier, Literal};
use log::info;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[derive(Debug, Clone, Default)]
pub struct InvariantEvaluator {
    fzn: FlatZinc,
    constraints: Vec<CallWithDefines>,
    graph: InvariantGraph,
    verbose: bool,
    arrays_hashmap: HashMap<Identifier, Array>,
    int_invariant_evaluator: IntInvariantEvaluator,
    float_invariant_evaluator: FloatInvariantEvaluator,
    bool_invariant_evaluator: BoolInvariantEvaluator,
    set_invariant_evaluator: SetInvariantEvaluator,
    solution: HashMap<String, VariableValue>,
}

impl InvariantEvaluator {
    pub fn new(path: &Path, fzn: FlatZinc, option: Option<&str>) -> Self {
        let mut constraints = Self::load_constraints_with_defines(path, &fzn);
        let arrays_hashmap: HashMap<Identifier, Array> = fzn.arrays.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let graph = InvariantGraph::build(&constraints, &arrays_hashmap);
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
        
        let int_invariant_evaluator = IntInvariantEvaluator::new(arrays_hashmap.clone(), verbose);
        let float_invariant_evaluator = FloatInvariantEvaluator::new(arrays_hashmap.clone(), verbose);
        let bool_invariant_evaluator = BoolInvariantEvaluator::new(arrays_hashmap.clone(), verbose);
        let set_invariant_evaluator = SetInvariantEvaluator::new(arrays_hashmap.clone(), verbose);

        Self {
            fzn,
            constraints,
            graph,
            verbose,
            arrays_hashmap,
            int_invariant_evaluator,
            float_invariant_evaluator,
            bool_invariant_evaluator,
            set_invariant_evaluator,
            solution: HashMap::new(),
        }
    }

    pub fn evaluate_constraints(&mut self, solution_provider: &SolutionProvider, constraints_to_eval: &[CallWithDefines]) -> (Option<f64>, f64){
        let defined_variables: Vec<String> = solution_provider.defined_vars_map().iter().cloned().collect();
        let mut variable_assigner = VariableAssigner::new(
            solution_provider.solution_map().clone(),
            defined_variables,
            self.constraints.clone(),
            self.arrays_hashmap.clone(),
        );
        self.solution = variable_assigner.search_defined_var_in_constraints();
        let constraint_list: Vec<_> = constraints_to_eval
            .iter()
            .cloned()
            .collect();

        self.evaluate_constraint_list(&constraint_list)
    }

    pub fn evaluate_invariants_graph(&mut self, solution_provider: &SolutionProvider) -> (Option<f64>, f64) {
        let start_total = std::time::Instant::now();
        let defined_variables: Vec<String> = solution_provider.defined_vars_map().iter().cloned().collect();
        
        let start_assigner = std::time::Instant::now();
        let mut variable_assigner = VariableAssigner::new(
            solution_provider.solution_map().clone(),
            defined_variables,
            self.constraints.clone(),
            self.arrays_hashmap.clone(),
        );
        self.solution = variable_assigner.search_defined_var_in_constraints();
        let assigner_time = start_assigner.elapsed().as_micros();

        self.graph.clear_evaluations();

        // let constraint_list: Vec<_> = self
        //     .constraints
        //     .iter()
        //     .filter(|e| e.defines.is_none())
        //     .cloned()
        //     .collect();

        let start_parallel = std::time::Instant::now();
        let (objective, constraint_violation) = self.evaluate_constraint_list(&self.constraints.clone());
        let parallel_time = start_parallel.elapsed().as_micros();
        let total_time = start_total.elapsed().as_micros();
        
        static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
        static TOTAL_ASSIGNER_TIME: AtomicU64 = AtomicU64::new(0);
        static TOTAL_PARALLEL_TIME: AtomicU64 = AtomicU64::new(0);
        static TOTAL_OVERALL_TIME: AtomicU64 = AtomicU64::new(0);
        
        CALL_COUNT.fetch_add(1, Ordering::Relaxed);
        TOTAL_ASSIGNER_TIME.fetch_add(assigner_time as u64, Ordering::Relaxed);
        TOTAL_PARALLEL_TIME.fetch_add(parallel_time as u64, Ordering::Relaxed);
        TOTAL_OVERALL_TIME.fetch_add(total_time as u64, Ordering::Relaxed);

        (objective, constraint_violation)
    }

    fn evaluate_constraint_list(&mut self, constraint_to_evaluate: &[CallWithDefines]) -> (Option<f64>, f64) {
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

        let mut num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        if num_threads == 0 {
            num_threads = 1;
        }
        
        num_threads = std::cmp::min(num_threads, len);
        let chunk_size = (len + num_threads - 1) / num_threads;
        
        let solution_arc = Arc::new(self.solution.clone());
        let int_eval_arc = Arc::new(self.int_invariant_evaluator.clone());
        let float_eval_arc = Arc::new(self.float_invariant_evaluator.clone());
        let bool_eval_arc = Arc::new(self.bool_invariant_evaluator.clone());
        let set_eval_arc = Arc::new(self.set_invariant_evaluator.clone());
        
        let mut collected: Vec<(usize, ConstraintEvaluation)> = Vec::with_capacity(len);

        std::thread::scope(|s| {
            let mut handles = Vec::with_capacity(num_threads);
            for chunk in constraint_to_evaluate.chunks(chunk_size) {
                let chunk_vec = chunk.to_vec();
                let solution_ref = Arc::clone(&solution_arc);
                let int_ref = Arc::clone(&int_eval_arc);
                let float_ref = Arc::clone(&float_eval_arc);
                let bool_ref = Arc::clone(&bool_eval_arc);
                let set_ref = Arc::clone(&set_eval_arc);

                handles.push(s.spawn(move || {
                    let mut local_results: Vec<(usize, ConstraintEvaluation)> = Vec::new();

                    for constraint in chunk_vec.into_iter() {
                        if constraint.call.ann.is_empty() {
                            let id = constraint.call.id.as_str();
                            
                            let eval: ConstraintEvaluation = if id == "int2float" {
                                float_ref.int2float(&constraint, &solution_ref)
                            } else if id == "bool2int" {
                                bool_ref.bool2int(&constraint, &solution_ref)
                            } else if id.contains("float") {
                                Self::dispatch_float_predicate(&float_ref, &constraint, &solution_ref)
                            } else if id.contains("bool") {
                                Self::dispatch_bool_predicate(&bool_ref, &constraint, &solution_ref)
                            } else if id.contains("set") {
                                Self::dispatch_set_predicate(&set_ref, &constraint, &solution_ref)
                            } else if id.contains("int") {
                                Self::dispatch_int_predicate(&int_ref, &constraint, &solution_ref)
                            } else {
                                panic!("Missing predicate: {}", id);
                            };
                            
                            local_results.push((constraint.id, eval));
                        }
                    }

                    local_results
                }));
            }

            for handle in handles {
                if let Ok(mut res) = handle.join() {
                    collected.append(&mut res);
                }
            }
        });
        
        for (idx, eval) in collected.into_iter() {
            constraint_violation += eval.violation;
            self.graph.attach_evaluation_by_constraint_index(idx, eval);
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

    pub fn invariant_graph(&self) -> &InvariantGraph {
        &self.graph
    }

    // Static dispatch methods for parallel evaluation
    fn dispatch_int_predicate(
        evaluator: &IntInvariantEvaluator,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        match constraint.call.id.as_str() {
            "array_int_element" => evaluator.array_int_element(constraint, solution),
            "array_var_int_element" => evaluator.array_int_element(constraint, solution),
            "int_abs" => evaluator.int_abs(constraint, solution),
            "int_div" => evaluator.int_div(constraint, solution),
            "int_eq" => evaluator.int_eq(constraint, solution),
            "int_eq_reif" => evaluator.int_eq_reif(constraint, solution),
            "int_le" => evaluator.int_le(constraint, solution),
            "int_le_reif" => evaluator.int_le_reif(constraint, solution),
            "int_lin_eq" => evaluator.int_lin_eq(constraint, solution),
            "int_lin_eq_reif" => evaluator.int_lin_eq_reif(constraint, solution),
            "int_lin_le" => evaluator.int_lin_le(constraint, solution),
            "int_lin_le_reif" => evaluator.int_lin_le_reif(constraint, solution),
            "int_lin_ne" => evaluator.int_lin_ne(constraint, solution),
            "int_lin_ne_reif" => evaluator.int_lin_ne_reif(constraint, solution),
            "int_lt" => evaluator.int_lt(constraint, solution),
            "int_lt_reif" => evaluator.int_lt_reif(constraint, solution),
            "int_max" => evaluator.int_max(constraint, solution),
            "int_min" => evaluator.int_min(constraint, solution),
            "int_mod" => evaluator.int_mod(constraint, solution),
            "int_ne" => evaluator.int_ne(constraint, solution),
            "int_ne_reif" => evaluator.int_ne_reif(constraint, solution),
            "int_pow" => evaluator.int_pow(constraint, solution),
            "int_times" => evaluator.int_times(constraint, solution),
            other => panic!("Missing int predicate: {}", other),
        }
    }

    fn dispatch_float_predicate(
        evaluator: &FloatInvariantEvaluator,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        match constraint.call.id.as_str() {
            "array_float_element" => evaluator.array_float_element(constraint, solution),
            "array_var_float_element" => evaluator.array_float_element(constraint, solution),
            "float_abs" => evaluator.float_abs(constraint, solution),
            "float_acos" => evaluator.float_acos(constraint, solution),
            "float_acosh" => evaluator.float_acosh(constraint, solution),
            "float_asin" => evaluator.float_asin(constraint, solution),
            "float_asinh" => evaluator.float_asinh(constraint, solution),
            "float_atan" => evaluator.float_atan(constraint, solution),
            "float_atanh" => evaluator.float_atanh(constraint, solution),
            "float_cos" => evaluator.float_cos(constraint, solution),
            "float_cosh" => evaluator.float_cosh(constraint, solution),
            "float_div" => evaluator.float_div(constraint, solution),
            "float_eq" => evaluator.float_eq(constraint, solution),
            "float_eq_reif" => evaluator.float_eq_reif(constraint, solution),
            "float_exp" => evaluator.float_exp(constraint, solution),
            "float_le" => evaluator.float_le(constraint, solution),
            "float_le_reif" => evaluator.float_le_reif(constraint, solution),
            "float_lin_eq" => evaluator.float_lin_eq(constraint, solution),
            "float_lin_eq_reif" => evaluator.float_lin_eq_reif(constraint, solution),
            "float_lin_le" => evaluator.float_lin_le(constraint, solution),
            "float_lin_le_reif" => evaluator.float_lin_le_reif(constraint, solution),
            "float_lin_lt" => evaluator.float_lin_lt(constraint, solution),
            "float_lin_lt_reif" => evaluator.float_lin_lt_reif(constraint, solution),
            "float_lin_ne" => evaluator.float_lin_ne(constraint, solution),
            "float_lin_ne_reif" => evaluator.float_lin_ne_reif(constraint, solution),
            "float_ln" => evaluator.float_ln(constraint, solution),
            "float_log10" => evaluator.float_log10(constraint, solution),
            "float_log2" => evaluator.float_log2(constraint, solution),
            "float_lt" => evaluator.float_lt(constraint, solution),
            "float_lt_reif" => evaluator.float_lt_reif(constraint, solution),
            "float_max" => evaluator.float_max(constraint, solution),
            "float_min" => evaluator.float_min(constraint, solution),
            "float_ne" => evaluator.float_ne(constraint, solution),
            "float_ne_reif" => evaluator.float_ne_reif(constraint, solution),
            "float_plus" => evaluator.float_plus(constraint, solution),
            "float_pow" => evaluator.float_pow(constraint, solution),
            "float_sin" => evaluator.float_sin(constraint, solution),
            "float_sinh" => evaluator.float_sinh(constraint, solution),
            "float_sqrt" => evaluator.float_sqrt(constraint, solution),
            "float_tan" => evaluator.float_tan(constraint, solution),
            "float_tanh" => evaluator.float_tanh(constraint, solution),
            "float_times" => evaluator.float_times(constraint, solution),
            "int2float" => evaluator.int2float(constraint, solution),
            other => panic!("Missing float predicate: {}", other),
        }
    }

    fn dispatch_bool_predicate(
        evaluator: &BoolInvariantEvaluator,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        match constraint.call.id.as_str() {
            "array_bool_and" => evaluator.array_bool_and(constraint, solution),
            "array_bool_element" => evaluator.array_bool_element(constraint, solution),
            "array_bool_xor" => evaluator.array_bool_xor(constraint, solution),
            "array_var_bool_element" => evaluator.array_bool_element(constraint, solution),
            "bool_and" => evaluator.bool_and(constraint, solution),
            "bool_clause" => evaluator.bool_clause(constraint, solution),
            "bool_eq" => evaluator.bool_eq(constraint, solution),
            "bool_eq_reif" => evaluator.bool_eq_reif(constraint, solution),
            "bool_le" => evaluator.bool_le(constraint, solution),
            "bool_le_reif" => evaluator.bool_le_reif(constraint, solution),
            "bool_lin_eq" => evaluator.bool_lin_eq(constraint, solution),
            "bool_lin_le" => evaluator.bool_lin_le(constraint, solution),
            "bool_lt" => evaluator.bool_lt(constraint, solution),
            "bool_lt_reif" => evaluator.bool_lt_reif(constraint, solution),
            "bool_not" => evaluator.bool_not(constraint, solution),
            "bool_or" => evaluator.bool_or(constraint, solution),
            "bool_xor" => evaluator.bool_xor(constraint, solution),
            "bool2int" => evaluator.bool2int(constraint, solution),
            other => panic!("Missing bool predicate: {}", other),
        }
    }

    fn dispatch_set_predicate(
        evaluator: &SetInvariantEvaluator,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        match constraint.call.id.as_str() {
            "array_set_element" => evaluator.array_set_element(constraint, solution),
            "array_var_set_element" => evaluator.array_set_element(constraint, solution),
            "set_card" => evaluator.set_card(constraint, solution),
            "set_diff" => evaluator.set_diff(constraint, solution),
            "set_eq" => evaluator.set_eq(constraint, solution),
            "set_eq_reif" => evaluator.set_eq_reif(constraint, solution),
            "set_in" => evaluator.set_in(constraint, solution),
            "set_in_reif" => evaluator.set_in_reif(constraint, solution),
            "set_intersect" => evaluator.set_intersect(constraint, solution),
            "set_le" => evaluator.set_le(constraint, solution),
            "set_le_reif" => evaluator.set_le_reif(constraint, solution),
            "set_lt" => evaluator.set_lt(constraint, solution),
            "set_lt_reif" => evaluator.set_lt_reif(constraint, solution),
            "set_ne" => evaluator.set_ne(constraint, solution),
            "set_ne_reif" => evaluator.set_ne_reif(constraint, solution),
            "set_subset" => evaluator.set_subset(constraint, solution),
            "set_subset_reif" => evaluator.set_subset_reif(constraint, solution),
            "set_superset" => evaluator.set_superset(constraint, solution),
            "set_superset_reif" => evaluator.set_superset_reif(constraint, solution),
            "set_symmetric_difference" => evaluator.set_symdiff(constraint, solution),
            "set_symdiff" => evaluator.set_symdiff(constraint, solution),
            "set_union" => evaluator.set_union(constraint, solution),
            other => panic!("Missing set predicate: {}", other),
        }
    }

    fn find_category_predicate(&self, id: &str) -> Result<&'static str, String> {
        if id == "int2float" {
            return Ok("float");
        }
        if id == "bool2int" {
            return Ok("bool");
        }
        if id.contains("float") {
            return Ok("float");
        }
        if id.contains("bool") {
            return Ok("bool");
        }
        if id.contains("set") {
            return Ok("set");
        }
        if id.contains("int") {
            return Ok("int");
        }

        Err(id.to_string())
    }

    fn evaluate_int_predicate(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        match constraint.call.id.as_str() {
            "array_int_element" => self
                .int_invariant_evaluator
                .array_int_element(&constraint, solution),
            "array_var_int_element" => self
                .int_invariant_evaluator
                .array_int_element(&constraint, solution),
            "int_abs" => self.int_invariant_evaluator.int_abs(&constraint, solution),
            "int_div" => self.int_invariant_evaluator.int_div(&constraint, solution),
            "int_eq" => self.int_invariant_evaluator.int_eq(&constraint, solution),
            "int_eq_reif" => self.int_invariant_evaluator.int_eq_reif(&constraint, solution),
            "int_le" => self.int_invariant_evaluator.int_le(&constraint, solution),
            "int_le_reif" => self.int_invariant_evaluator.int_le_reif(&constraint, solution),
            "int_lin_eq" => self.int_invariant_evaluator.int_lin_eq(&constraint, solution),
            "int_lin_eq_reif" => self.int_invariant_evaluator.int_lin_eq_reif(&constraint, solution),
            "int_lin_le" => self.int_invariant_evaluator.int_lin_le(&constraint, solution),
            "int_lin_le_reif" => self.int_invariant_evaluator.int_lin_le_reif(&constraint, solution),
            "int_lin_ne" => self.int_invariant_evaluator.int_lin_ne(&constraint, solution),
            "int_lin_ne_reif" => self.int_invariant_evaluator.int_lin_ne_reif(&constraint, solution),
            "int_lt" => self.int_invariant_evaluator.int_lt(&constraint, solution),
            "int_lt_reif" => self.int_invariant_evaluator.int_lt_reif(&constraint, solution),
            "int_max" => self.int_invariant_evaluator.int_max(&constraint, solution),
            "int_min" => self.int_invariant_evaluator.int_min(&constraint, solution),
            "int_mod" => self.int_invariant_evaluator.int_mod(&constraint, solution),
            "int_ne" => self.int_invariant_evaluator.int_ne(&constraint, solution),
            "int_ne_reif" => self.int_invariant_evaluator.int_ne_reif(&constraint, solution),
            "int_pow" => self.int_invariant_evaluator.int_pow(&constraint, solution),
            "int_times" => self.int_invariant_evaluator.int_times(&constraint, solution),
            other => panic!("Missing predicate: {}", other),
        }
    }

    fn evaluate_float_predicate(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        match constraint.call.id.as_str() {
            "array_float_element" => self
                .float_invariant_evaluator
                .array_float_element(&constraint, solution),
            "array_var_float_element" => self
                .float_invariant_evaluator
                .array_float_element(&constraint, solution),
            "float_abs" => self.float_invariant_evaluator.float_abs(&constraint, solution),
            "float_acos" => self.float_invariant_evaluator.float_acos(&constraint, solution),
            "float_acosh" => self.float_invariant_evaluator.float_acosh(&constraint, solution),
            "float_asin" => self.float_invariant_evaluator.float_asin(&constraint, solution),
            "float_asinh" => self.float_invariant_evaluator.float_asinh(&constraint, solution),
            "float_atan" => self.float_invariant_evaluator.float_atan(&constraint, solution),
            "float_atanh" => self.float_invariant_evaluator.float_atanh(&constraint, solution),
            "float_cos" => self.float_invariant_evaluator.float_cos(&constraint, solution),
            "float_cosh" => self.float_invariant_evaluator.float_cosh(&constraint, solution),
            "float_div" => self.float_invariant_evaluator.float_div(&constraint, solution),
            "float_eq" => self.float_invariant_evaluator.float_eq(&constraint, solution),
            "float_eq_reif" => self.float_invariant_evaluator.float_eq_reif(&constraint, solution),
            "float_exp" => self.float_invariant_evaluator.float_exp(&constraint, solution),
            "float_le" => self.float_invariant_evaluator.float_le(&constraint, solution),
            "float_le_reif" => self.float_invariant_evaluator.float_le_reif(&constraint, solution),
            "float_lin_eq" => self.float_invariant_evaluator.float_lin_eq(&constraint, solution),
            "float_lin_eq_reif" => self
                .float_invariant_evaluator
                .float_lin_eq_reif(&constraint, solution),
            "float_lin_le" => self.float_invariant_evaluator.float_lin_le(&constraint, solution),
            "float_lin_le_reif" => self
                .float_invariant_evaluator
                .float_lin_le_reif(&constraint, solution),
            "float_lin_lt" => self.float_invariant_evaluator.float_lin_lt(&constraint, solution),
            "float_lin_lt_reif" => self
                .float_invariant_evaluator
                .float_lin_lt_reif(&constraint, solution),
            "float_lin_ne" => self.float_invariant_evaluator.float_lin_ne(&constraint, solution),
            "float_lin_ne_reif" => self
                .float_invariant_evaluator
                .float_lin_ne_reif(&constraint, solution),
            "float_ln" => self.float_invariant_evaluator.float_ln(&constraint, solution),
            "float_log10" => self.float_invariant_evaluator.float_log10(&constraint, solution),
            "float_log2" => self.float_invariant_evaluator.float_log2(&constraint, solution),
            "float_lt" => self.float_invariant_evaluator.float_lt(&constraint, solution),
            "float_lt_reif" => self.float_invariant_evaluator.float_lt_reif(&constraint, solution),
            "float_max" => self.float_invariant_evaluator.float_max(&constraint, solution),
            "float_min" => self.float_invariant_evaluator.float_min(&constraint, solution),
            "float_ne" => self.float_invariant_evaluator.float_ne(&constraint, solution),
            "float_ne_reif" => self.float_invariant_evaluator.float_ne_reif(&constraint, solution),
            "float_plus" => self.float_invariant_evaluator.float_plus(&constraint, solution),
            "float_pow" => self.float_invariant_evaluator.float_pow(&constraint, solution),
            "float_sin" => self.float_invariant_evaluator.float_sin(&constraint, solution),
            "float_sinh" => self.float_invariant_evaluator.float_sinh(&constraint, solution),
            "float_sqrt" => self.float_invariant_evaluator.float_sqrt(&constraint, solution),
            "float_tan" => self.float_invariant_evaluator.float_tan(&constraint, solution),
            "float_tanh" => self.float_invariant_evaluator.float_tanh(&constraint, solution),
            "float_times" => self.float_invariant_evaluator.float_times(&constraint, solution),
            "int2float" => self.float_invariant_evaluator.int2float(&constraint, solution),
            other => panic!("Missing predicate: {}", other),
        }
    }

    fn evaluate_bool_predicate(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        match constraint.call.id.as_str() {
            "array_bool_and" => self.bool_invariant_evaluator.array_bool_and(&constraint, solution),
            "array_bool_element" => self
                .bool_invariant_evaluator
                .array_bool_element(&constraint, solution),
            "array_bool_xor" => self.bool_invariant_evaluator.array_bool_xor(&constraint, solution),
            "array_var_bool_element" => self
                .bool_invariant_evaluator
                .array_bool_element(&constraint, solution),
            "bool_and" => self.bool_invariant_evaluator.bool_and(&constraint, solution),
            "bool_clause" => self.bool_invariant_evaluator.bool_clause(&constraint, solution),
            "bool_eq" => self.bool_invariant_evaluator.bool_eq(&constraint, solution),
            "bool_eq_reif" => self.bool_invariant_evaluator.bool_eq_reif(&constraint, solution),
            "bool_le" => self.bool_invariant_evaluator.bool_le(&constraint, solution),
            "bool_le_reif" => self.bool_invariant_evaluator.bool_le_reif(&constraint, solution),
            "bool_lin_eq" => self.bool_invariant_evaluator.bool_lin_eq(&constraint, solution),
            "bool_lin_le" => self.bool_invariant_evaluator.bool_lin_le(&constraint, solution),
            "bool_lt" => self.bool_invariant_evaluator.bool_lt(&constraint, solution),
            "bool_lt_reif" => self.bool_invariant_evaluator.bool_lt_reif(&constraint, solution),
            "bool_not" => self.bool_invariant_evaluator.bool_not(&constraint, solution),
            "bool_or" => self.bool_invariant_evaluator.bool_or(&constraint, solution),
            "bool_xor" => self.bool_invariant_evaluator.bool_xor(&constraint, solution),
            "bool2int" => self.bool_invariant_evaluator.bool2int(&constraint, solution),
            other => panic!("Missing predicate: {}", other),
        }
    }

    fn evaluate_set_predicate(
        &self,
        constraint: &CallWithDefines,
        solution: &HashMap<String, VariableValue>,
    ) -> ConstraintEvaluation {
        match constraint.call.id.as_str() {
            "array_set_element" => self
                .set_invariant_evaluator
                .array_set_element(&constraint, solution),
            "array_var_set_element" => self
                .set_invariant_evaluator
                .array_set_element(&constraint, solution),
            "set_card" => self.set_invariant_evaluator.set_card(&constraint, solution),
            "set_diff" => self.set_invariant_evaluator.set_diff(&constraint, solution),
            "set_eq" => self.set_invariant_evaluator.set_eq(&constraint, solution),
            "set_eq_reif" => self.set_invariant_evaluator.set_eq_reif(&constraint, solution),
            "set_in" => self.set_invariant_evaluator.set_in(&constraint, solution),
            "set_in_reif" => self.set_invariant_evaluator.set_in_reif(&constraint, solution),
            "set_intersect" => self.set_invariant_evaluator.set_intersect(&constraint, solution),
            "set_le" => self.set_invariant_evaluator.set_le(&constraint, solution),
            "set_le_reif" => self.set_invariant_evaluator.set_le_reif(&constraint, solution),
            "set_lt" => self.set_invariant_evaluator.set_lt(&constraint, solution),
            "set_lt_reif" => self.set_invariant_evaluator.set_lt_reif(&constraint, solution),
            "set_ne" => self.set_invariant_evaluator.set_ne(&constraint, solution),
            "set_ne_reif" => self.set_invariant_evaluator.set_ne_reif(&constraint, solution),
            "set_subset" => self.set_invariant_evaluator.set_subset(&constraint, solution),
            "set_subset_reif" => self.set_invariant_evaluator.set_subset_reif(&constraint, solution),
            "set_superset" => self.set_invariant_evaluator.set_superset(&constraint, solution),
            "set_superset_reif" => self
                .set_invariant_evaluator
                .set_superset_reif(&constraint, solution),
            "set_symmetric_difference" => self.set_invariant_evaluator.set_symdiff(&constraint, solution),
            "set_symdiff" => self.set_invariant_evaluator.set_symdiff(&constraint, solution),
            "set_union" => self.set_invariant_evaluator.set_union(&constraint, solution),
            other => panic!("Missing set predicate: {}", other),
        }
    }

    fn evaluate_domain_constraints(
        &self,
        solution: HashMap<String, VariableValue>,
        variable_domains: BTreeMap<String, Vec<DomainItem>>,
    ) -> f64 {
        let mut violation: f64 = 0.0;

        for (variable, value) in &solution {
            if let Some(domain_items) = variable_domains.get(variable) {
                let v = match value {
                    VariableValue::Int(i) => self.evaluate_int_domain(*i, domain_items, variable),
                    VariableValue::Float(f) => {
                        self.evaluate_float_domain(*f, domain_items, variable)
                    }
                    VariableValue::Bool(b) => self.eval_bool_domain(*b, domain_items, variable),
                    VariableValue::Set(vec) => self.eval_set_domain(vec, domain_items, variable),
                };
                violation += v;
            }
        }

        violation
    }

    fn evaluate_int_domain(&self, v: i64, domain_items: &[DomainItem], variable: &str) -> f64 {
        const EPS: f64 = std::f64::EPSILON;
        let mut allowed = false;
        let mut nearest: Option<f64> = None;

        for item in domain_items {
            match item {
                DomainItem::IntSingleton(s) => {
                    if *s == v {
                        allowed = true;
                        break;
                    } else {
                        let d = ((*s - v).abs()) as f64;
                        nearest = Some(nearest.map_or(d, |p| p.min(d)));
                    }
                }
                DomainItem::IntRange(l, h) => {
                    if v >= *l && v <= *h {
                        allowed = true;
                        break;
                    } else {
                        let d = if v < *l {
                            (*l - v) as f64
                        } else {
                            (v - *h) as f64
                        };
                        nearest = Some(nearest.map_or(d, |p| p.min(d)));
                    }
                }
                DomainItem::FloatSingleton(sf) => {
                    let vf = v as f64;
                    if (vf - *sf).abs() <= EPS {
                        allowed = true;
                        break;
                    } else {
                        let d = (vf - *sf).abs();
                        nearest = Some(nearest.map_or(d, |p| p.min(d)));
                    }
                }
                DomainItem::FloatRange(lf, hf) => {
                    let vf = v as f64;
                    if vf >= *lf && vf <= *hf {
                        allowed = true;
                        break;
                    } else {
                        let d = if vf < *lf { *lf - vf } else { vf - *hf };
                        nearest = Some(nearest.map_or(d, |p| p.min(d)));
                    }
                }
                DomainItem::BoolSet(_) => {}
            }
        }

        if !allowed {
            let penalty = nearest.unwrap_or(1.0);
            if self.verbose {
                self.print_domain_repr(variable, domain_items);
            }
            penalty
        } else {
            0.0
        }
    }

    fn evaluate_float_domain(&self, v: f64, domain_items: &[DomainItem], variable: &str) -> f64 {
        const EPS: f64 = std::f64::EPSILON;
        let mut allowed = false;
        let mut nearest: Option<f64> = None;

        for item in domain_items {
            match item {
                DomainItem::FloatSingleton(s) => {
                    if (v - *s).abs() <= EPS {
                        allowed = true;
                        break;
                    } else {
                        let d = (v - *s).abs();
                        nearest = Some(nearest.map_or(d, |p| p.min(d)));
                    }
                }
                DomainItem::FloatRange(l, h) => {
                    if v >= *l && v <= *h {
                        allowed = true;
                        break;
                    } else {
                        let d = if v < *l { *l - v } else { v - *h };
                        nearest = Some(nearest.map_or(d, |p| p.min(d)));
                    }
                }
                DomainItem::IntSingleton(si) => {
                    if (v - (*si as f64)).abs() <= EPS {
                        allowed = true;
                        break;
                    } else {
                        let d = (v - (*si as f64)).abs();
                        nearest = Some(nearest.map_or(d, |p| p.min(d)));
                    }
                }
                DomainItem::IntRange(l, h) => {
                    let vf = v;
                    if vf >= (*l as f64) && vf <= (*h as f64) {
                        allowed = true;
                        break;
                    } else {
                        let d = if vf < (*l as f64) {
                            (*l as f64) - vf
                        } else {
                            vf - (*h as f64)
                        };
                        nearest = Some(nearest.map_or(d, |p| p.min(d)));
                    }
                }
                DomainItem::BoolSet(_) => {}
            }
        }

        if !allowed {
            let penalty = nearest.unwrap_or(1.0);
            if self.verbose {
                self.print_domain_repr(variable, domain_items);
            }
            penalty
        } else {
            0.0
        }
    }

    fn eval_bool_domain(&self, b: bool, domain_items: &[DomainItem], variable: &str) -> f64 {
        let mut allowed = false;

        for item in domain_items {
            match item {
                DomainItem::BoolSet(bs) => {
                    if bs.contains(&b) {
                        allowed = true;
                        break;
                    }
                }
                DomainItem::IntSingleton(si) => {
                    if (*si == 0 && !b) || (*si == 1 && b) {
                        allowed = true;
                        break;
                    }
                }
                _ => {}
            }
        }

        if !allowed {
            if self.verbose {
                self.print_domain_repr(variable, domain_items);
            }
            1.0
        } else {
            0.0
        }
    }

    fn eval_set_domain(
        &self,
        vec: &HashSet<i64>,
        domain_items: &[DomainItem],
        variable: &str,
    ) -> f64 {
        let mut sum_d = 0.0;
        for &elem in vec.iter() {
            let mut elem_allowed = false;
            let mut elem_nearest: Option<f64> = None;

            for item in domain_items {
                match item {
                    DomainItem::IntSingleton(s) => {
                        if elem == *s {
                            elem_allowed = true;
                            break;
                        } else {
                            let d = ((*s - elem).abs()) as f64;
                            elem_nearest = Some(elem_nearest.map_or(d, |p| p.min(d)));
                        }
                    }
                    DomainItem::IntRange(l, h) => {
                        if elem >= *l && elem <= *h {
                            elem_allowed = true;
                            break;
                        } else {
                            let d = if elem < *l {
                                (*l - elem) as f64
                            } else {
                                (elem - *h) as f64
                            };
                            elem_nearest = Some(elem_nearest.map_or(d, |p| p.min(d)));
                        }
                    }
                    DomainItem::FloatSingleton(sf) => {
                        let ef = elem as f64;
                        if (ef - *sf).abs() <= std::f64::EPSILON {
                            elem_allowed = true;
                            break;
                        } else {
                            let d = (ef - *sf).abs();
                            elem_nearest = Some(elem_nearest.map_or(d, |p| p.min(d)));
                        }
                    }
                    DomainItem::FloatRange(lf, hf) => {
                        let ef = elem as f64;
                        if ef >= *lf && ef <= *hf {
                            elem_allowed = true;
                            break;
                        } else {
                            let d = if ef < *lf { *lf - ef } else { ef - *hf };
                            elem_nearest = Some(elem_nearest.map_or(d, |p| p.min(d)));
                        }
                    }
                    DomainItem::BoolSet(_) => {}
                }
            }

            if !elem_allowed {
                sum_d += elem_nearest.unwrap_or(1.0);
            }
        }

        if sum_d > 0.0 && self.verbose {
            self.print_domain_repr(variable, domain_items);
        }

        sum_d
    }

    fn print_domain_repr(&self, variable: &str, domain_items: &[DomainItem]) {
        if let Some(first) = domain_items.first() {
            match first {
                DomainItem::IntRange(l, h) => self.print_verbose_output(
                    variable,
                    VariableValue::Int(*l),
                    VariableValue::Int(*h),
                ),
                DomainItem::IntSingleton(s) => self.print_verbose_output(
                    variable,
                    VariableValue::Int(*s),
                    VariableValue::Int(*s),
                ),
                DomainItem::FloatRange(lf, hf) => self.print_verbose_output(
                    variable,
                    VariableValue::Float(*lf),
                    VariableValue::Float(*hf),
                ),
                DomainItem::FloatSingleton(sf) => self.print_verbose_output(
                    variable,
                    VariableValue::Float(*sf),
                    VariableValue::Float(*sf),
                ),
                DomainItem::BoolSet(bs) => {
                    if bs.len() >= 2 {
                        self.print_verbose_output(
                            variable,
                            VariableValue::Bool(bs[0]),
                            VariableValue::Bool(bs[1]),
                        );
                    } else {
                        self.print_verbose_output(
                            variable,
                            VariableValue::Bool(bs[0]),
                            VariableValue::Bool(bs[0]),
                        );
                    }
                }
            }
        }
    }

    fn print_verbose_output(&self, variable: &str, l: VariableValue, h: VariableValue) {
        let fmt = |vv: &VariableValue| match vv {
            VariableValue::Int(i) => i.to_string(),
            VariableValue::Float(f) => f.to_string(),
            VariableValue::Bool(b) => b.to_string(),
            VariableValue::Set(s) => format!("{:?}", s),
        };
        let l_s = fmt(&l);
        let h_s = fmt(&h);
        info!("Violation: domain [{}, {}] for {}", l_s, h_s, variable);
    }

    fn load_variable_domains(fzn: &FlatZinc) -> BTreeMap<String, Vec<DomainItem>> {
        let mut result = BTreeMap::new();

        for (name, var_def) in &fzn.variables {
            if let Ok(val) = serde_json::to_value(var_def) {
                if let Some(domain_arr) = val.get("domain").and_then(|d| d.as_array()) {
                    let mut items: Vec<DomainItem> = Vec::new();

                    for entry in domain_arr {
                        if let Some(pair) = entry.as_array() {
                            if pair.len() == 2 {
                                if let (Some(l), Some(h)) = (pair[0].as_i64(), pair[1].as_i64()) {
                                    if l == h {
                                        items.push(DomainItem::IntSingleton(l));
                                    } else {
                                        items.push(DomainItem::IntRange(l, h));
                                    }
                                    continue;
                                }
                                if let (Some(lf), Some(hf)) = (pair[0].as_f64(), pair[1].as_f64()) {
                                    if (lf - hf).abs() <= std::f64::EPSILON {
                                        items.push(DomainItem::FloatSingleton(lf));
                                    } else {
                                        items.push(DomainItem::FloatRange(lf, hf));
                                    }
                                    continue;
                                }
                            }
                            for el in pair {
                                Self::store_singleton_value_domain(&mut items, el);
                            }
                            continue;
                        }
                    }

                    if !items.is_empty() {
                        result.insert(name.clone(), items);
                    }
                }
            }
        }

        result
    }

    fn store_singleton_value_domain(items: &mut Vec<DomainItem>, el: &Value) {
        if let Some(i) = el.as_i64() {
            items.push(DomainItem::IntSingleton(i));
        } else if let Some(f) = el.as_f64() {
            items.push(DomainItem::FloatSingleton(f));
        } else if let Some(b) = el.as_bool() {
            items.push(DomainItem::BoolSet(vec![b]));
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
                        Some(Value::String(st)) => {
                            if item == "X_INTRODUCED_647_" {
                            println!("Found defines for constraint: {}", item);
                        }
                            Some(st.clone())
                        }
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

#[derive(Debug, Clone)]
pub struct CallWithDefines {
    pub(crate) id: usize,
    pub(crate) call: Call,
    pub(crate) defines: Option<Identifier>,
}

#[derive(Debug, Clone)]
enum DomainItem {
    IntRange(i64, i64),
    IntSingleton(i64),
    FloatRange(f64, f64),
    FloatSingleton(f64),
    BoolSet(Vec<bool>),
}
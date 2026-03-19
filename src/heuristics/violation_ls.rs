// use std::arch::x86_64::_MM_FROUND_CUR_DIRECTION;
// use std::collections::HashSet;
// use std::hash::Hash;
// use std::{collections::HashMap, path::PathBuf};
// use std::fs::{self, File};
// use std::io::{BufReader, Read};
// use rand::RngExt;
// use rand::SeedableRng;
// use rand_chacha::ChaCha20Rng;
// use serde::Deserialize;

// use crate::data_utility::data_utility::ConstraintEvaluation;
// use crate::{invariant_evaluator::{CallWithDefines, InvariantEvaluator}, solution_provider::{SolutionProvider, VariableValue}};
// use flatzinc_serde::{Type, Domain, FlatZinc};
// use rand::seq::IteratorRandom; 

// const DEFAULT_SEED: u64 = 42;
// const WFIX: [i64; 2] = [145, 180];
// const HFIX: [i64; 2] = [145, 65];

// #[derive(Clone, Debug)]
// struct Jump {
//     var: String,
//     value: VariableValue,
//     score: i64,
// }

// #[derive(Deserialize, Debug)]
// struct Solution {
//     x: Vec<i64>,
//     y: Vec<i64>,
//     t: Vec<i64>,
//     n: i64,
// }


// pub struct ViolationLS{
//     variables_bounds: HashMap<String, (VariableValue, VariableValue)>,
//     variables_type: HashMap<String, Type>,
//     weights: Vec<f64>,
//     obj_coeff: f64,
//     solution_provider: SolutionProvider,
//     invariant_evaluator: InvariantEvaluator,
//     fzn_path: PathBuf,
//     ozn_path: PathBuf,
//     initial_solution_path: PathBuf,
//     assignament: HashMap<String, VariableValue>,
//     variables: Vec<String>,
//     set_variables: Vec<String>,
//     constraints: Vec<CallWithDefines>,
//     jump_tables: HashMap<String, Vec<Jump>>,
//     rng: ChaCha20Rng,
// }

// impl ViolationLS {

//     pub fn new(fzn_path: PathBuf, ozn_path: PathBuf, initial_solution_path: PathBuf) -> Self {
//         Self {
//             variables_bounds: HashMap::new(),
//             variables_type: HashMap::new(),
//             weights: Vec::new(),
//             obj_coeff: 1.0,
//             solution_provider: SolutionProvider::default(),
//             invariant_evaluator: InvariantEvaluator::default(),
//             fzn_path,
//             ozn_path,
//             initial_solution_path,
//             assignament: HashMap::new(),
//             variables: Vec::new(),
//             set_variables: Vec::new(),
//             constraints: Vec::new(),
//             jump_tables: HashMap::new(),
//             rng: ChaCha20Rng::seed_from_u64(DEFAULT_SEED),
//         }
//     }

//     pub fn initialize(&mut self) {
//         let mut s = String::new();
//         let file = File::open(&self.fzn_path).expect("Failed to open fzn file");
//         BufReader::new(file)
//             .read_to_string(&mut s)
//             .expect("Failed to read fzn file");
//         let s = s.strip_prefix("\u{feff}").unwrap_or(&s);
//         let fzn: FlatZinc = serde_json::from_str(s).expect("Failed to parse flatzinc json");

//         self.solution_provider = SolutionProvider::new(fzn.clone(), &self.ozn_path);
//         self.invariant_evaluator = InvariantEvaluator::new(&*self.fzn_path, fzn.clone(), None);

//         self.weights = vec![1.0; fzn.constraints.len()];

//         let undefined_vars: HashMap<String, &flatzinc_serde::Variable> = fzn.variables
//             .iter()
//             .filter(|(_, var)| !var.defined)
//             .map(|(name, var)| (name.clone(), var))
//             .collect();

//         let solution_content = match fs::read_to_string(&self.initial_solution_path){
//             Ok(content) => content,
//             Err(e) => panic!("Failed to read initial solution file {}: {}", self.initial_solution_path.display(), e),
//         };
//         let solution: Solution = serde_json::from_str(&solution_content).expect("Failed to parse solution json");

//         self.solution_provider.provide_array_of_int("x".to_string(), solution.x);
//         self.solution_provider.provide_array_of_int("y".to_string(), solution.y);
//         self.solution_provider.provide_array_of_int("t".to_string(), solution.t);
//         self.solution_provider.provide_int("n".to_string(), solution.n);

//         self.assignament = self.solution_provider.solution_map().iter()
//             .filter(|(var_name, _)| undefined_vars.contains_key(*var_name))
//             .filter(|(_, value)| matches!(value, VariableValue::Int(_) | VariableValue::Bool(_)))
//             .map(|(k, v)| (k.clone(), v.clone()))
//             .collect();

//         self.set_variables = undefined_vars.iter()
//             .filter(|(_, var)| matches!(var.ty, Type::IntSet))
//             .map(|(name, _)| name.clone())
//             .collect();

//         self.variables_bounds.clear();
//         for (var_name, var) in &undefined_vars {
//             let var_name_str = var_name.to_string();

//             match &var.ty {
//                 Type::Int => {
//                     self.variables_type.insert(var_name_str.clone(), Type::Int);
//                 }
//                 Type::Float => {
//                     self.variables_type.insert(var_name_str.clone(), Type::Float);
//                 }
//                 Type::Bool => {
//                     self.variables_type.insert(var_name_str.clone(), Type::Bool);
//                 }
//                 _ => { }
//             }


//             match &var.domain {
//                 Some(Domain::Int(range)) => {
//                     let min_v = *range.lower_bound().unwrap_or(&0);
//                     let max_v = *range.upper_bound().unwrap_or(&1000000);
//                     self.variables_bounds.insert(
//                         var_name_str,
//                         (VariableValue::Int(min_v), VariableValue::Int(max_v))
//                     );
//                 }
//                 Some(Domain::Float(range)) => {
//                     let min_v = *range.lower_bound().unwrap_or(&0.0);
//                     let max_v = *range.upper_bound().unwrap_or(&1000000.0);
//                     self.variables_bounds.insert(
//                         var_name_str,
//                         (VariableValue::Float(min_v), VariableValue::Float(max_v))
//                     );
//                 }
//                 None => {
//                     panic!("Variable {} has no domain defined", var_name_str);
//                 }
//                 _ => {
//                     panic!("Unsupported domain type for variable {}", var_name_str);
//                 }
//             }
//         }

//         self.variables = undefined_vars.iter()
//             .map(|(name, _)| name.to_string())
//             .collect();
        
//         self.rng = ChaCha20Rng::seed_from_u64(DEFAULT_SEED);

//         println!("Initial assignment: {:?}", self.assignament);
//         println!("Initial objective: {}", self.compute_obj());
//         self.invariant_evaluator.evaluate_invariants_graph(&self.solution_provider);
//     }

//     pub fn search(&mut self, time_limit: f64) {
//         let start_time = std::time::Instant::now();
//         let duration = std::time::Duration::from_secs_f64(time_limit);
//         self.invariant_evaluator.evaluate_invariants_graph(&self.solution_provider);

//         while start_time.elapsed() < duration {
//             let evaluations = self.invariant_evaluator.invariant_graph().get_constraint_evaluation_nodes();
//             let mut  vars_in_violated_constraints: HashSet<String> = HashSet::new();

//             // for eval in evaluations.iter().filter(|eval| eval.violation > 0.0) {
//             //     let constraint_vars = self.invariant_evaluator.invariant_graph().get_constraint_variables_by_index(eval.constraint_id);
//             //     for var in constraint_vars {
//             //         if !self.assignament.contains_key(&var) && self.set_variables.contains(&var) {
//             //             //check var has not set type and get the variables that define it
//             //            let defined_vars = self.invariant_evaluator.invariant_graph().get_recursive_definition_variables(&var);
//             //             for def_var in defined_vars {
//             //                 if self.assignament.contains_key(&def_var) {
//             //                     vars_in_violated_constraints.insert(def_var);
//             //                 }
//             //             }
//             //         }else{
//             //             vars_in_violated_constraints.insert(var);
//             //         }
//             //     }
//             // }

//             vars_in_violated_constraints.extend(
//         evaluations.iter()
//             .filter(|eval| eval.violation > 0.0)
//             .flat_map(|eval| {
//                 self.invariant_evaluator.invariant_graph()
//                     .get_constraint_variables_by_index(eval.constraint_id)
//                     .into_iter()
//                     .flat_map(|var| {
//                         if !self.assignament.contains_key(&var) || self.set_variables.contains(&var) {
//                             self.invariant_evaluator.invariant_graph()
//                                 .get_root_variables(&var)
//                                 .into_iter()
//                                 .filter(|def_var| self.assignament.contains_key(def_var))
//                                 .collect::<Vec<_>>()
//                         } else {
//                             vec![var]
//                         }
//                     })
//                 })
//             );
            
//             vars_in_violated_constraints.retain(|var| self.assignament.contains_key(var) || matches!(self.assignament.get(var), Some(VariableValue::Set(_))));
//             let var_to_sample = (3).min(self.assignament.len());
//             let mut var_to_change = Vec::with_capacity(var_to_sample);

//             if(vars_in_violated_constraints.is_empty()){
//                 var_to_change = self.variables.iter().cloned().sample(&mut self.rng, var_to_sample);
//                 println!("No violated constraints found. Current solution is feasible.");
//             }else{
//                 var_to_change = vars_in_violated_constraints.iter().cloned().sample(&mut self.rng, var_to_sample);
//             }

//             let mut jump_values: HashMap<String, Option<VariableValue>> = HashMap::new();
//             let mut current_obj = self.compute_obj();

//             while !var_to_change.is_empty() {
//                 match self.assignament.get(&var_to_change[0]){
//                     Some(VariableValue::Int(v)) => {
//                         let v = *v;
//                         let (jump_value, score) = self.ternary_search(&var_to_change[0], v, current_obj);
//                         if(jump_value != v){
//                             jump_values.insert(var_to_change[0].clone(), Some(VariableValue::Int(jump_value)));
//                             println!("Jumping variable {} from {} to {}, score: {}", var_to_change[0], v, jump_value, score);
//                         }else{
//                             jump_values.insert(var_to_change[0].clone(), None);
//                         }
//                         //println!("Current variable to change: {}, current value: {}", var_to_change[0], v);
//                     }
//                     Some(VariableValue::Bool(v)) => {
//                         let v = *v;
//                         let (jump_value, score) = self.flip_value(&var_to_change[0], v, current_obj);
//                         if jump_value != v {
//                             jump_values.insert(var_to_change[0].clone(), Some(VariableValue::Bool(jump_value)));
//                             println!("Flipping variable {} from {} to {}, score: {}", var_to_change[0], v, jump_value, score);
//                         } else {
//                             jump_values.insert(var_to_change[0].clone(), None);
//                         }
//                         println!("Current variable to change: {}, current value: {}", var_to_change[0], v);
//                     }
//                     None => {
//                         println!("Current variable to change: {}, current value not found in assignment", var_to_change[0]);
//                     }
//                     _ => {
//                         println!("Current variable to change: {}, current value has unsupported type", var_to_change[0]);
//                     }
//                 }
//                 var_to_change.remove(0);
            
//             }

//             let mut change_assignament = false;
//             let mut not_changed_vars = Vec::new();
//             for (var, jump_value) in jump_values {
//                 if jump_value.is_some() {
//                 let new_value = jump_value.unwrap();
//                 self.assignament.insert(var.clone(), new_value.clone());
//                 match new_value {
//                     VariableValue::Int(v) => self.solution_provider.provide_int(var.clone(), v),
//                     VariableValue::Bool(b) => self.solution_provider.provide_bool(var.clone(), b),
//                     _ => {}
//                 }
//                     change_assignament = true;
//                 }else{
//                     not_changed_vars.push(var.clone());            
//                 }
//             }

//             if (vars_in_violated_constraints.is_empty()){
//                     self.obj_coeff += 1.0;
//             }

//             if (not_changed_vars.len() == var_to_sample) {
//                 for var in not_changed_vars {
//                     let constraints = self.invariant_evaluator.invariant_graph().get_variable_constraint_nodes(&var);
//                     for constraint in constraints {
//                         self.weights[constraint.id] += 1.0//self.weights[constraint.id] * self.rng.random_range(0.0..1.0);
//                     }
//                 }
//                 if self.obj_coeff > 0.0 {
//                     self.obj_coeff -= 1.0;
//                 }
//             }

//             if(change_assignament){
//                 println!("assignament changed");
//                 let (_, violation ) = self.invariant_evaluator.evaluate_invariants_graph(&self.solution_provider);
//                 println!("new violation: {}", violation);
//                 if violation == 0.0{
//                     println!("New Feasibile solution: {:?}", self.assignament);
//                     println!("objective: {}", self.compute_obj());
//                 }
//                 current_obj = self.compute_obj();

//             }
//         }

//         println!("Final assignment: {:?}", self.assignament);
//         println!("Final objective: {}", self.compute_obj());
//         let (_, violation) = self.invariant_evaluator.evaluate_invariants_graph(&self.solution_provider);
//         println!("Final violation: {}", violation);
//     }

//     fn evaluate_int_score(&mut self, var: &str, v: i64, variable_constraints: &[CallWithDefines], current_evaluation: &[ConstraintEvaluation], current_obj: f64) -> f64 {
//         //println!("Variable {}: Value: {}", var, v);
//         self.solution_provider.provide_int(var.to_string(), v);
//         //println!("value in the solution to test: {:?}", self.solution_provider.solution_map().get(var));
//         self.invariant_evaluator.evaluate_constraints(&self.solution_provider, variable_constraints);
//         let evaluations = self.invariant_evaluator.invariant_graph().get_variable_constraint_evaluation_nodes(var);

//         let mut score_cons = 0.0;
//         for (idx, evaluation) in evaluations.iter().enumerate() {
//             let constraint_id = evaluation.constraint_id;
//             score_cons += self.weights[constraint_id] * (evaluation.violation - current_evaluation[idx].violation);
//         }
//         //println!("Constraint score: {}", score_cons);

//         self.assignament.insert(var.to_string(), VariableValue::Int(v));
//         //println!("Current objective: {}, new objective: {}", current_obj, self.compute_obj());
//         let score_obj = self.obj_coeff * (self.compute_obj() - current_obj);
//         //println!("Objective score: {}", score_obj);

//         score_cons + score_obj
//     }

//     fn ternary_search(&mut self, var: &str, var_value: i64, current_obj: f64) -> (i64, f64) {
//         let variable_constraints = self.invariant_evaluator.invariant_graph().get_variable_constraint_nodes(var);
//         //let current_evaluation = self.invariant_evaluator.invariant_graph().get_constraint_evaluation_nodes();
//         let current_evaluation = self.invariant_evaluator.invariant_graph().get_variable_constraint_evaluation_nodes(var);
//         let (min_v, max_v) = self.variables_bounds.get(var).expect("Variable not found in bounds");
//         let min_v = match min_v {
//             VariableValue::Int(v) => *v,
//             _ => panic!("Expected int variable"),
//         };
//         let max_v = match max_v {
//             VariableValue::Int(v) => *v,
//             _ => panic!("Expected int variable"),
//         };

//         let mut left = min_v;
//         let mut right = max_v;

//         while right - left > 2 {
//             let mid1 = left + (right - left) / 3;
//             let mid2 = right - (right - left) / 3;

//             let score1 = self.evaluate_int_score(var, mid1, &variable_constraints, &current_evaluation, current_obj);
//             let score2 = self.evaluate_int_score(var, mid2, &variable_constraints, &current_evaluation, current_obj);

//             if score1 < score2 {
//                 right = mid2;
//             } else {
//                 left = mid1;
//             }
//         }

//         let mut best_value = var_value;
//         let mut best_score = self.evaluate_int_score(var, var_value, &variable_constraints, &current_evaluation, current_obj);

//         for v in left..=right {
//             let score = self.evaluate_int_score(var, v, &variable_constraints, &current_evaluation, current_obj);
//             //println!("Evaluated value {}: score {}", v, score);
//             if score < best_score {
//                 best_score = score;
//                 best_value = v;
//             }
//         }

//         self.assignament.insert(var.to_string(), VariableValue::Int(var_value));
//         //println!("best value: {:?}, best score: {}", best_value, best_score);
//         (best_value, best_score)

//     }

//      fn evaluate_bool_score(&mut self, var: &str, v: bool, variable_constraints: &[CallWithDefines], current_evaluation: &[ConstraintEvaluation], current_obj: f64) -> f64 {
//         //println!("Variable {}: Value: {}", var, v);
//         self.solution_provider.provide_bool(var.to_string(), v);
//         //println!("value in the solution to test: {:?}", self.solution_provider.solution_map().get(var));
//         self.invariant_evaluator.evaluate_constraints(&self.solution_provider, variable_constraints);
//         let evaluations = self.invariant_evaluator.invariant_graph().get_variable_constraint_evaluation_nodes(var);

//         let mut score_cons = 0.0;
//         for (idx, evaluation) in evaluations.iter().enumerate() {
//             let constraint_id = evaluation.constraint_id;
//             score_cons += self.weights[constraint_id] * (evaluation.violation - current_evaluation[idx].violation);
//         }
//         //println!("Constraint score: {}", score_cons);

//         self.assignament.insert(var.to_string(), VariableValue::Bool(v));
//         //println!("Current objective: {}, new objective: {}", current_obj, self.compute_obj());
//         let score_obj = self.obj_coeff * (self.compute_obj() - current_obj);
//         //println!("Objective score: {}", score_obj);

//         score_cons + score_obj
//     }


//     fn flip_value(&mut self, var: &str, var_value: bool, current_obj: f64) -> (bool, f64) {
//         let variable_constraints = self.invariant_evaluator.invariant_graph().get_variable_constraint_nodes(var);
//         let current_evaluation = self.invariant_evaluator.invariant_graph().get_variable_constraint_evaluation_nodes(var);

//         let new_value = !var_value;
//         let best_score = self.evaluate_bool_score(var, var_value, &variable_constraints, &current_evaluation, current_obj);
//         let new_score = self.evaluate_bool_score(var, new_value, &variable_constraints, &current_evaluation, current_obj);

//         self.assignament.insert(var.to_string(), VariableValue::Bool(var_value));
//         if new_score < best_score {
//             (new_value, new_score)
//         } else {
//             (var_value, best_score)
//         }
//     }

//     fn compute_obj(&self) -> f64{
//         let x_vars: Vec<&str> = vec!["X_INTRODUCED_16_", "X_INTRODUCED_17_", "X_INTRODUCED_18_", "X_INTRODUCED_19_", "X_INTRODUCED_20_", "X_INTRODUCED_21_", "X_INTRODUCED_22_", "X_INTRODUCED_23_", "X_INTRODUCED_24_", "X_INTRODUCED_25_"];

//         let x_values: Vec<i64> = x_vars.iter().filter_map(|v| {
//             match self.assignament.get(*v) {
//                 Some(VariableValue::Int(val)) => Some(*val),
//                 _ => None,
//             }
//         }).collect();

//         let y_vars: Vec<&str> = vec!["X_INTRODUCED_26_", "X_INTRODUCED_27_", "X_INTRODUCED_28_", "X_INTRODUCED_29_", "X_INTRODUCED_30_", "X_INTRODUCED_31_", "X_INTRODUCED_32_", "X_INTRODUCED_33_", "X_INTRODUCED_34_", "X_INTRODUCED_35_"];
//         let y_values: Vec<i64> = y_vars.iter().filter_map(|v| {
//             match self.assignament.get(*v) {
//                 Some(VariableValue::Int(val)) => Some(*val),
//                 _ => None,
//             }
//         }).collect();

//         let t_vars: Vec<&str> = vec!["X_INTRODUCED_36_", "X_INTRODUCED_37_", "X_INTRODUCED_38_", "X_INTRODUCED_39_", "X_INTRODUCED_40_", "X_INTRODUCED_41_", "X_INTRODUCED_42_", "X_INTRODUCED_43_", "X_INTRODUCED_44_", "X_INTRODUCED_45_"];
//         let t_values: Vec<i64> = t_vars.iter().filter_map(|v| {
//             match self.assignament.get(*v) {
//                 Some(VariableValue::Int(val)) => Some(*val),
//                 _ => None,
//             }
//         }).collect();

//         let mut objective = 0.0;
//         let mut manhattan_distance_sum = 0;
//         let mut area_sum = 0;

//         for idx in 0..t_values.len(){
//             let t1 = t_values[idx];
//             if t1 != 0 && idx < x_values.len() && idx < y_values.len(){
//                 for (idx2) in (idx + 1)..t_values.len(){
//                     let t2 = t_values[idx2];
//                     if t2 != 0 && idx2 < x_values.len() && idx2 < y_values.len(){
//                         if(x_values[idx] != 2500 && x_values[idx + 1] != 2500 && y_values[idx] != 1500 && y_values[idx + 1] != 1500){
//                             let x1_center = x_values[idx] + WFIX[(t1-1) as usize] / 2;
//                             let y1_center = y_values[idx] + HFIX[(t1-1) as usize] / 2;
//                             let x2_center = x_values[idx2] + WFIX[(t2-1) as usize] / 2;
//                             let y2_center = y_values[idx2] + HFIX[(t2-1) as usize] / 2;
//                             manhattan_distance_sum += (x1_center - x2_center).abs() + (y1_center - y2_center).abs();
//                         }
//                     }
//                 }
        
//             }
//         }

//         for (idx, &t) in t_values.iter().enumerate(){
//             if t != 0{
//                 if(x_values[idx] != 2500 && y_values[idx] != 1500){
//                     area_sum += WFIX[(t-1) as usize] * HFIX[(t-1) as usize];
//                 }

//             }
//         }
//         -(manhattan_distance_sum as f64 + area_sum as f64)
//     }
// }
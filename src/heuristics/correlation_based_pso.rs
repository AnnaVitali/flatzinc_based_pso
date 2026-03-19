// use crate::invariant_evaluator::{CallWithDefines, InvariantEvaluator};
// use crate::solution_provider::{SolutionProvider, VariableValue};
// use flatzinc_serde::{Domain, FlatZinc, Identifier, Type, Variable};
// use rand::SeedableRng;
// use rand::{RngExt};
// use rand_chacha::ChaCha20Rng;
// use std::collections::{HashMap, HashSet};
// use std::fs::{File, OpenOptions};
// use std::io::{BufReader, Read, Write};
// use std::path::PathBuf;
// use std::time::{Duration, Instant};


// const DEFAULT_SEED: u64 = 42;
// const FEASIBILITY_TOL: f64 = 1e-3;

// #[derive(Clone)]
// pub struct CorrelationBasedParticle {
//     id: i64,
//     for_variable_violations: HashMap<String, f64>,
//     position: HashMap<String, VariableValue>,
//     velocity: HashMap<String, f64>,
//     variable_bounds: HashMap<String, (VariableValue, VariableValue)>,
//     int_var_names: HashSet<String>,
//     local_best_position: HashMap<String, VariableValue>,
//     local_best_obj: Option<f64>,
//     local_best_violation: f64,
//     solution_provider: SolutionProvider,
//     invariant_evaluator: InvariantEvaluator,
//     fzn_path: PathBuf,
//     ozn_path: PathBuf,
//     rng: ChaCha20Rng,
// }

// impl std::fmt::Debug for CorrelationBasedParticle {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("CorrelationBasedParticle")
//             .field("id", &self.id)
//             .field("position", &self.position)
//             .field("velocity", &self.velocity)
//             .field("local_best_position", &self.local_best_position)
//             .field("local_best_obj", &self.local_best_obj)
//             .field("local_best_violation", &self.local_best_violation)
//             .field("solution_provider", &self.solution_provider)
//             .field("invariant_evaluator", &self.invariant_evaluator)
//             .finish()
//     }
// }

// impl Default for CorrelationBasedParticle {
//     fn default() -> Self {
//         Self {
//             id: 0,
//             for_variable_violations: HashMap::new(),
//             position: HashMap::new(),
//             velocity: HashMap::new(),
//             variable_bounds: HashMap::new(),
//             int_var_names: HashSet::new(),
//             local_best_position: HashMap::new(),
//             local_best_obj: None,
//             local_best_violation: std::f64::INFINITY,
//             solution_provider: Default::default(),
//             invariant_evaluator: Default::default(),
//             fzn_path: Default::default(),
//             ozn_path: Default::default(),
//             rng: ChaCha20Rng::seed_from_u64(DEFAULT_SEED),
//         }
//     }
// }

// impl CorrelationBasedParticle {
//     pub fn new() -> Self {
//         Default::default()
//     }

//     pub fn initialize(&mut self, seed: i64, id: i64, fzn_path: PathBuf, ozn_path: PathBuf) {
//         self.id = id;
//         self.rng = ChaCha20Rng::seed_from_u64(seed as u64 + id as u64);
//         self.fzn_path = fzn_path;
//         self.ozn_path = ozn_path;
//         let mut s = String::new();
//         let file = File::open(&self.fzn_path).expect("Failed to open fzn file");
//         BufReader::new(file)
//             .read_to_string(&mut s)
//             .expect("Failed to read fzn file");
//         let s = s.strip_prefix("\u{feff}").unwrap_or(&s);
//         let fzn: FlatZinc = serde_json::from_str(s).expect("Failed to parse flatzinc json");

//         let undefined_vars: Vec<_> = fzn.variables
//             .iter()
//             .filter(|(_, var)| !var.defined)
//             .collect();

//         self.solution_provider = SolutionProvider::new(fzn.clone(), &self.ozn_path);
//         self.invariant_evaluator = InvariantEvaluator::new(&*self.fzn_path, fzn.clone(), None);

//         self.initialize_position_and_velocity(&undefined_vars);

//         let result = self.evaluate_current_violations();
//         self.local_best_position = self.position.clone();
//         self.local_best_obj = result.0;
//         self.local_best_violation = result.1;

//         for (var_name, _) in &self.position {
//             let violation = self.compute_for_variable_violation(var_name);
//             self.for_variable_violations
//                 .insert(var_name.clone(), violation);
//         }
//     }

//    fn initialize_position_and_velocity(&mut self, variables: &[(&Identifier, &Variable)]) {
//         for (identifier, variable) in variables {
//             let key = identifier.to_string();

//             match variable.ty {
//                 Type::Int => {
//                     let domain = variable
//                         .domain
//                         .as_ref()
//                         .unwrap_or_else(|| panic!("No domain for variable `{}`", identifier));
//                     match domain {
//                         Domain::Int(range) => {
//                             let min_v = *range.lower_bound().unwrap() as f64;
//                             let max_v = *range.upper_bound().unwrap() as f64;
//                             let x = VariableValue::Float(self.rng.random_range(min_v..max_v));
//                             let vel = self.rng.random_range(min_v..max_v);
//                             self.position.insert(key.clone(), x);
//                             self.velocity.insert(key.clone(), vel);
//                             self.variable_bounds.insert(
//                                 key.clone(),
//                                 (VariableValue::Float(min_v), VariableValue::Float(max_v)),
//                             );
//                             self.int_var_names.insert(key);
//                         }
//                         _ => panic!("Non-integer domain for int variable `{}`", identifier),
//                     };
//                 }
//                 Type::Bool => {
//                     let x = VariableValue::Bool(rand::random::<bool>());
//                     let vel = self.rng.random_range(0.0..1.0);
//                     self.position.insert(key.clone(), x);
//                     self.velocity.insert(key.clone(), vel);
//                     self.variable_bounds
//                         .insert(key, (VariableValue::Bool(false), VariableValue::Bool(true)));
//                 }
//                 Type::Float => {
//                     let domain = variable
//                         .domain
//                         .as_ref()
//                         .unwrap_or_else(|| panic!("No domain for variable `{}`", identifier));
//                     match domain {
//                         Domain::Float(range) => {
//                             let min_v = *range.lower_bound().unwrap();
//                             let max_v = *range.upper_bound().unwrap();
//                             let x = VariableValue::Float(self.rng.random_range(min_v..max_v));
//                             let vel = self.rng.random_range(min_v..max_v);
//                             self.position.insert(key.clone(), x);
//                             self.velocity.insert(key.clone(), vel);
//                             self.variable_bounds.insert(
//                                 key,
//                                 (VariableValue::Float(min_v), VariableValue::Float(max_v)),
//                             );
//                         }
//                         _ => panic!("Non-floating domain for float variable `{}`", identifier),
//                     };
//                 }

//                 _ => continue,
//             }
//         }
//     }

//     pub fn update_velocity_and_position(
//         &mut self,
//         global_best: &HashMap<String, VariableValue>,
//         w: f64,
//         c1: f64,
//         c2: f64,
//     ) {
        
//         let mut diff = 0.0;

//         let max_violation_denom = self.for_variable_violations.values().copied().fold(0./0., f64::max) + f64::EPSILON;

//         let vars_to_process: Vec<(String, VariableValue)> = if self.local_best_violation <= 0.001 {
//             self.position
//                 .iter()
//                 .map(|(name, val)| (name.clone(), val.clone()))
//                 .collect()
//         } else {
//             self.position
//                 .iter()
//                 .filter(|(name, _)| self.for_variable_violations.get(*name).copied().unwrap_or(0.0) > 0.0)
//                 .map(|(name, val)| (name.clone(), val.clone()))
//                 .collect()
//         };
        
//         for (var_name, curr_pos) in vars_to_process {
//             let var_key = var_name.clone();
            
//             let phi = self.for_variable_violations[&var_key] / max_violation_denom;
//             let w_adaptive = w* (1.0 + phi);
//             let r1 = self.rng.random_range(0.0..1.0);
//             let r2 = self.rng.random_range(0.0..1.0);
//             let curr_vel = self.velocity[&var_key];
//             let p_best = &self.local_best_position[&var_key];
//             let g_best = &global_best[&var_key];

//             let updated_velocity = match &curr_pos {

//                 VariableValue::Float(value) => {
//                     match (p_best, g_best) {
//                         (VariableValue::Float(p), VariableValue::Float(g)) => {
//                             w_adaptive * curr_vel + c1 * r1 * (p - value) + c2 * r2 * (g - value)
//                         }
//                         _ => panic!("Velocity update type mismatch for {}", var_key),
//                     }
//                 }
//                 VariableValue::Bool(value) => {
//                     match (p_best, g_best) {
//                         (VariableValue::Bool(p), VariableValue::Bool(g)) => {
//                             w_adaptive * curr_vel
//                                 + c1 * r1 * ((*p as i64 - *value as i64) as f64)
//                                 + c2 * r2 * ((*g as i64 - *value as i64) as f64)
//                         }
//                         _ => panic!("Velocity update type mismatch for {}", var_key),
//                     }
//                 }
//                 _ => panic!("Velocity update mismatch for {}", var_key),
//             };

//             self.velocity.insert(var_key.clone(), updated_velocity);

//             let updated_position = match &curr_pos {
//                 VariableValue::Float(value) => VariableValue::Float(value + updated_velocity),
//                 VariableValue::Bool(_) => {
//                     let p = 1.0 / (1.0 + (-updated_velocity).exp());
//                     VariableValue::Bool(rand::random::<f64>() < p)
//                 }
//                 _ => panic!("Position update mismatch for {}", var_key),
//             };

//             let updated_position = self.clamp_position_to_bounds(&var_key, updated_position);

//             diff += self.compute_difference(&curr_pos, &updated_position);
//             self.position.insert(var_key.clone(), updated_position);
//         }

//         if diff > 0.0 {
//             let (obj_position, pos_violations) = self.evaluate_current_violations();

//             if Self::is_better_candidate(
//                 obj_position,
//                 pos_violations,
//                 self.local_best_obj,
//                 self.local_best_violation,
//             ) {
//                 self.local_best_position = self.position.clone();
//                 self.local_best_violation = pos_violations;
//                 self.local_best_obj = obj_position;
//             }
//         }
//     }

//     fn is_better_candidate(
//         candidate_obj: Option<f64>,
//         candidate_violation: f64,
//         incumbent_obj: Option<f64>,
//         incumbent_violation: f64,
//     ) -> bool {
//         let candidate_feasible = candidate_violation <= FEASIBILITY_TOL;
//         let incumbent_feasible = incumbent_violation <= FEASIBILITY_TOL;

//         if candidate_feasible && !incumbent_feasible {
//             return true;
//         }
//         if !candidate_feasible && incumbent_feasible {
//             return false;
//         }

//         if candidate_feasible && incumbent_feasible {
//             match (candidate_obj, incumbent_obj) {
//                 (Some(c), Some(i)) => c < i,
//                 (Some(_), None) => true,
//                 _ => false,
//             }
//         } else {
//             candidate_violation < incumbent_violation
//         }
//     }

//     fn compute_for_variable_violation(&self, var_name: &str) -> f64 {
//         self.invariant_evaluator
//             .invariant_graph()
//             .get_variable_constraint_evaluation_nodes(var_name)
//             .iter()
//             .map(|eval| eval.violation)
//             .sum()
//     }

//     fn clamp_position_to_bounds(
//         &self,
//         var_name: &str,
//         new_position: VariableValue,
//     ) -> VariableValue {
//         match (
//             new_position,
//             self.variable_bounds.get(var_name),
//         ) {
//             (
//                 VariableValue::Float(position),
//                 Some(&(VariableValue::Float(min_v), VariableValue::Float(max_v))),
//             ) => {
//                 VariableValue::Float(position.clamp(min_v, max_v))
//             }
//             (
//                 VariableValue::Int(position),
//                 Some(&(VariableValue::Int(min_v), VariableValue::Int(max_v))),
//             ) => {
//                 VariableValue::Int(position.clamp(min_v, max_v))
//             }
//             (other, _) => other,
//         }
//     }

//     fn compute_difference(&self, old: &VariableValue, new: &VariableValue) -> f64 {
//         match (old, new) {
//             (VariableValue::Int(a), VariableValue::Int(b)) => (*a - *b).abs() as f64,
//             (VariableValue::Float(a), VariableValue::Float(b)) => (*a - *b).abs(),
//             (VariableValue::Bool(a), VariableValue::Bool(b)) => {
//                 (*a as i64 - *b as i64).abs() as f64
//             }
//             _ => 0.0,
//         }
//     }

//     pub(crate) fn evaluate_current_violations(&mut self) -> (Option<f64>, f64) {
//         for (var_name, var_value) in &self.position {
//             match var_value {
//                 VariableValue::Bool(val) => {
//                     self.solution_provider
//                         .provide_bool(var_name.to_string(), *val);
//                 }
//                 VariableValue::Float(val) => {
//                     if self.int_var_names.contains(var_name) {
//                         self.solution_provider
//                             .provide_int(var_name.to_string(), val.round() as i64);
//                     } else {
//                         self.solution_provider
//                             .provide_float(var_name.to_string(), *val);
//                     }
//                 }
//                 _ => panic!(
//                     "Unsupported variable type for violation evaluation: {}",
//                     var_name
//                 ),
//             }
//         }

//         let result = self
//             .invariant_evaluator
//             .evaluate_invariants_graph(&self.solution_provider);

//         result
//     }

//     pub fn local_best_position(&self) -> &HashMap<String, VariableValue> {
//         &self.local_best_position
//     }

//     pub fn local_best_obj(&self) -> Option<f64> {
//         self.local_best_obj
//     }

//     pub fn local_best_violation(&self) -> f64 {
//         self.local_best_violation
//     }

//     pub fn id(&self) -> i64 {
//         self.id
//     }
// }

// #[derive(Clone)]
// pub struct CorrelationGuidedPSO {
//     seed: i64,
//     max_time_seconds: f64,
//     w: f64,
//     c1: f64,
//     c2: f64,
//     temperature: f64,
//     cooling_rate: f64,
//     de_f: f64,
//     de_top_ratio: f64,
//     result_path: PathBuf,
//     swarm: Vec<CorrelationBasedParticle>,
//     global_best_position: HashMap<String, VariableValue>,
//     global_best_obj: Option<f64>,
//     global_best_violation: f64,
//     fzn_path: PathBuf,
//     ozn_path: PathBuf,
//     rng: ChaCha20Rng,
// }

// impl std::fmt::Debug for CorrelationGuidedPSO {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("MixedPSO")
//             .field("max_time_seconds", &self.max_time_seconds)
//             .field("w", &self.w)
//             .field("c1", &self.c1)
//             .field("c2", &self.c2)
//             .field("de_f", &self.de_f)
//             .field("de_top_ratio", &self.de_top_ratio)
//             .field("global_best_position", &self.global_best_position)
//             .field("global_best_obj", &self.global_best_obj)
//             .field("global_best_violation", &self.global_best_violation)
//             .field("obj_function", &"<fn>")
//             .finish()
//     }
// }

// impl CorrelationGuidedPSO {
//     pub fn new(
//         seed: i64,
//         swarm_size: i64,
//         max_time_seconds: f64,
//         w: f64,
//         c1: f64,
//         c2: f64,
//         cooling_rate: f64,
//         result_path: PathBuf,
//         fzn_path: PathBuf,
//         ozn_path: PathBuf,
//     ) -> Self {
//         let mut pso = Self {
//             seed,
//             max_time_seconds,
//             w,
//             c1,
//             c2,
//             temperature: std::f64::INFINITY,
//             cooling_rate,
//             de_f: 0.8,
//             de_top_ratio: 0.2,
//             result_path,
//             swarm: vec![CorrelationBasedParticle::new(); swarm_size as usize],
//             global_best_position: HashMap::new(),
//             global_best_obj: None,
//             global_best_violation: std::f64::INFINITY,
//             fzn_path,
//             ozn_path,
//             rng: ChaCha20Rng::seed_from_u64(DEFAULT_SEED ^ seed as u64),
//         };

//         pso.initialize();

//         pso
//     }

//     pub fn search(&mut self) {
//         let start_time = Instant::now();
//         let time_limit = Duration::from_secs_f64(self.max_time_seconds);
//         let mut iteration = 0;

//         println!("Starting search with time limit: {:.2} seconds", self.max_time_seconds);
//         let mut first_iteration = true;
//         loop {
//             if start_time.elapsed() >= time_limit {
//                 println!("Time limit reached after {} iterations", iteration);
//                 break;
//             }

//             let global_best_snapshot = self.global_best_position.clone();
//             for particle in &mut self.swarm {
//                 particle.update_velocity_and_position(
//                     &global_best_snapshot,
//                     self.w,
//                     self.c1,
//                     self.c2,
//                 );
//             }

//             if(first_iteration){
//                 //temperature is the worst violation
//                 self.temperature = self.swarm.iter()
//                     .map(|p| p.local_best_violation())
//                     .fold(0./0., f64::max);
//                 first_iteration = false;
//             }

//             for particle in &self.swarm {
//                 let proposed_obj = particle.local_best_obj().unwrap();
//                 let proposed_violation = particle.local_best_violation();

//                 let delta = (proposed_violation - self.global_best_violation) + (proposed_obj - self.global_best_obj.unwrap()); //(proposed_obj + proposed_violation) - (self.global_best_obj.unwrap() + self.global_best_violation);
//                         let probability = (-delta / self.temperature).exp();
//                         if rand::random_range(0.0..1.0) < probability {
//                             self.global_best_position = particle.local_best_position().clone();
//                             self.global_best_obj = particle.local_best_obj();
//                             self.global_best_violation = particle.local_best_violation();
//                         }
//                 // if particle.local_best_obj().is_some(){
                   

//                 //     if proposed_violation == 0.0 && self.global_best_violation == 0.0 {
//                 //         if proposed_obj < self.global_best_obj.unwrap() {
//                 //             self.global_best_position = particle.local_best_position().clone();
//                 //             self.global_best_obj = particle.local_best_obj();
//                 //             self.global_best_violation = particle.local_best_violation();
//                 //         }
//                 //     }else if proposed_violation <= self.global_best_violation {
//                 //         self.global_best_position = particle.local_best_position().clone();
//                 //         self.global_best_obj = particle.local_best_obj();
//                 //         self.global_best_violation = particle.local_best_violation();
//                 //     }else{
//                 //         let delta = (proposed_obj + proposed_violation) - (self.global_best_obj.unwrap() + self.global_best_violation);
//                 //         let probability = (-delta / self.temperature).exp();
//                 //         if rand::random_range(0.0..1.0) < probability {
//                 //             self.global_best_position = particle.local_best_position().clone();
//                 //             self.global_best_obj = particle.local_best_obj();
//                 //             self.global_best_violation = particle.local_best_violation();
//                 //         }
//                 //     }
//                 // }
//             }

//             self.temperature *= self.cooling_rate;
//             iteration += 1;

//             }

//             self.apply_de_on_best_particles();
//             self.refresh_global_best_from_swarm();
//             iteration += 1;

//         for particle in &self.swarm {
//             if CorrelationBasedParticle::is_better_candidate(
//                     particle.local_best_obj(),
//                     particle.local_best_violation(),
//                     self.global_best_obj,
//                     self.global_best_violation,
//                 ) {
//                     self.global_best_position = particle.local_best_position().clone();
//                     self.global_best_obj = particle.local_best_obj();
//                     self.global_best_violation = particle.local_best_violation();
//                 }
//         }

//         let elapsed = start_time.elapsed();
//         println!("Search completed in {:.3} seconds ({} iterations)", elapsed.as_secs_f64(), iteration);
//         println!("Best solution:\n {:?}", self.global_best_position);
//         println!("Best objective: {:?}", self.global_best_obj);
//         println!("Best violations: {:?}", self.global_best_violation);
//         if self.global_best_violation <= FEASIBILITY_TOL {
//            println!("{:?}", self.global_best_obj.unwrap_or(0.0));
//         } else {
//             println!("{:?}", self.global_best_violation);
//         }

//         // if let Err(err) = self.append_best_metrics() {
//         //     eprintln!(
//         //         "Failed to append best objective/violation to {}: {}",
//         //         self.result_path.display(),
//         //         err
//         //     );
//         // }
//     }

//     fn append_best_metrics(&self) -> std::io::Result<()> {
//         let mut file = OpenOptions::new()
//             .create(true)
//             .append(true)
//             .open(&self.result_path)?;

//         writeln!(
//             file,
//             "best_objective={:?}, best_violation={}",
//             self.global_best_obj,
//             self.global_best_violation
//         )?;

//         Ok(())
//     }

//     fn initialize(&mut self) {
//         let mut id = 0i64;
//         let mut worst_violation = 0.0;
//         for particle in &mut self.swarm {
//             particle.initialize(self.seed, id, self.fzn_path.clone(), self.ozn_path.clone());
//             id += 1;
//         }

//         let mut best_index = 0usize;
//         for (i, p) in self.swarm.iter().enumerate().skip(1) {
//             let best = &self.swarm[best_index];
//             if CorrelationBasedParticle::is_better_candidate(
//                 p.local_best_obj(),
//                 p.local_best_violation(),
//                 best.local_best_obj(),
//                 best.local_best_violation(),
//             ) {
//                 best_index = i;
//             }

//             if p.local_best_violation() > worst_violation {
//                 worst_violation = p.local_best_violation();
//             }
//         }

//         let best_particle = &self.swarm[best_index];
//         self.global_best_position = best_particle.local_best_position().clone();
//         self.global_best_obj = best_particle.local_best_obj();
//         self.global_best_violation = best_particle.local_best_violation();

//         println!("Simulated annealing violation guided PSO");
//         println!("Initial solution:\n {:?}", self.global_best_position);
//         println!("Initial objective: {:?}", self.global_best_obj);
//         println!("Initial violations: {:?}", self.global_best_violation);
//     }

//     fn to_numeric(value: &VariableValue) -> Option<f64> {
//         match value {
//             VariableValue::Float(v) => Some(*v),
//             VariableValue::Int(v) => Some(*v as f64),
//             VariableValue::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
//             _ => None,
//         }
//     }

//     fn clamp_numeric_to_bounds(
//         bounds: Option<&(VariableValue, VariableValue)>,
//         value: f64,
//     ) -> VariableValue {
//         match bounds {
//             Some((VariableValue::Float(min_v), VariableValue::Float(max_v))) => {
//                 VariableValue::Float(value.clamp(*min_v, *max_v))
//             }
//             Some((VariableValue::Int(min_v), VariableValue::Int(max_v))) => {
//                 let clamped = value.round().clamp(*min_v as f64, *max_v as f64);
//                 VariableValue::Int(clamped as i64)
//             }
//             Some((VariableValue::Bool(_), VariableValue::Bool(_))) => VariableValue::Bool(value >= 0.5),
//             _ => VariableValue::Float(value),
//         }
//     }

//     fn sample_two_distinct_indices(&mut self, exclude: usize) -> Option<(usize, usize)> {
//         let len = self.swarm.len();
//         if len < 3 {
//             return None;
//         }

//         loop {
//             let r1 = self.rng.random_range(0..len);
//             let r2 = self.rng.random_range(0..len);
//             if r1 != exclude && r2 != exclude && r1 != r2 {
//                 return Some((r1, r2));
//             }
//         }
//     }

//     fn refresh_global_best_from_swarm(&mut self) {
//         for particle in &self.swarm {
//             if CorrelationBasedParticle::is_better_candidate(
//                 particle.local_best_obj(),
//                 particle.local_best_violation(),
//                 self.global_best_obj,
//                 self.global_best_violation,
//             ) {
//                 self.global_best_position = particle.local_best_position().clone();
//                 self.global_best_obj = particle.local_best_obj();
//                 self.global_best_violation = particle.local_best_violation();
//             }
//         }
//     }

//     fn apply_de_on_best_particles(&mut self) {
//         let len = self.swarm.len();
//         if len < 3 {
//             return;
//         }

//         let mut indices: Vec<usize> = (0..len).collect();
//         indices.sort_by(|a, b| {
//             let pa = &self.swarm[*a];
//             let pb = &self.swarm[*b];
//             pa.local_best_violation()
//                 .total_cmp(&pb.local_best_violation())
//                 .then_with(|| {
//                     let oa = pa.local_best_obj().unwrap_or(f64::INFINITY);
//                     let ob = pb.local_best_obj().unwrap_or(f64::INFINITY);
//                     oa.total_cmp(&ob)
//                 })
//         });

//         let mut top_count = ((len as f64) * self.de_top_ratio).ceil() as usize;
//         if top_count == 0 {
//             top_count = 1;
//         }
//         top_count = top_count.min(len);

//         for idx in indices.into_iter().take(top_count) {
//             let Some((r1, r2)) = self.sample_two_distinct_indices(idx) else {
//                 continue;
//             };

//             let best_pos = self.swarm[idx].local_best_position().clone();
//             let pos_r1 = self.swarm[r1].position.clone();
//             let pos_r2 = self.swarm[r2].position.clone();

//             let mut trial = self.swarm[idx].position.clone();
//             for (var_name, base_val) in &best_pos {
//                 let Some(x_best) = Self::to_numeric(base_val) else {
//                     continue;
//                 };
//                 let Some(x_r1) = pos_r1.get(var_name).and_then(Self::to_numeric) else {
//                     continue;
//                 };
//                 let Some(x_r2) = pos_r2.get(var_name).and_then(Self::to_numeric) else {
//                     continue;
//                 };

//                 let mutant = x_best + self.de_f * (x_r1 - x_r2);
//                 let bounds = self.swarm[idx].variable_bounds.get(var_name);
//                 trial.insert(var_name.clone(), Self::clamp_numeric_to_bounds(bounds, mutant));
//             }

//             let old_position = self.swarm[idx].position.clone();
//             self.swarm[idx].position = trial.clone();
//             let (trial_obj, trial_violation) = self.swarm[idx].evaluate_current_violations();

//             if CorrelationBasedParticle::is_better_candidate(
//                 trial_obj,
//                 trial_violation,
//                 self.swarm[idx].local_best_obj,
//                 self.swarm[idx].local_best_violation,
//             ) {
//                 self.swarm[idx].local_best_position = trial;
//                 self.swarm[idx].local_best_obj = trial_obj;
//                 self.swarm[idx].local_best_violation = trial_violation;
//             } else {
//                 self.swarm[idx].position = old_position;
//             }
//         }
//     }
// }
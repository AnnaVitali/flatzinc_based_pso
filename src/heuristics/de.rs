// use crate::invariant_evaluator::InvariantEvaluator;
// use crate::solution_provider::{SolutionProvider, VariableValue};
// use flatzinc_serde::{Domain, FlatZinc, Identifier, Type, Variable};
// use rand::SeedableRng;
// use rand::RngExt;
// use rand_chacha::ChaCha20Rng;
// use std::collections::{HashMap, HashSet};
// use std::fs::{File, OpenOptions};
// use std::io::{BufReader, Read, Write};
// use std::path::PathBuf;
// use std::time::Instant;

// const DEFAULT_SEED: u64 = 42;

// #[derive(Clone)]
// struct DECandidate {
//     position: HashMap<String, VariableValue>,
//     obj: Option<f64>,
//     violation: f64,
// }

// #[derive(Clone)]
// pub struct DifferentialEvolution {
//     seed: i64,
//     population_size: usize,
//     max_iterations: usize,
//     f: f64,
//     cr: f64,
//     result_path: PathBuf,
//     fzn_path: PathBuf,
//     ozn_path: PathBuf,
//     variable_bounds: HashMap<String, (VariableValue, VariableValue)>,
//     int_var_names: HashSet<String>,
//     variable_names: Vec<String>,
//     population: Vec<DECandidate>,
//     best_position: HashMap<String, VariableValue>,
//     best_obj: Option<f64>,
//     best_violation: f64,
//     solution_provider: SolutionProvider,
//     invariant_evaluator: InvariantEvaluator,
//     rng: ChaCha20Rng,
// }

// impl std::fmt::Debug for DifferentialEvolution {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("DifferentialEvolution")
//             .field("population_size", &self.population_size)
//             .field("max_iterations", &self.max_iterations)
//             .field("f", &self.f)
//             .field("cr", &self.cr)
//             .field("best_obj", &self.best_obj)
//             .field("best_violation", &self.best_violation)
//             .finish()
//     }
// }

// impl DifferentialEvolution {
//     pub fn new(
//         seed: i64,
//         population_size: i64,
//         max_iterations: i64,
//         f: f64,
//         cr: f64,
//         result_path: PathBuf,
//         fzn_path: PathBuf,
//         ozn_path: PathBuf,
//     ) -> Self {
//         let mut s = String::new();
//         let file = File::open(&fzn_path).expect("Failed to open fzn file");
//         BufReader::new(file)
//             .read_to_string(&mut s)
//             .expect("Failed to read fzn file");
//         let s = s.strip_prefix("\u{feff}").unwrap_or(&s);
//         let fzn: FlatZinc = serde_json::from_str(s).expect("Failed to parse flatzinc json");

//         let mut variable_bounds: HashMap<String, (VariableValue, VariableValue)> = HashMap::new();
//         let mut int_var_names: HashSet<String> = HashSet::new();
//         let mut variable_names: Vec<String> = Vec::new();

//         let undefined_vars: Vec<_> = fzn
//             .variables
//             .iter()
//             .filter(|(_, var)| !var.defined)
//             .collect();

//         Self::initialize_bounds(
//             &undefined_vars,
//             &mut variable_bounds,
//             &mut int_var_names,
//             &mut variable_names,
//         );

//         let solution_provider = SolutionProvider::new(fzn.clone(), &ozn_path);
//         let invariant_evaluator = InvariantEvaluator::new(&fzn_path, fzn.clone(), None);

//         let pop_size = population_size.max(4) as usize;
//         let max_iters = max_iterations.max(1) as usize;

//         let mut de = Self {
//             seed,
//             population_size: pop_size,
//             max_iterations: max_iters,
//             f,
//             cr,
//             result_path,
//             fzn_path,
//             ozn_path,
//             variable_bounds,
//             int_var_names,
//             variable_names,
//             population: Vec::with_capacity(pop_size),
//             best_position: HashMap::new(),
//             best_obj: None,
//             best_violation: f64::INFINITY,
//             solution_provider,
//             invariant_evaluator,
//             rng: ChaCha20Rng::seed_from_u64(DEFAULT_SEED ^ seed as u64),
//         };

//         de.initialize_population();
//         de
//     }

//     fn initialize_bounds(
//         variables: &[(&Identifier, &Variable)],
//         variable_bounds: &mut HashMap<String, (VariableValue, VariableValue)>,
//         int_var_names: &mut HashSet<String>,
//         variable_names: &mut Vec<String>,
//     ) {
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
//                             variable_bounds.insert(
//                                 key.clone(),
//                                 (VariableValue::Float(min_v), VariableValue::Float(max_v)),
//                             );
//                             int_var_names.insert(key.clone());
//                             variable_names.push(key);
//                         }
//                         _ => panic!("Non-integer domain for int variable `{}`", identifier),
//                     }
//                 }
//                 Type::Bool => {
//                     variable_bounds.insert(
//                         key.clone(),
//                         (VariableValue::Bool(false), VariableValue::Bool(true)),
//                     );
//                     variable_names.push(key);
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
//                             variable_bounds.insert(
//                                 key.clone(),
//                                 (VariableValue::Float(min_v), VariableValue::Float(max_v)),
//                             );
//                             variable_names.push(key);
//                         }
//                         _ => panic!("Non-floating domain for float variable `{}`", identifier),
//                     }
//                 }
//                 _ => {}
//             }
//         }
//     }

//     fn initialize_population(&mut self) {
//         for _ in 0..self.population_size {
//             let position = self.random_position();
//             let (obj, violation) = self.evaluate_position(&position);
//             self.population.push(DECandidate {
//                 position,
//                 obj,
//                 violation,
//             });
//         }

//         self.refresh_best();
//     }

//     fn random_position(&mut self) -> HashMap<String, VariableValue> {
//         let mut position = HashMap::with_capacity(self.variable_names.len());
//         for var_name in &self.variable_names {
//             let bounds = self
//                 .variable_bounds
//                 .get(var_name)
//                 .unwrap_or_else(|| panic!("Missing bounds for {}", var_name));
//             let value = match bounds {
//                 (VariableValue::Float(min_v), VariableValue::Float(max_v)) => {
//                     VariableValue::Float(self.rng.random_range(*min_v..*max_v))
//                 }
//                 (VariableValue::Bool(_), VariableValue::Bool(_)) => {
//                     VariableValue::Bool(rand::random::<bool>())
//                 }
//                 _ => continue,
//             };
//             position.insert(var_name.clone(), value);
//         }
//         position
//     }

//     fn to_numeric(value: &VariableValue) -> Option<f64> {
//         match value {
//             VariableValue::Float(v) => Some(*v),
//             VariableValue::Int(v) => Some(*v as f64),
//             VariableValue::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
//             _ => None,
//         }
//     }

//     fn clamp_value_to_bounds(&self, var_name: &str, value: f64) -> VariableValue {
//         match self.variable_bounds.get(var_name) {
//             Some((VariableValue::Float(min_v), VariableValue::Float(max_v))) => {
//                 VariableValue::Float(value.clamp(*min_v, *max_v))
//             }
//             Some((VariableValue::Bool(_), VariableValue::Bool(_))) => {
//                 VariableValue::Bool(value >= 0.5)
//             }
//             _ => VariableValue::Float(value),
//         }
//     }

//     fn evaluate_position(&mut self, position: &HashMap<String, VariableValue>) -> (Option<f64>, f64) {
//         for (var_name, var_value) in position {
//             match var_value {
//                 VariableValue::Bool(val) => {
//                     self.solution_provider.provide_bool(var_name.to_string(), *val);
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
//                 VariableValue::Int(val) => {
//                     self.solution_provider.provide_int(var_name.to_string(), *val);
//                 }
//                 _ => {}
//             }
//         }

//         self.invariant_evaluator
//             .evaluate_invariants_graph(&self.solution_provider)
//     }

//     fn better_than(
//         candidate_obj: Option<f64>,
//         candidate_violation: f64,
//         incumbent_obj: Option<f64>,
//         incumbent_violation: f64,
//     ) -> bool {
//         if candidate_violation <= 1e-9 && incumbent_violation <= 1e-9 {
//             match (candidate_obj, incumbent_obj) {
//                 (Some(c), Some(i)) => c < i,
//                 (Some(_), None) => true,
//                 _ => false,
//             }
//         } else {
//             candidate_violation < incumbent_violation
//         }
//     }

//     fn refresh_best(&mut self) {
//         let mut best_index = 0usize;
//         for i in 1..self.population.len() {
//             let cand = &self.population[i];
//             let best = &self.population[best_index];
//             if Self::better_than(cand.obj, cand.violation, best.obj, best.violation) {
//                 best_index = i;
//             }
//         }

//         self.best_position = self.population[best_index].position.clone();
//         self.best_obj = self.population[best_index].obj;
//         self.best_violation = self.population[best_index].violation;
//     }

//     fn sample_three_distinct(&mut self, exclude: usize) -> (usize, usize, usize) {
//         loop {
//             let r1 = self.rng.random_range(0..self.population_size);
//             let r2 = self.rng.random_range(0..self.population_size);
//             let r3 = self.rng.random_range(0..self.population_size);
//             if r1 != exclude
//                 && r2 != exclude
//                 && r3 != exclude
//                 && r1 != r2
//                 && r1 != r3
//                 && r2 != r3
//             {
//                 return (r1, r2, r3);
//             }
//         }
//     }

//     fn iterate_once(&mut self) {
//         for i in 0..self.population_size {
//             let target = self.population[i].clone();
//             let (r1, r2, r3) = self.sample_three_distinct(i);

//             let base = self.population[r1].position.clone();
//             let diff_a = self.population[r2].position.clone();
//             let diff_b = self.population[r3].position.clone();

//             let mut trial = HashMap::with_capacity(self.variable_names.len());
//             let j_rand = self.rng.random_range(0..self.variable_names.len());

//             for (j, var_name) in self.variable_names.iter().enumerate() {
//                 let use_mutant = self.rng.random_range(0.0..1.0) < self.cr || j == j_rand;

//                 let value = if use_mutant {
//                     let x1 = base
//                         .get(var_name)
//                         .and_then(Self::to_numeric)
//                         .unwrap_or_else(|| panic!("Missing base value for {}", var_name));
//                     let x2 = diff_a
//                         .get(var_name)
//                         .and_then(Self::to_numeric)
//                         .unwrap_or_else(|| panic!("Missing diff_a value for {}", var_name));
//                     let x3 = diff_b
//                         .get(var_name)
//                         .and_then(Self::to_numeric)
//                         .unwrap_or_else(|| panic!("Missing diff_b value for {}", var_name));
//                     let mutant = x1 + self.f * (x2 - x3);
//                     self.clamp_value_to_bounds(var_name, mutant)
//                 } else {
//                     target
//                         .position
//                         .get(var_name)
//                         .cloned()
//                         .unwrap_or_else(|| panic!("Missing target value for {}", var_name))
//                 };

//                 trial.insert(var_name.clone(), value);
//             }

//             let (trial_obj, trial_violation) = self.evaluate_position(&trial);
//             if Self::better_than(trial_obj, trial_violation, target.obj, target.violation) {
//                 self.population[i] = DECandidate {
//                     position: trial,
//                     obj: trial_obj,
//                     violation: trial_violation,
//                 };
//             }
//         }

//         self.refresh_best();
//     }

//     pub fn search(&mut self) {
//         let start = Instant::now();
//         println!(
//             "Starting DE with population={}, iterations={}, F={}, CR={}",
//             self.population_size, self.max_iterations, self.f, self.cr
//         );

//         for _ in 0..self.max_iterations {
//             self.iterate_once();
//         }

//         let elapsed = start.elapsed().as_secs_f64();
//         println!("DE completed in {:.3} seconds", elapsed);
//         println!("Best solution:\n {:?}", self.best_position);
//         println!("Best objective: {:?}", self.best_obj);
//         println!("Best violations: {:?}", self.best_violation);
//         if self.best_violation <= 0.001 {
//             println!("{:?}", self.best_obj.unwrap_or(0.0));
//         } else {
//             println!("{:?}", self.best_violation);
//         }

//         if let Err(err) = self.append_best_metrics() {
//             eprintln!(
//                 "Failed to append best objective/violation to {}: {}",
//                 self.result_path.display(),
//                 err
//             );
//         }
//     }

//     fn append_best_metrics(&self) -> std::io::Result<()> {
//         let mut file = OpenOptions::new()
//             .create(true)
//             .append(true)
//             .open(&self.result_path)?;

//         writeln!(
//             file,
//             "best_objective={:?}, best_violation={}",
//             self.best_obj, self.best_violation
//         )?;

//         Ok(())
//     }
// }

use crate::data_utility::normalizer::Normalizer;
use crate::invariant_evaluator:: InvariantEvaluator;
use crate::solution_provider::{SolutionProvider, VariableValue};
use flatzinc_serde::{Domain, FlatZinc, Identifier, Type, Variable};
use rand::SeedableRng;
use rand::{RngExt};
use rand_chacha::ChaCha20Rng;
use serde::Deserialize;
use serde_json::from_str;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::PathBuf;


const DEFAULT_SEED: u64 = 42;
const FEASIBILITY_TOL: f64 = 1e-3;
const WFIX: [f64; 2] = [145.0, 180.0];
const HFIX: [f64; 2] = [145.0, 65.0];

#[derive(Deserialize, Debug)]struct Solution {
    x: Vec<i64>,
    y: Vec<i64>,
    t: Vec<i64>,
    n: i64,
}

pub fn is_better_candidate(
        candidate_obj: Option<f64>,
        candidate_violation: f64,
        incumbent_obj: Option<f64>,
        incumbent_violation: f64,
    ) -> bool {
        let candidate_feasible = candidate_violation <= FEASIBILITY_TOL;
        let incumbent_feasible = incumbent_violation <= FEASIBILITY_TOL;

        if candidate_feasible && !incumbent_feasible {
            return true;
        }
        if !candidate_feasible && incumbent_feasible {
            return false;
        }

        if candidate_feasible && incumbent_feasible {
            match (candidate_obj, incumbent_obj) {
                (Some(c), Some(i)) => c < i,
                (Some(_), None) => true,
                _ => false,
            }
        } else {
            candidate_violation < incumbent_violation
        }
    }


#[derive(Clone)]
pub struct PSOParticle {
    id: i64,
    normalizer: Normalizer,
    variables: Vec<(Identifier, Variable)>,
    for_variable_violations: HashMap<String, f64>,
    position: HashMap<String, f64>,
    velocity: HashMap<String, f64>,
    variable_bounds: HashMap<String, (VariableValue, VariableValue)>,
    int_var_names: HashSet<String>,
    local_best_position: HashMap<String, f64>,
    local_best_obj: Option<f64>,
    local_best_violation: f64,
    solution_provider: SolutionProvider,
    invariant_evaluator: InvariantEvaluator,
    fzn_path: PathBuf,
    ozn_path: PathBuf,
    rng: ChaCha20Rng,
}

impl std::fmt::Debug for PSOParticle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Particle")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("velocity", &self.velocity)
            .field("local_best_position", &self.local_best_position)
            .field("local_best_obj", &self.local_best_obj)
            .field("local_best_violation", &self.local_best_violation)
            .field("solution_provider", &self.solution_provider)
            .field("invariant_evaluator", &self.invariant_evaluator)
            // closure is not `Debug`, show placeholder
            .finish()
    }
}

impl Default for PSOParticle {
    fn default() -> Self {
        Self {
            id: 0,
            normalizer: Normalizer::default(),
            variables: Vec::new(),
            for_variable_violations: HashMap::new(),
            position: HashMap::new(),
            velocity: HashMap::new(),
            variable_bounds: HashMap::new(),
            int_var_names: HashSet::new(),
            local_best_position: HashMap::new(),
            local_best_obj: None,
            local_best_violation: std::f64::INFINITY,
            solution_provider: Default::default(),
            invariant_evaluator: Default::default(),
            fzn_path: Default::default(),
            ozn_path: Default::default(),
            rng: ChaCha20Rng::seed_from_u64(DEFAULT_SEED),
        }
    }
}

impl PSOParticle {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn initialize(&mut self, seed: i64, id: i64, fzn_path: PathBuf, ozn_path: PathBuf) {
        self.id = id;
        self.rng = ChaCha20Rng::seed_from_u64(seed as u64 + id as u64);
        self.fzn_path = fzn_path;
        self.ozn_path = ozn_path;
        let mut s = String::new();
        let file = File::open(&self.fzn_path).expect("Failed to open fzn file");
        BufReader::new(file)
            .read_to_string(&mut s)
            .expect("Failed to read fzn file");
        let s = s.strip_prefix("\u{feff}").unwrap_or(&s);
        let fzn: FlatZinc = from_str(s).expect("Failed to parse flatzinc json");
        self.solution_provider = SolutionProvider::new(fzn.clone(), &self.ozn_path);
        self.invariant_evaluator = InvariantEvaluator::new(&*self.fzn_path, fzn.clone(), None);

        self.variables = fzn.variables
            .iter()
            .filter(|(_, var)| !var.defined)
            .map(|(id, var)| (id.clone(), var.clone()))
            .collect();
    }

    pub fn set_initial_position_and_random_velocity(&mut self, initial_solution_path: &PathBuf) {
        let solution_content = match fs::read_to_string(initial_solution_path) {
            Ok(content) => content,
            Err(e) => panic!("Failed to read initial solution file {}: {}", initial_solution_path.display(), e),
        };
        let solution: Solution = from_str(&solution_content).expect("Failed to parse solution json");

        let x_array = solution.x.iter().map(|&v| v as f64).collect();
        let y_array = solution.y.iter().map(|&v| v as f64).collect();

        println!("Initial solution - X: {:?}, Y: {:?}", x_array, y_array);

        self.solution_provider.provide_array_of_float("x".to_string(), x_array);
        self.solution_provider.provide_array_of_float("y".to_string(), y_array);
        self.solution_provider.provide_array_of_int("t".to_string(), solution.t);
        self.solution_provider.provide_int("n".to_string(), solution.n);

        let denormalized_position = self
            .solution_provider
            .solution_map()
            .iter()
            .filter(|(var_name, _)| {
                self.variables.iter().any(|(id, _)| id.to_string() == **var_name)
            })
            .filter(|(_, value)| matches!(value, VariableValue::Int(_) | VariableValue::Bool(_) | VariableValue::Float(_)))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

         for (identifier, variable) in &self.variables {
            let key = identifier.to_string();

            match variable.ty {
                Type::Int => {
                    let domain = variable
                        .domain
                        .as_ref()
                        .unwrap_or_else(|| panic!("No domain for variable `{}`", identifier));
                    match domain {
                        Domain::Int(range) => {
                            let min_v = *range.lower_bound().unwrap();
                            let max_v = *range.upper_bound().unwrap();
                            let vel = self.rng.random_range(-0.1..0.1);
                            self.velocity.insert(key.clone(), vel);
                            self.variable_bounds.insert(
                                key.clone(),
                                (VariableValue::Int(min_v), VariableValue::Int(max_v)),
                            );
                        }
                        _ => panic!("Non-integer domain for int variable `{}`", identifier),
                    };
                }
                Type::Bool => {
                    let vel = rand::rng().random();
                    self.velocity.insert(key.clone(), vel);
                    self.variable_bounds
                        .insert(key, (VariableValue::Bool(false), VariableValue::Bool(true)));
                }
                Type::Float => {
                    let domain = variable
                        .domain
                        .as_ref()
                        .unwrap_or_else(|| panic!("No domain for variable `{}`", identifier));
                    match domain {
                        Domain::Float(range) => {
                            let min_v = *range.lower_bound().unwrap();
                            let max_v = *range.upper_bound().unwrap();
                            let vel = self.rng.random_range(-0.1..0.1);
                            self.velocity.insert(key.clone(), vel);
                            self.variable_bounds.insert(
                                key,
                                (VariableValue::Float(min_v), VariableValue::Float(max_v)),
                            );
                        }
                        _ => panic!("Non-floating domain for float variable `{}`", identifier),
                    };
                }
                _ => continue,
            }
        }

        self.normalizer = Normalizer::new(self.variable_bounds.clone());
        let mut feasibile_position = self.normalizer.normalize(&denormalized_position);

        for (var, val) in feasibile_position.iter_mut() {
            let noise = self.rng.random_range(-0.05..0.05); // 5% noise
            *val = (*val + noise).clamp(0.0, 1.0);
            self.position.insert(var.clone(), *val);
        }

        self.local_best_position = self.position.clone();
        let perturbed_denormalized_position = self.normalizer.denormalize(&self.position);
        let denormalized_repaired = self.repair_solution(&perturbed_denormalized_position);
        self.local_best_obj = self.compute_obj(&denormalized_repaired).into();
        self.local_best_violation = self.evaluate_current_violations(&denormalized_repaired).1;
    }
 

    pub fn random_initialize_position_and_velocity(&mut self) {
        for (identifier, variable) in &self.variables {
            let key = identifier.to_string();

            match variable.ty {
                Type::Int => {
                    let domain = variable
                        .domain
                        .as_ref()
                        .unwrap_or_else(|| panic!("No domain for variable `{}`", identifier));
                    match domain {
                        Domain::Int(range) => {
                            let min_v = *range.lower_bound().unwrap();
                            let max_v = *range.upper_bound().unwrap();
                            let x = self.rng.random_range(0.0..1.0);
                            let vel = self.rng.random_range(-0.1..0.1);
                            self.position.insert(key.clone(), x);
                            self.velocity.insert(key.clone(), vel);
                            self.variable_bounds.insert(
                                key.clone(),
                                (VariableValue::Int(min_v), VariableValue::Int(max_v)),
                            );
                        }
                        _ => panic!("Non-integer domain for int variable `{}`", identifier),
                    };
                }
                Type::Bool => {
                    let x = self.rng.random_range(0.0..1.0);
                    let vel = self.rng.random_range(-0.1..0.1);
                    self.position.insert(key.clone(), x);
                    self.velocity.insert(key.clone(), vel);
                    self.variable_bounds
                        .insert(key, (VariableValue::Bool(false), VariableValue::Bool(true)));
                }
                Type::Float => {
                    let domain = variable
                        .domain
                        .as_ref()
                        .unwrap_or_else(|| panic!("No domain for variable `{}`", identifier));
                    match domain {
                        Domain::Float(range) => {
                            let min_v = *range.lower_bound().unwrap();
                            let max_v = *range.upper_bound().unwrap();
                            let x = self.rng.random_range(0.0..1.0);
                            let vel = self.rng.random_range(-0.1..0.1);
                            self.position.insert(key.clone(), x);
                            self.velocity.insert(key.clone(), vel);
                            self.variable_bounds.insert(
                                key,
                                (VariableValue::Float(min_v), VariableValue::Float(max_v)),
                            );
                        }
                        _ => panic!("Non-floating domain for float variable `{}`", identifier),
                    };
                }
                _ => continue,
            }
        }

        self.normalizer = Normalizer::new(self.variable_bounds.clone());
        let denormalized_position = self.normalizer.denormalize(&self.position);
        let repaired_position = self.repair_solution(&denormalized_position);
        self.local_best_position = self.position.clone();
        self.local_best_obj = self.compute_obj(&repaired_position).into();
        self.local_best_violation = self.evaluate_current_violations(&repaired_position).1;
    }

    pub fn update_velocity_and_position(
        &mut self,
        global_best_denormalized: &HashMap<String, VariableValue>,
        w: f64,
        c1: f64,
        c2: f64,
    ) {
        let global_best = self.normalizer.normalize(global_best_denormalized);
        let mut new_position: HashMap<String, f64> = HashMap::with_capacity(self.position.len());
        
        for (var_name, var_value) in &self.position {
            let vel = self.velocity.get(var_name).unwrap_or(&0.0);
            let local_best_val = self.local_best_position.get(var_name).unwrap_or(var_value);
            let global_best_val = global_best.get(var_name).unwrap_or(var_value);

            let r1: f64 = self.rng.random_range(0.0..1.0);
            let r2: f64 = self.rng.random_range(0.0..1.0);

            let new_vel = w * vel + c1 * r1 * (local_best_val - var_value) + c2 * r2 * (global_best_val - var_value);
            self.velocity.insert(var_name.clone(), new_vel);

            let new_pos = (var_value + new_vel).clamp(0.0, 1.0);
            new_position.insert(var_name.clone(), new_pos);
        }

        self.position = new_position;
        let denormalized_position = self.normalizer.denormalize(&self.position);
        let repaired_position = self.repair_solution(&denormalized_position);
        let candidate_obj = self.compute_obj(&repaired_position).into();
        let candidate_violation = self.evaluate_current_violations(&repaired_position).1;
        if is_better_candidate(candidate_obj, candidate_violation, self.local_best_obj, self.local_best_violation){
            self.local_best_position = self.position.clone();
            self.local_best_obj = candidate_obj;
            self.local_best_violation = candidate_violation;
        }
  
    }

    fn repair_solution(&self, denormalized_position: &HashMap<String, VariableValue>) -> HashMap<String, VariableValue> {
        let mut repaired_solution = HashMap::with_capacity(denormalized_position.len());
        let (x_values, y_values, t_values) = self.extract_variables_value(denormalized_position);

        for idx in 0..t_values.len(){
            if t_values[idx] == 0 {
                repaired_solution.insert(format!("X_INTRODUCED_{}_", idx + 16), VariableValue::Float(2500.0));
                repaired_solution.insert(format!("X_INTRODUCED_{}_", idx + 26), VariableValue::Float(1500.0));
            } else if x_values[idx] > 1200.0{
                repaired_solution.insert(format!("X_INTRODUCED_{}_", idx + 16), VariableValue::Float(1200.0));
                
            } else if y_values[idx] > 700.0{
                repaired_solution.insert(format!("X_INTRODUCED_{}_", idx + 26), VariableValue::Float(700.0));
            }else {
                repaired_solution.insert(format!("X_INTRODUCED_{}_", idx + 16), VariableValue::Float(x_values[idx]));
                repaired_solution.insert(format!("X_INTRODUCED_{}_", idx + 26), VariableValue::Float(y_values[idx]));
                repaired_solution.insert(format!("X_INTRODUCED_{}_", idx + 36), VariableValue::Int(t_values[idx]));
            }
             
        }
        repaired_solution

        }
    

    fn compute_obj(&self, denormalized_position: &HashMap<String, VariableValue>) -> f64{
        let (x_values, y_values, t_values) = self.extract_variables_value(denormalized_position);

        let mut manhattan_distance_sum = 0.0;
        let mut area_sum = 0.0;

        for idx in 0..t_values.len(){
            let t1 = t_values[idx];
            if t1 != 0 && idx < x_values.len() && idx < y_values.len(){
                for (idx2) in (idx + 1)..t_values.len(){
                    let t2 = t_values[idx2];
                    if t2 != 0 && idx2 < x_values.len() && idx2 < y_values.len(){
                        if(x_values[idx] != 2500.0 && x_values[idx + 1] != 2500.0 && y_values[idx] != 1500.0 && y_values[idx + 1] != 1500.0){
                            let x1_center = x_values[idx] + WFIX[(t1-1) as usize] / 2.0;
                            let y1_center = y_values[idx] + HFIX[(t1-1) as usize] / 2.0;
                            let x2_center = x_values[idx2] + WFIX[(t2-1) as usize] / 2.0;
                            let y2_center = y_values[idx2] + HFIX[(t2-1) as usize] / 2.0;
                            manhattan_distance_sum += (x1_center - x2_center).abs() + (y1_center - y2_center).abs();
                        }
                    }
                }
        
            }
        }

        for (idx, &t) in t_values.iter().enumerate(){
            if t != 0{
                if(x_values[idx] != 2500.0 && y_values[idx] != 1500.0){
                    area_sum += WFIX[(t-1) as usize] * HFIX[(t-1) as usize];
                }
            }
        }
        -(manhattan_distance_sum as f64 + area_sum as f64)
    }

    pub(crate) fn evaluate_current_violations(&mut self, denormalized_position: &HashMap<String, VariableValue>) -> (Option<f64>, f64) {
        for (var_name, var_value) in denormalized_position {
            match var_value {
                VariableValue::Bool(val) => {
                    self.solution_provider
                        .provide_bool(var_name.to_string(), *val);
                }
                VariableValue::Float(val) => {
                    self.solution_provider.provide_float(var_name.to_string(), *val);
                }
                VariableValue::Int(val) => {
                    self.solution_provider
                        .provide_int(var_name.to_string(), *val);
                }
                VariableValue::Set(val) => {
                    self.solution_provider
                        .provide_set(var_name.to_string(), val.clone());
                }
            }
        }
        let result = self
            .invariant_evaluator
            .evaluate_invariants_graph(&self.solution_provider);

        result
    }

    pub fn local_best_position(&self) -> HashMap<String, VariableValue> {
        self.normalizer.denormalize(&self.local_best_position)
    }

    pub fn local_best_obj(&self) -> Option<f64> {
        self.local_best_obj
    }

    pub fn local_best_violation(&self) -> f64 {
        self.local_best_violation
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    fn extract_variables_value(&self, denormalized_position: &HashMap<String, VariableValue>) -> (Vec<f64>, Vec<f64>, Vec<i64>) {
        let x_vars: Vec<&str> = vec!["X_INTRODUCED_16_", "X_INTRODUCED_17_", "X_INTRODUCED_18_", "X_INTRODUCED_19_", "X_INTRODUCED_20_", "X_INTRODUCED_21_", "X_INTRODUCED_22_", "X_INTRODUCED_23_", "X_INTRODUCED_24_", "X_INTRODUCED_25_"];

        let x_values: Vec<f64> = x_vars.iter().filter_map(|v| {
            match denormalized_position.get(*v) {
                Some(VariableValue::Float(val)) => Some(*val),
                _ => None,
            }
        }).collect();

        let y_vars: Vec<&str> = vec!["X_INTRODUCED_26_", "X_INTRODUCED_27_", "X_INTRODUCED_28_", "X_INTRODUCED_29_", "X_INTRODUCED_30_", "X_INTRODUCED_31_", "X_INTRODUCED_32_", "X_INTRODUCED_33_", "X_INTRODUCED_34_", "X_INTRODUCED_35_"];
        let y_values: Vec<f64> = y_vars.iter().filter_map(|v| {
            match denormalized_position.get(*v) {
                Some(VariableValue::Float(val)) => Some(*val),
                _ => None,
            }
        }).collect();

        let t_vars: Vec<&str> = vec!["X_INTRODUCED_36_", "X_INTRODUCED_37_", "X_INTRODUCED_38_", "X_INTRODUCED_39_", "X_INTRODUCED_40_", "X_INTRODUCED_41_", "X_INTRODUCED_42_", "X_INTRODUCED_43_", "X_INTRODUCED_44_", "X_INTRODUCED_45_"];
        let t_values: Vec<i64> = t_vars.iter().filter_map(|v| {
            match denormalized_position.get(*v) {
                Some(VariableValue::Int(val)) => Some(*val),
                _ => None,
            }
        }).collect();
        (x_values, y_values, t_values)
    }

}

#[derive(Clone)]
pub struct PSO {
    seed: i64,
    max_iteration: i64,
    w: f64,
    c1: f64,
    c2: f64,
    available_solutions: i64,
    swarm: Vec<PSOParticle>,
    global_best_position: HashMap<String, VariableValue>,
    global_best_obj: Option<f64>,
    global_best_violation: f64,
    fzn_path: PathBuf,
    ozn_path: PathBuf,
}

impl std::fmt::Debug for PSO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MixedPSO")
            .field("max_iteration", &self.max_iteration)
            .field("w", &self.w)
            .field("c1", &self.c1)
            .field("c2", &self.c2)
            .field("global_best_position", &self.global_best_position)
            .field("global_best_obj", &self.global_best_obj)
            .field("global_best_violation", &self.global_best_violation)
            .field("fzn_path", &self.fzn_path)
            .field("ozn_path", &self.ozn_path)
            .finish()
    }
}

impl PSO {
    pub fn new(
        seed: i64,
        swarm_size: i64,
        max_iteration: i64,
        w: f64,
        c1: f64,
        c2: f64,
        available_solutions: i64,
        fzn_path: PathBuf,
        ozn_path: PathBuf,
    ) -> Self {
        
        Self {
            seed,
            max_iteration,
            w,
            c1,
            c2,
            available_solutions,
            swarm: vec![PSOParticle::new(); swarm_size as usize],
            global_best_position: HashMap::new(),
            global_best_obj: None,
            global_best_violation: std::f64::INFINITY,
            fzn_path,
            ozn_path,
        }
    }

    pub fn search(&mut self) {
        //Initialization
        let half_swarm = self.swarm.len() / 2;
        for (id, particle) in self.swarm.iter_mut().enumerate() {
            particle.initialize(self.seed, id as i64, self.fzn_path.clone(), self.ozn_path.clone());

            if id < half_swarm {
                let solution_number = rand::rng().random_range(1..=self.available_solutions);
                println!("solution number: {}", solution_number);
                particle.set_initial_position_and_random_velocity(&PathBuf::from(format!("initial_solutions/solution_{}.json", solution_number)));
            } else {
                 particle.set_initial_position_and_random_velocity(&PathBuf::from(format!("initial_solutions/dummy.json")));
            }

            if is_better_candidate(
                particle.local_best_obj(),
                particle.local_best_violation(),
                self.global_best_obj,
                self.global_best_violation,
            ) {
                self.global_best_position = particle.local_best_position().clone();
                self.global_best_obj = particle.local_best_obj();
                self.global_best_violation = particle.local_best_violation();
            }
        }

        //search
        for iter in 0..self.max_iteration{
            for particle in &mut self.swarm {
                particle.update_velocity_and_position(
                    &self.global_best_position,
                    self.w,
                    self.c1,
                    self.c2,
                );
            }

            for particle in &self.swarm {
                if is_better_candidate(particle.local_best_obj(), particle.local_best_violation(), self.global_best_obj, self.global_best_violation){
                    self.global_best_position = particle.local_best_position();
                    self.global_best_obj = particle.local_best_obj();
                    self.global_best_violation = particle.local_best_violation();
                }
            }

            println!("Iteration {}: Best objective: {:?}, Best violation: {:?}", iter, self.global_best_obj, self.global_best_violation);
        }

        println!("Final best solution:\n {:?}\nObjective: {:?}\nViolation: {:?}", self.global_best_position, self.global_best_obj, self.global_best_violation);
    }
}
use crate::evaluator::evaluator::Evaluator;
use crate::solution_provider::{SolutionProvider, VariableValue};
use flatzinc_serde::{Domain, FlatZinc, Identifier, Type, Variable};
use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde_json::from_str;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use crate::data_utility::minizinc_solution_normalizer::MiniZincSolutionNormalizer;

const DEFAULT_SEED: u64 = 42;
const FEASIBILITY_TOL: f64 = 1e-3;

#[derive(Clone)]
pub struct FlatzincBasedParticle {
    id: i64,
    normalizer: MiniZincSolutionNormalizer,
    variables: HashMap<String, Variable>,
    variable_index: HashMap<String, usize>,
    position: Vec<f64>,
    velocity: Vec<f64>,
    variable_bounds: HashMap<String, (VariableValue, VariableValue)>,
    local_best_position: HashMap<String, f64>,
    local_best_obj: Option<f64>,
    local_best_violation: f64,
    solution_provider: SolutionProvider,
    invariant_evaluator: Evaluator,
    fzn_path: PathBuf,
    ozn_path: PathBuf,
    rng: ChaCha20Rng,
    stagnation_counter: i64,
}

#[derive(Clone)]
pub struct FlatzincBasedPSO {
    seed: i64,
    max_iteration: i64,
    w: f64,
    c1: f64,
    c2: f64,
    swarm: Vec<FlatzincBasedParticle>,
    global_best_position: HashMap<String, VariableValue>,
    global_best_obj: Option<f64>,
    global_best_violation: f64,
    fzn_path: PathBuf,
    ozn_path: PathBuf,
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

impl Default for FlatzincBasedParticle {
    fn default() -> Self {
        Self {
            id: 0,
            normalizer: MiniZincSolutionNormalizer::default(),
            variables: HashMap::new(),
            variable_index: HashMap::new(),
            position: Vec::new(),
            velocity: Vec::new(),
            variable_bounds: HashMap::new(),
            local_best_position: HashMap::new(),
            local_best_obj: None,
            local_best_violation: std::f64::INFINITY,
            solution_provider: Default::default(),
            invariant_evaluator: Default::default(),
            fzn_path: Default::default(),
            ozn_path: Default::default(),
            rng: ChaCha20Rng::seed_from_u64(DEFAULT_SEED),
            stagnation_counter: 0,
        }
    }
}

impl FlatzincBasedParticle {
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
        self.invariant_evaluator = Evaluator::new(&*self.fzn_path, fzn.clone(), None);

        // collect, sort by trailing numeric suffix, and build maps
        let mut vars: Vec<(String, Variable)> = fzn
            .variables
            .iter()
            .filter(|(_, var)| !var.defined)
            .map(|(id, var)| (id.clone(), var.clone()))
            .collect();

        // sort by numeric suffix (uses var_index defined on impl)
        vars.sort_by_key(|(id, _)| Self::var_index(id).unwrap_or(usize::MAX));

        self.variables.clear();
        self.variable_index.clear();
        for (idx, (name, var)) in vars.into_iter().enumerate() {
            self.variables.insert(name.clone(), var);
            self.variable_index.insert(name, idx + 1); // 1-based index
        }
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
                            self.variable_bounds.insert(
                                key.clone(),
                                (VariableValue::Int(min_v), VariableValue::Int(max_v)),
                            );
                        }
                        _ => panic!("Non-integer domain for int variable `{}`", identifier),
                    };
                }
                Type::Bool => {
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

        let mut id = 1;
        for _ in &self.variable_bounds {
            let x = self.rng.random_range(0.0..1.0);
            let vel = self.rng.random_range(-0.1..0.1);
            self.position.push(x);
            self.velocity.push(vel);
            id += 1;
        }

        self.normalizer = MiniZincSolutionNormalizer::new(self.variable_bounds.clone());
        let mapped_position = self.position_to_named_map(&self.position);
        let denormalized_position = self.normalizer.denormalize(&mapped_position);
        self.local_best_position = mapped_position;
        (self.local_best_obj, self.local_best_violation) =
            self.evaluate_current_violations(&denormalized_position);
    }

    pub fn update_velocity_and_position(
        &mut self,
        global_best_denormalized: &HashMap<String, VariableValue>,
        w: f64,
        c1: f64,
        c2: f64,
    ) {
        let global_best = self.normalizer.normalize(global_best_denormalized);
        let global_best_vec = self.named_map_to_position(&global_best);
        let local_best_vec = self.named_map_to_position(&self.local_best_position);
        let mut new_position: Vec<f64> = Vec::with_capacity(self.position.len());

        for (idx, vel) in self.velocity.iter_mut().enumerate() {
            let var_value = *self.position.get(idx).unwrap_or(&0.0);
            let local_best_val = *local_best_vec.get(idx).unwrap_or(&var_value);
            let global_best_val = *global_best_vec.get(idx).unwrap_or(&var_value);

            let r1: f64 = self.rng.random_range(0.0..1.0);
            let r2: f64 = self.rng.random_range(0.0..1.0);

            let new_vel = w * *vel
                + c1 * r1 * (local_best_val - var_value)
                + c2 * r2 * (global_best_val - var_value);
            *vel = new_vel;

            let new_pos = (var_value + new_vel).clamp(0.0, 1.0);
            new_position.push(new_pos);

            let crazy_factor = self.rng.random_range(0.0..1.0);
            if crazy_factor < 0.5 {
                let noise = self.rng.random_range(-0.2..0.2);
                let crazy_vel = (new_vel + noise).clamp(0.0, 0.1);
                *vel = crazy_vel;
            }
        }

        self.position = new_position;
        let mapped_position = self.position_to_named_map(&self.position);
        let denormalized_position = self.normalizer.denormalize(&mapped_position);
        let (candidate_obj, candidate_violation) =
            self.evaluate_current_violations(&denormalized_position);

        if self.stagnation_counter == 5 {
            self.random_initialize_position_and_velocity();
            self.stagnation_counter = 0;
            return;
        }

        if is_better_candidate(
            candidate_obj,
            candidate_violation,
            self.local_best_obj,
            self.local_best_violation,
        ) {
            self.local_best_position = mapped_position;
            self.local_best_obj = candidate_obj;
            self.local_best_violation = candidate_violation;
        } else {
            self.stagnation_counter += 1;
        }
    }

    pub(crate) fn evaluate_current_violations(
        &mut self,
        denormalized_position: &HashMap<String, VariableValue>,
    ) -> (Option<f64>, f64) {
        for (var_name, var_value) in denormalized_position {
            match var_value {
                VariableValue::Bool(val) => {
                    self.solution_provider
                        .provide_bool(var_name.to_string(), *val);
                }
                VariableValue::Float(val) => {
                    self.solution_provider
                        .provide_float(var_name.to_string(), *val);
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

    fn var_index(name: &str) -> Option<usize> {
        // iterate from the end: first skip trailing non-digits (e.g. trailing underscores),
        // then collect the contiguous digit run immediately before them.
        let mut digits_rev = String::new();
        let mut iter = name.chars().rev();

        // find first digit from the end (skip trailing non-digits)
        while let Some(ch) = iter.next() {
            if ch.is_ascii_digit() {
                digits_rev.push(ch);
                break;
            }
        }

        // if no digit found at all
        if digits_rev.is_empty() {
            return None;
        }

        // collect remaining contiguous digits (still in reverse order)
        for ch in iter {
            if ch.is_ascii_digit() {
                digits_rev.push(ch);
            } else {
                break;
            }
        }

        let digits: String = digits_rev.chars().rev().collect();
        digits.parse::<usize>().ok()
    }

    pub fn position_to_named_map(&self, position: &[f64]) -> HashMap<String, f64> {
        let mut map = HashMap::with_capacity(self.variable_index.len());
        for (name, &one_based_idx) in &self.variable_index {
            // convert to 0-based safely
            let pos_idx = one_based_idx.saturating_sub(1);
            let val = position.get(pos_idx).copied().unwrap_or(0.0);
            map.insert(name.clone(), val);
        }
        map
    }

    pub fn named_map_to_position(&self, global_best: &HashMap<String, f64>) -> Vec<f64> {
        let max_idx = match self.variable_index.values().copied().max() {
            Some(m) => m,
            None => return Vec::new(),
        };
        let mut pos = vec![0.0_f64; max_idx];
        for (name, &val) in global_best {
            if let Some(&one_based) = self.variable_index.get(name) {
                if one_based >= 1 && one_based <= max_idx {
                    pos[one_based - 1] = val;
                }
            }
        }
        pos
    }
}

impl FlatzincBasedPSO {
    pub fn new(
        seed: i64,
        swarm_size: i64,
        max_iteration: i64,
        w: f64,
        c1: f64,
        c2: f64,
        fzn_path: PathBuf,
        ozn_path: PathBuf,
    ) -> Self {
        Self {
            seed,
            max_iteration,
            w,
            c1,
            c2,
            swarm: vec![FlatzincBasedParticle::new(); swarm_size as usize],
            global_best_position: HashMap::new(),
            global_best_obj: None,
            global_best_violation: std::f64::INFINITY,
            fzn_path,
            ozn_path,
        }
    }

    pub fn search(&mut self) -> (Option<f64>, f64) {
        //Initialization
        for (id, particle) in self.swarm.iter_mut().enumerate() {
            particle.initialize(
                self.seed,
                id as i64,
                self.fzn_path.clone(),
                self.ozn_path.clone(),
            );

            particle.random_initialize_position_and_velocity();

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
        println!(
            "Initial best solution:\n {:?}",
            self.global_best_position
        );

        //Search
        for iter in 0..self.max_iteration {
            for particle in &mut self.swarm {
                particle.update_velocity_and_position(
                    &self.global_best_position,
                    self.w,
                    self.c1,
                    self.c2,
                );
            }

            for particle in &self.swarm {
                if is_better_candidate(
                    particle.local_best_obj(),
                    particle.local_best_violation(),
                    self.global_best_obj,
                    self.global_best_violation,
                ) {
                    self.global_best_position = particle.local_best_position();
                    self.global_best_obj = particle.local_best_obj();
                    self.global_best_violation = particle.local_best_violation();
                }
            }

            println!(
                "Iteration {}: Best objective: {:?}, Best violation: {:?}",
                iter, self.global_best_obj, self.global_best_violation
            );
        }

        println!(
            "Final best solution:\n {:?}\nObjective: {:?}\nViolation: {:?}",
            self.global_best_position, self.global_best_obj, self.global_best_violation
        );

        (self.global_best_obj, self.global_best_violation)
    }
}

impl std::fmt::Debug for FlatzincBasedParticle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlatzincBasedParticle")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("velocity", &self.velocity)
            .field("local_best_position", &self.local_best_position)
            .field("local_best_obj", &self.local_best_obj)
            .field("local_best_violation", &self.local_best_violation)
            .field("variables_count", &self.variables.len())
            .field("variable_bounds_count", &self.variable_bounds.len())
            .field("stagnation_counter", &self.stagnation_counter)
            .field("fzn_path", &self.fzn_path)
            .field("ozn_path", &self.ozn_path)
            .field("normalizer", &"<MiniZincSolutionNormalizer>")
            .field("solution_provider", &"<SolutionProvider>")
            .field("invariant_evaluator", &"<Evaluator>")
            .field("rng", &"<ChaCha20Rng>")
            .finish()
    }
}

impl std::fmt::Debug for FlatzincBasedPSO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlatzincBasedPSO")
            .field("seed", &self.seed)
            .field("max_iteration", &self.max_iteration)
            .field("w", &self.w)
            .field("c1", &self.c1)
            .field("c2", &self.c2)
            .field("swarm", &self.swarm)
            .field("global_best_position", &self.global_best_position)
            .field("global_best_obj", &self.global_best_obj)
            .field("global_best_violation", &self.global_best_violation)
            .field("fzn_path", &self.fzn_path)
            .field("ozn_path", &self.ozn_path)
            .finish()
    }
}

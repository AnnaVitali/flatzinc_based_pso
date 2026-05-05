use crate::data_utility::solution_normalizer::SolutionNormalizer;
use crate::data_utility::types::Register;
use crate::data_utility::types::VariableValue;
use crate::evaluator::mini_evaluator::MiniEvaluator;
use crate::heuristics::pso_utility::is_better_candidate;
use crate::solution_provider::SolutionProvider;
use flatzinc_serde::{Domain, FlatZinc, Type, Variable};
use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde_json::from_str;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

const DEFAULT_SEED: u64 = 42;

#[derive(Clone)]
/// A struct representing a particle in the Particle Swarm Optimization (PSO) algorithm, specifically designed for solving FlatZinc problems.
/// Each particle maintains its own position, velocity, and best known solution, and interacts with a global best solution to guide its search process.
pub struct FlatzincBasedParticle {
    /// A unique identifier for the particle, used for tracking and debugging purposes.
    id: i64,
    /// A normalizer for converting between the particle's position representation and the variable values in the FlatZinc problem, based on variable bounds.
    normalizer: SolutionNormalizer,
    /// A map of variable names to their corresponding `Variable` definitions from the FlatZinc model, used for understanding the problem structure and variable types.
    variables: BTreeMap<String, Variable>,
    variable_types: Vec<Type>,
    // A map of variable names to their corresponding indices in the particle's position vector, used for mapping between the particle's position and the FlatZinc variables.
    variables_in_position_index: BTreeMap<String, usize>,
    // A map of indices in the particle's position vector to their corresponding variable registers, used for providing solutions to the FlatZinc model based on the particle's position.
    index_to_register_map: BTreeMap<usize, Register>,
    // A map of variable registers to their corresponding variable names, used to rebuild name-based solutions from the register-indexed solution vector.
    register_to_name_map: BTreeMap<Register, String>,
    /// A vector representing the current solution of the particle in terms of variable values.
    solution: Vec<Option<VariableValue>>,
    /// A vector representing the current position of the particle in the normalized search space, where each value is typically in the range [0, 1].
    position: Vec<f64>,
    /// A vector representing the current velocity of the particle in the normalized search space, which influences how the particle moves towards its local and global best positions.
    velocity: Vec<f64>,
    /// A vector of tuples representing the (min, max) bounds for each variable in the FlatZinc problem, used for normalizing and denormalizing the particle's position values.
    variable_bounds: Vec<(f64, f64)>,
    /// A vector representing the best known position of the particle in terms of variable values, which is updated whenever the particle finds a better solution.
    local_best_position: Vec<f64>,
    /// The objective value of the best known solution for the particle, used for comparing against other solutions and guiding the search process.
    local_best_obj: Option<f64>,
    /// The total violation of constraints for the best known solution of the particle, used for determining feasibility and guiding the search process.
    local_best_violation: f64,
    /// A provider for generating solutions based on the current variable assignments, used for evaluating the particle's position against the FlatZinc model.
    solution_provider: SolutionProvider,
    /// An evaluator for assessing the invariants and constraints of the FlatZinc model based on the current solution, used for calculating objective values and constraint violations.
    invariant_evaluator: MiniEvaluator,
    /// The file path to the FlatZinc model, used for initializing the particle and providing context for the solution provider and invariant evaluator.
    fzn_path: PathBuf,
    /// A random number generator for the particle, used for introducing stochasticity in the velocity and position updates, as well as for random initialization.
    rng: ChaCha20Rng,
    /// A counter for tracking how many consecutive iterations the particle has gone without improving its local best solution, used for implementing a stagnation-based reinitialization strategy.
    stagnation_counter: i64,
}

#[derive(Clone)]
/// A struct representing the Particle Swarm Optimization (PSO) algorithm, specifically designed for solving FlatZinc problems.
/// The PSO algorithm maintains a swarm of particles, each of which explores the search space to find optimal solutions to
/// the FlatZinc problem, guided by both their local best solutions and a global best solution found by the swarm.
pub struct FlatzincBasedPSO {
    /// A seed value for initializing the random number generators of the particles, ensuring reproducibility of the search process.
    seed: i64,
    /// The maximum number of iterations for the PSO algorithm, determining how long the search process will run before termination.
    max_iteration: i64,
    /// The inertia weight for the velocity update, controlling how much the particle's previous velocity influences its current movement.
    w: f64,
    /// The cognitive coefficient for the velocity update, controlling how much the particle's local best position influences its current movement.
    c1: f64,
    /// The social coefficient for the velocity update, controlling how much the global best position influences the particle's current movement.
    c2: f64,
    /// A vector of particles that make up the swarm, each of which explores the search space and maintains its own local best solution.
    swarm: Vec<FlatzincBasedParticle>,
    /// A map representing the best known position found by any particle in the swarm, which is used to guide the search process of all particles.
    global_best_position: Vec<f64>,
    /// The objective value of the global best solution found by the swarm, used for comparing against other solutions and guiding the search process.
    global_best_obj: Option<f64>,
    /// The total violation of constraints for the global best solution found by the swarm, used for determining feasibility and guiding the search process.
    global_best_violation: f64,
    /// The file path to the FlatZinc model, used for initializing the particles and providing context for the solution providers and invariant evaluators.
    fzn_path: PathBuf,
    /// The identifier of the particle that currently holds the best solution found by the swarm.
    best_particle_id: Option<i64>,
}

/// Implements the `Default` trait for `FlatzincBasedParticle`, providing a default constructor that initializes all fields with default values.
impl Default for FlatzincBasedParticle {
    fn default() -> Self {
        Self {
            id: 0,
            normalizer: SolutionNormalizer::default(),
            variables: BTreeMap::new(),
            variable_types: Vec::new(),
            variables_in_position_index: BTreeMap::new(),
            index_to_register_map: BTreeMap::new(),
            solution: Vec::new(),
            position: Vec::new(),
            velocity: Vec::new(),
            variable_bounds: Vec::new(),
            local_best_position: Vec::new(),
            local_best_obj: None,
            local_best_violation: std::f64::INFINITY,
            solution_provider: Default::default(),
            invariant_evaluator: Default::default(),
            fzn_path: Default::default(),
            rng: ChaCha20Rng::seed_from_u64(DEFAULT_SEED),
            stagnation_counter: 0,
            register_to_name_map: BTreeMap::new(),
        }
    }
}

/// Implements the `Default` trait for `FlatzincBasedPSO`, providing a default constructor that initializes all fields with default values.
impl FlatzincBasedParticle {
    pub fn new() -> Self {
        Default::default()
    }

    /// Initializes the particle with the given seed, identifier, and file paths for the FlatZinc model and output solution.
    /// This method sets up the random number generator, loads the FlatZinc model, initializes the solution provider and invariant evaluator,
    /// and prepares the variable mappings and bounds for the particle's search process.
    ///
    /// # Arguments
    /// * `seed` - A seed value for initializing the random number generator, ensuring reproducibility of the particle's behavior.
    /// * `id` - A unique identifier for the particle, used for tracking and debugging purposes.
    /// * `fzn_path` - The file path to the FlatZinc model, used for loading the problem definition and initializing the solution provider and invariant evaluator.
    pub fn initialize(&mut self, seed: i64, id: i64, fzn_path: PathBuf) {
        self.id = id;
        self.rng = ChaCha20Rng::seed_from_u64(seed as u64 + id as u64);
        self.fzn_path = fzn_path;
        let mut s = String::new();
        let file = File::open(&self.fzn_path).expect("Failed to open fzn file");
        BufReader::new(file)
            .read_to_string(&mut s)
            .expect("Failed to read fzn file");
        let s = s.strip_prefix("\u{feff}").unwrap_or(&s);
        let fzn: FlatZinc = from_str(s).expect("Failed to parse flatzinc json");
        self.solution_provider = SolutionProvider::new(fzn.clone());
        self.solution = vec![None; fzn.variables.len()];
        self.invariant_evaluator = MiniEvaluator::new(&*self.fzn_path, fzn.clone(), None);
        let mut vars: Vec<(String, Variable)> = fzn
            .variables
            .iter()
            .filter(|(_, var)| !var.defined)
            .map(|(id, var)| (id.clone(), var.clone()))
            .collect();

        vars.sort_by_key(|(id, _)| Self::var_index(id).unwrap_or(usize::MAX));

        let variable_registers_map = self.solution_provider.get_vars_register_map();
        for (idx, (name, var)) in vars.into_iter().enumerate() {
            if !var.defined {
                self.variables.insert(name.clone(), var);
                self.variables_in_position_index.insert(name.clone(), idx); // 1-based index
                let register = variable_registers_map
                    .get(&name)
                    .copied()
                    .expect("Register not found for variable");
                self.index_to_register_map.insert(idx, register);
                self.register_to_name_map.insert(register, name);
            }
        }

        self.variable_bounds = vec![(0.0, 0.0); self.variables.len()];
        self.variable_types = vec![Type::Float; self.variables.len()];
        for (var, id) in &self.variables_in_position_index {
            let variable = self
                .variables
                .get(var)
                .expect("Variable not found in variables map");
            match variable.ty {
                Type::Int => {
                    let domain = variable
                        .domain
                        .as_ref()
                        .unwrap_or_else(|| panic!("No domain for variable `{}`", var));
                    match domain {
                        Domain::Int(range) => {
                            let min_v = *range.lower_bound().unwrap();
                            let max_v = *range.upper_bound().unwrap();
                            self.variable_bounds[*id] = (min_v as f64, max_v as f64);
                        }
                        _ => panic!("Non-integer domain for int variable `{}`", var),
                    };
                }
                Type::Bool => {
                    self.variable_bounds[*id] = (0.0, 1.0);
                }
                Type::Float => {
                    let domain = variable
                        .domain
                        .as_ref()
                        .unwrap_or_else(|| panic!("No domain for variable `{}`", var));
                    match domain {
                        Domain::Float(range) => {
                            let min_v = *range.lower_bound().unwrap();
                            let max_v = *range.upper_bound().unwrap();
                            self.variable_bounds[*id] = (min_v, max_v);
                        }
                        _ => panic!("Non-floating domain for float variable `{}`", var),
                    };
                }
                _ => continue,
            }

            self.variable_types[*id] = variable.ty.clone();
        }

        self.normalizer = SolutionNormalizer::new(self.variable_bounds.clone());
    }

    /// Randomly initializes the particle's position and velocity based on the variable bounds defined in the FlatZinc model.
    /// This method populates the `variable_bounds` map with the min and max values for each variable, then initializes the `position` and `velocity`
    /// vectors with random values in the normalized space [0, 1] for position and a small range for velocity.
    pub fn random_initialize_position_and_velocity(&mut self) {
        self.position.clear();
        self.velocity.clear();

        for _ in &self.variable_bounds {
            let x = self.rng.random_range(0.0..1.0);
            let vel = self.rng.random_range(-0.1..0.1);
            self.position.push(x);
            self.velocity.push(vel);
        }

        let denormalized_position = self.normalizer.denormalize(&self.position);
        self.local_best_position = self.position.clone();
        (self.local_best_obj, self.local_best_violation) =
            self.evaluate_current_violations(&denormalized_position);
    }

    /// Updates the particle's velocity and position based on its current velocity, the distance to its local best position, and the distance to the global best position.
    ///
    /// # Arguments
    /// * `global_best_denormalized` - A map representing the global best position found by the swarm, with variable names as keys and their corresponding values.
    /// * `w` - The inertia weight for the velocity update, controlling how much the particle's previous velocity influences its current movement.
    /// * `c1` - The cognitive coefficient for the velocity update, controlling how much the particle's local best position influences its current movement.
    /// * `c2` - The social coefficient for the velocity update, controlling how much the global best position influences the particle's current movement.
    pub fn update_velocity_and_position(
        &mut self,
        global_best_denormalized: &Vec<f64>,
        w: f64,
        c1: f64,
        c2: f64,
    ) {
        let global_best = self.normalizer.normalize(global_best_denormalized);
        let mut new_position: Vec<f64> = Vec::with_capacity(self.position.len());

        for (idx, vel) in self.velocity.iter_mut().enumerate() {
            let var_value = *self.position.get(idx).unwrap_or(&0.0);
            let local_best_val = *self.local_best_position.get(idx).unwrap_or(&var_value);
            let global_best_val = *global_best.get(idx).unwrap_or(&var_value);

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
        let denormalized_position = self.normalizer.denormalize(&self.position);
        let (candidate_obj, candidate_violation) =
            self.evaluate_current_violations(&denormalized_position);

        if self.stagnation_counter == 5 {
            self.restart_position_and_velocity();
            self.stagnation_counter = 0;
            return;
        }

        if is_better_candidate(
            candidate_obj,
            candidate_violation,
            self.local_best_obj,
            self.local_best_violation,
        ) {
            self.local_best_position = self.position.clone();
            self.local_best_obj = candidate_obj;
            self.local_best_violation = candidate_violation;
        } else {
            self.stagnation_counter += 1;
        }
    }

    /// Returns the position of the particle's local best solution.
    ///
    /// # Returns
    /// The HashMap containing the variable names and their corresponding values for the particle's local best position.
    pub fn local_best_position(&self) -> Vec<f64> {
        self.normalizer.denormalize(&self.local_best_position)
    }

    /// Converts a denormalized position vector into a solution map with variable names and their corresponding values, based on the variable types defined in the FlatZinc model.
    /// 
    /// # Arguments
    /// * `denormalized_position` - A vector of f64 values representing the denormalized position of the particle, where each value corresponds to a variable in the FlatZinc model.
    /// # Returns
    /// A `HashMap<String, VariableValue>` mapping variable names to their corresponding values, constructed from the denormalized position vector and the variable type information.
    pub fn get_denormalized_solution(
        &self,
        //denormalized_position: &Vec<f64>,
    ) -> HashMap<String, VariableValue> {
        let best_position_denormalized = self.normalizer.denormalize(&self.local_best_position);
        self.index_to_register_map
            .iter()
            .filter_map(|(index, register)| {
                let raw_value = best_position_denormalized.get(*index)?;
                let var_type = self.variable_types.get(*index)?;
                let name = self.register_to_name_map.get(register)?;

                let value = match var_type {
                    Type::Int => VariableValue::Int(raw_value.round() as i64),
                    Type::Bool => VariableValue::Bool(*raw_value >= 0.5),
                    Type::Float => VariableValue::Float(*raw_value),
                    _ => return None,
                };

                Some((name.clone(), value))
            })
            .collect()
    }

    /// Returns the objective value of the particle's local best solution, which may be `None` if the objective is not defined for that solution.
    /// # Returns
    /// The objective value of the particle's local best solution, or `None` if the objective is not defined.
    pub fn local_best_obj(&self) -> Option<f64> {
        self.local_best_obj
    }

    /// Returns the total violation of constraints for the particle's local best solution.
    /// # Returns
    /// The total violation of constraints for the particle's local best solution.
    pub fn local_best_violation(&self) -> f64 {
        self.local_best_violation
    }

    /// Returns the unique identifier of the particle.
    /// # Returns
    /// The unique identifier of the particle, which is used for tracking and debugging purposes.
    pub fn id(&self) -> i64 {
        self.id
    }

    fn evaluate_current_violations(
        &mut self,
        denormalized_position: &Vec<f64>,
    ) -> (Option<f64>, f64) {
        for (index, reg) in self.index_to_register_map.iter() {
            let var_value = denormalized_position
                .get(*index)
                .expect("Value for decision variable not found");
            let var_type = self.variable_types.get(*index).expect("Variable type not found");
            match var_type {
                Type::Int => {
                    self.solution[*reg as usize] =
                        Some(VariableValue::Int(var_value.round() as i64));
                }
                Type::Bool => {
                    self.solution[*reg as usize] =
                        Some(VariableValue::Bool(*var_value >= 0.5));
                }
                Type::Float => {
                    self.solution[*reg as usize] = Some(VariableValue::Float(*var_value));
                }
                _ => continue,
            }
        }
        
        self.solution_provider.provide_solution(&self.solution);

        let result = self
            .invariant_evaluator
            .evaluate_invariants_graph(&self.solution_provider);

        result
    }

    
    fn restart_position_and_velocity(&mut self) {
        self.position.clear();
        self.velocity.clear();

        for _ in &self.variable_bounds {
            let x = self.rng.random_range(0.0..1.0);
            let vel = self.rng.random_range(-0.1..0.1);
            self.position.push(x);
            self.velocity.push(vel);
        }
    }

    fn var_index(name: &str) -> Option<usize> {
        let mut digits_rev = String::new();
        let mut iter = name.chars().rev();

        while let Some(ch) = iter.next() {
            if ch.is_ascii_digit() {
                digits_rev.push(ch);
                break;
            }
        }

        if digits_rev.is_empty() {
            return None;
        }

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
}

/// Implements the `Debug` trait for `FlatzincBasedParticle`, providing a custom debug representation that includes key fields of the particle for easier debugging and visualization.
impl FlatzincBasedPSO {
    pub fn new(
        seed: i64,
        swarm_size: i64,
        max_iteration: i64,
        w: f64,
        c1: f64,
        c2: f64,
        fzn_path: PathBuf,
    ) -> Self {
        Self {
            seed,
            max_iteration,
            w,
            c1,
            c2,
            swarm: vec![FlatzincBasedParticle::new(); swarm_size as usize],
            global_best_position: Vec::new(),
            global_best_obj: None,
            global_best_violation: std::f64::INFINITY,
            fzn_path,
            best_particle_id: None,
        }
    }

    /// Executes the PSO search process, iteratively updating the particles' velocities and positions, and tracking the global best solution found by the swarm.
    /// # Returns
    /// A tuple containing the objective value of the global best solution (if defined) and the total violation of constraints for that solution, representing the quality of the best solution found by the PSO algorithm.
    pub fn search(&mut self) -> (Option<f64>, f64) {
        for (id, particle) in self.swarm.iter_mut().enumerate() {
            particle.initialize(self.seed, id as i64, self.fzn_path.clone());

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
                self.best_particle_id = Some(particle.id());
            }
        }
        println!("Initial best solution:\n {:?}", self.global_best_position);

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
                    self.best_particle_id = Some(particle.id());
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

        let final_solution = self.swarm
            .iter()
            .find(|p| p.id() == self.best_particle_id.unwrap())
            .expect("Best particle not found")
            .get_denormalized_solution();
        let final_solution_prittified = Self::preattify_solution_map(&final_solution);

        println!(
            "Final best solution MiniZinc:\n{}Objective: {:?}\nViolation: {:?}",
            final_solution_prittified, self.global_best_obj, self.global_best_violation
        );

        (self.global_best_obj, self.global_best_violation)
    }

    fn preattify_solution_map(solution: &HashMap<String, VariableValue>) -> String {
        let mut entries: Vec<_> = solution.iter().collect();
        entries.sort_by(|(left_name, _), (right_name, _)| left_name.cmp(right_name));

        let mut output = String::new();
        for (name, value) in entries {
            let rendered_value = match value {
                VariableValue::Int(value) => value.to_string(),
                VariableValue::Float(value) => value.to_string(),
                VariableValue::Bool(value) => value.to_string(),
                VariableValue::Set(values) => {
                    let mut ordered_values: Vec<_> = values.iter().copied().collect();
                    ordered_values.sort_unstable();
                    format!("{{{}}}", ordered_values.iter().map(i64::to_string).collect::<Vec<_>>().join(", "))
                }
            };

            let _ = writeln!(&mut output, "{} = {};", name, rendered_value);
        }

        output
    }
}

/// Implements the `Debug` trait for `FlatzincBasedPSO`, providing a custom debug representation that includes key fields of the PSO algorithm for easier debugging and visualization.
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
            .field("normalizer", &"<MiniZincSolutionNormalizer>")
            .field("solution_provider", &"<SolutionProvider>")
            .field("invariant_evaluator", &"<Evaluator>")
            .field("rng", &"<ChaCha20Rng>")
            .finish()
    }
}

/// Implements the `Debug` trait for `FlatzincBasedPSO`, providing a custom debug representation that includes key fields of the PSO algorithm for easier debugging and visualization.
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
            .finish()
    }
}

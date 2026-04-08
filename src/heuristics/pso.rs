use crate::data_utility::solution_normalizer::SolutionNormalizer;
use crate::heuristics::pso_utility::is_better_candidate;
use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::sync::Arc;

const DEFAULT_SEED: u64 = 42;

#[derive(Clone)]
/// A struct representing a particle in the Particle Swarm Optimization (PSO) algorithm, which maintains its position, velocity, local best solution, and other relevant information for optimization.
pub struct PSOParticle {
    /// The unique identifier for the particle, used for tracking and debugging purposes.
    id: i64,
    /// A `SolutionNormalizer` instance used to normalize and denormalize variable values for consistent evaluation across the search space.
    normalizer: SolutionNormalizer,
    /// The current position of the particle in the search space, represented as a vector of normalized variable values.
    position: Vec<f64>,
    /// The current velocity of the particle, which determines how it moves through the search space during optimization.
    velocity: Vec<f64>,
    /// The evaluation function used to assess the quality of solutions, which takes a vector of variable values and returns a tuple of objective value and constraint violation.
    evaluate_fn: Arc<dyn Fn(&[f64]) -> (f64, f64) + Send + Sync>,
    /// The bounds for each variable in the search space, represented as a vector of tuples where each tuple contains the lower and upper bounds for a variable.
    variable_bounds: Vec<(f64, f64)>,
    /// The best position found by the particle so far, which is updated whenever a better solution is found during the optimization process.
    local_best_position: Vec<f64>,
    /// The objective value of the best solution found by the particle, used for comparing against other solutions and guiding the search process.
    local_best_obj: f64,
    /// The total violation of constraints for the best solution found by the particle, used to determine the feasibility of solutions and guide the search process.
    local_best_violation: f64,
    /// A random number generator used for stochastic components of the PSO algorithm, such as random initialization and velocity updates, which is seeded for reproducibility.
    rng: ChaCha20Rng,
    /// A counter to track the number of iterations since the last improvement in the local best solution, used to trigger reinitialization of the particle if it gets stuck in a local optimum.
    stagnation_counter: i64,
}

#[derive(Clone)]
/// A struct representing the Particle Swarm Optimization (PSO) algorithm, which manages a swarm of particles and performs the optimization process to find the best solution for a given problem.
pub struct PSO {
    /// The seed for the random number generator, used to ensure reproducibility of the optimization process across different runs.
    seed: i64,
    /// The maximum number of iterations to perform during the optimization process, which determines how long the algorithm will run before terminating.
    max_iteration: i64,
    /// The inertia weight used in the velocity update formula, which controls the influence of the previous velocity on the current velocity and helps balance exploration and exploitation in the search process.
    w: f64,
    /// The cognitive coefficient used in the velocity update formula, which controls the influence of the particle's local best position on its movement and encourages particles to explore their own best solutions.
    c1: f64,
    /// The social coefficient used in the velocity update formula, which controls the influence of the global best position on the particle's movement and encourages particles to follow the best solution found by the swarm.
    c2: f64,
    /// The evaluation function used to assess the quality of solutions, which takes a vector of variable values and returns a tuple of objective value and constraint violation, guiding the optimization process towards better solutions.
    evaluation_function: Arc<dyn Fn(&[f64]) -> (f64, f64) + Send + Sync>,
    /// The bounds for each variable in the search space, represented as a vector of tuples where each tuple contains the lower and upper bounds for a variable, which is used to initialize particles and ensure they stay within feasible regions of the search space.
    variable_bounds: Vec<(f64, f64)>,
    /// The swarm of particles participating in the optimization process, where each particle represents a potential solution and explores the search space to find better solutions over iterations.
    swarm: Vec<PSOParticle>,
    /// The best position found by any particle in the swarm, which is updated whenever a better solution is found and serves as a guide for the particles to move towards during the optimization process.
    global_best_position: Vec<f64>,
    /// The objective value of the best solution found by the swarm, used for comparing against other solutions and determining the quality of the best solution found.
    global_best_obj: f64,
    /// The total violation of constraints for the best solution found by the swarm, used to determine the feasibility of the best solution and guide the search process towards feasible regions of the search space.
    global_best_violation: f64,
}

/// Implements the `Default` trait for `PSOParticle`, providing a default constructor that initializes all fields with default values, allowing for easy creation of particles without needing to specify all parameters upfront.
impl Default for PSOParticle {
    fn default() -> Self {
        Self {
            id: 0,
            normalizer: SolutionNormalizer::default(),
            position: Vec::new(),
            velocity: Vec::new(),
            variable_bounds: Vec::new(),
            local_best_position: Vec::new(),
            local_best_obj: std::f64::INFINITY,
            local_best_violation: std::f64::INFINITY,
            evaluate_fn: Arc::new(|_| (std::f64::INFINITY, std::f64::INFINITY)),
            rng: ChaCha20Rng::seed_from_u64(DEFAULT_SEED),
            stagnation_counter: 0,
        }
    }
}

/// Implements the `Default` trait for `PSO`, providing a default constructor that initializes all fields with default values, allowing for easy creation of a PSO instance without needing to specify all parameters upfront.
impl PSOParticle {
    pub fn new() -> Self {
        Default::default()
    }

    /// Initializes the particle with the given parameters, including setting up the random number generator with a seed based on the particle's ID, storing variable bounds, and preparing the evaluation function for assessing solutions during the optimization process.
    ///
    /// # Arguments
    /// * `seed` - The base seed for the random number generator, which is combined with the particle's ID to ensure unique randomness for each particle.
    /// * `id` - The unique identifier for the particle, used for tracking and seeding the random number generator.
    /// * `variable_bounds` - A vector of tuples representing the lower and upper bounds for each variable in the search space, which is used to initialize the particle's position and ensure it stays within feasible regions.
    /// * `evaluate_fn` - An evaluation function that takes a vector of variable values and returns a tuple of objective value and constraint violation, which is used to assess the quality of solutions found by the particle during the optimization process.  
    pub fn initialize(
        &mut self,
        seed: i64,
        id: i64,
        variable_bounds: Vec<(f64, f64)>,
        evaluate_fn: Arc<dyn Fn(&[f64]) -> (f64, f64) + Send + Sync>,
    ) {
        self.id = id;
        self.rng = ChaCha20Rng::seed_from_u64(seed as u64 + id as u64);
        self.variable_bounds = variable_bounds;
        self.evaluate_fn = evaluate_fn;
        self.normalizer = SolutionNormalizer::new(self.variable_bounds.clone());
    }

    /// Randomly initializes the particle's position and velocity within the normalized search space, ensuring that the initial position is feasible and evaluating the initial solution to set the local best position, objective value, and constraint violation for the particle.
    /// This method is called during the initialization of the particle and also when the particle gets stuck in a local optimum, allowing it to explore new regions of the search space and potentially find better solutions.
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
            (self.evaluate_fn)(&denormalized_position);
    }

    /// Updates the particle's velocity and position based on its current velocity, the distance to its local best position, and the distance to the global best position, following the standard PSO update rules. The method also includes a "crazy" factor that introduces random noise to the velocity to help escape local optima, and it evaluates the new position to update the local best solution if the new position is better.
    /// 
    /// # Arguments
    /// * `global_best_denormalized` - The global best position in the denormalized search space, which is used to calculate the social component of the velocity update.
    /// * `w` - The inertia weight that controls the influence of the previous velocity on the current velocity, helping to balance exploration and exploitation in the search process.
    /// * `c1` - The cognitive coefficient that controls the influence of the particle's local best position on its movement, encouraging particles to explore their own best solutions.
    /// * `c2` - The social coefficient that controls the influence of the global best position on the particle's movement, encouraging particles to follow the best solution found by the swarm.
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
        let (candidate_obj, candidate_violation) = (self.evaluate_fn)(&denormalized_position);

        if self.stagnation_counter == 5 {
            self.random_initialize_position_and_velocity();
            self.stagnation_counter = 0;
            return;
        }

        if is_better_candidate(
            Some(candidate_obj),
            candidate_violation,
            Some(self.local_best_obj),
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
    /// # Returns
    /// A vector of variable values representing the local best position found by the particle, denormalized to the original search space for accurate evaluation and comparison with other solutions.
    pub fn local_best_position(&self) -> Vec<f64> {
        self.normalizer.denormalize(&self.local_best_position)
    }

    /// Returns the objective value of the particle's local best solution.
    /// # Returns
    /// The objective value associated with the local best solution found by the particle, which is used
    pub fn local_best_obj(&self) -> f64 {
        self.local_best_obj
    }

    /// Returns the total violation of constraints for the particle's local best solution.
    /// # Returns
    /// The total violation of constraints associated with the local best solution found by the particle.
    pub fn local_best_violation(&self) -> f64 {
        self.local_best_violation
    }

    pub fn id(&self) -> i64 {
        self.id
    }
}

/// Implements the `Default` trait for `PSO`, providing a default constructor that initializes all fields with default values, allowing for easy creation of a PSO instance without needing to specify all parameters upfront.
impl PSO {
    pub fn new(
        seed: i64,
        swarm_size: i64,
        max_iteration: i64,
        w: f64,
        c1: f64,
        c2: f64,
        evaluation_function: Arc<dyn Fn(&[f64]) -> (f64, f64) + Send + Sync>,
        variable_bounds: Vec<(f64, f64)>,
    ) -> Self {
        Self {
            seed,
            max_iteration,
            w,
            c1,
            c2,
            evaluation_function,
            variable_bounds,
            swarm: vec![PSOParticle::new(); swarm_size as usize],
            global_best_position: Vec::new(),
            global_best_obj: std::f64::INFINITY,
            global_best_violation: std::f64::INFINITY,
        }
    }

    /// Executes the PSO search process, which involves initializing the swarm of particles, iteratively updating their velocities and positions based on their local best solutions and the global best solution, and evaluating their performance to find the best solution for the given problem. The method returns the objective value and constraint violation of the best solution found after the specified number of iterations.
    /// # Returns
    /// A tuple containing the objective value and constraint violation of the best solution found by the PSO algorithm after completing the search process, which can be used to assess the quality of the solution and its feasibility with respect to the problem constraints.
    pub fn search(&mut self) -> (f64, f64) {

        for (id, particle) in self.swarm.iter_mut().enumerate() {
            particle.initialize(
                self.seed,
                id as i64,
                self.variable_bounds.clone(),
                self.evaluation_function.clone(),
            );

            particle.random_initialize_position_and_velocity();

            if is_better_candidate(
                Some(particle.local_best_obj()),
                particle.local_best_violation(),
                Some(self.global_best_obj),
                self.global_best_violation,
            ) {
                self.global_best_position = particle.local_best_position().clone();
                self.global_best_obj = particle.local_best_obj();
                self.global_best_violation = particle.local_best_violation();
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
                    Some(particle.local_best_obj()),
                    particle.local_best_violation(),
                    Some(self.global_best_obj),
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

/// Implements the `Debug` trait for `PSOParticle`, providing a custom debug representation that includes key information about the particle's state, such as its ID, the lengths of its position and velocity vectors, the local best objective value and violation, and placeholders for the normalizer, evaluation function, and random number generator to avoid printing large or complex structures in the debug output.
impl std::fmt::Debug for PSOParticle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PSOParticle")
            .field("id", &self.id)
            .field("position_len", &self.position.len())
            .field("velocity_len", &self.velocity.len())
            .field("local_best_position_len", &self.local_best_position.len())
            .field("local_best_obj", &self.local_best_obj)
            .field("local_best_violation", &self.local_best_violation)
            .field("variable_bounds_count", &self.variable_bounds.len())
            .field("stagnation_counter", &self.stagnation_counter)
            .field("normalizer", &"<SolutionNormalizer>")
            .field("evaluate_fn", &"<Fn(&[f64]) -> (f64, f64)>")
            .field("rng", &"<ChaCha20Rng>")
            .finish()
    }
}

/// Implements the `Debug` trait for `PSO`, providing a custom debug representation that includes key information about the PSO instance, such as the seed, maximum iterations, coefficients, the size of the swarm, variable bounds, and placeholders for the evaluation function to avoid printing large or complex structures in the debug output.
impl std::fmt::Debug for PSO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PSO")
            .field("seed", &self.seed)
            .field("max_iteration", &self.max_iteration)
            .field("w", &self.w)
            .field("c1", &self.c1)
            .field("c2", &self.c2)
            .field("swarm_size", &self.swarm.len())
            .field("variable_bounds_count", &self.variable_bounds.len())
            .field("global_best_position_len", &self.global_best_position.len())
            .field("global_best_obj", &self.global_best_obj)
            .field("global_best_violation", &self.global_best_violation)
            .field("evaluation_function", &"<Fn(&[f64]) -> (f64, f64)>")
            .finish()
    }
}

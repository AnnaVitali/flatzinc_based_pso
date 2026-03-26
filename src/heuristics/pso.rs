use crate::data_utility::solution_normalizer::SolutionNormalizer;
use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::sync::Arc;

const DEFAULT_SEED: u64 = 42;
const FEASIBILITY_TOL: f64 = 1e-3;

#[derive(Clone)]
pub struct PSOParticle {
    id: i64,
    normalizer: SolutionNormalizer,
    position: Vec<f64>,
    velocity: Vec<f64>,
    evaluate_fn: Arc<dyn Fn(&[f64]) -> (f64, f64) + Send + Sync>,
    variable_bounds: Vec<(f64, f64)>,
    local_best_position: Vec<f64>,
    local_best_obj: f64,
    local_best_violation: f64,
    rng: ChaCha20Rng,
    stagnation_counter: i64,
}

#[derive(Clone)]
pub struct PSO {
    seed: i64,
    max_iteration: i64,
    w: f64,
    c1: f64,
    c2: f64,
    evaluation_function: Arc<dyn Fn(&[f64]) -> (f64, f64) + Send + Sync>,
    variable_bounds: Vec<(f64, f64)>,
    swarm: Vec<PSOParticle>,
    global_best_position: Vec<f64>,
    global_best_obj: f64,
    global_best_violation: f64,
}

pub fn is_better_candidate(
    candidate_obj: f64,
    candidate_violation: f64,
    incumbent_obj: f64,
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
            (c, i) => c < i,
        }
    } else {
        candidate_violation < incumbent_violation
    }
}

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

impl PSOParticle {
    pub fn new() -> Self {
        Default::default()
    }

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

    pub fn local_best_position(&self) -> Vec<f64> {
        self.normalizer.denormalize(&self.local_best_position)
    }

    pub fn local_best_obj(&self) -> f64 {
        self.local_best_obj
    }

    pub fn local_best_violation(&self) -> f64 {
        self.local_best_violation
    }

    pub fn id(&self) -> i64 {
        self.id
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

    pub fn search(&mut self) -> (f64, f64) {
        //Initialization
        for (id, particle) in self.swarm.iter_mut().enumerate() {
            particle.initialize(
                self.seed,
                id as i64,
                self.variable_bounds.clone(),
                self.evaluation_function.clone(),
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

        println!("Initial best solution:\n {:?}", self.global_best_position);

        //search
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

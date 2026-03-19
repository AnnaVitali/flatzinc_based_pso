// use rand::rngs::StdRng;
// use rand::{Rng, SeedableRng};
// use std::f64;

// const VIOLATION_TOLERANCE: f64 = 1e-5;
// const EQUALITY_TOLERANCE: f64 = 1e-5;

// fn eq(a: f64, b: f64) -> bool {
//     (a - b).abs() < EQUALITY_TOLERANCE
// }

// #[derive(Clone, Copy, PartialEq)]
// enum RowType {
//     Equal,
//     Lte,
//     Gte,
// }

// #[derive(Clone, Copy)]
// enum VarType {
//     Continuous,
//     Integer,
// }

// #[derive(Clone, Copy)]
// enum CallbackControlFlow {
//     Terminate,
//     Continue,
// }

// pub struct FJStatus<'a> {
//     pub total_effort: usize,
//     pub effort_since_last_improvement: usize,
//     pub num_vars: usize,
//     pub solution_objective_value: f64,
//     pub solution: Option<&'a [f64]>,
// }

// #[derive(Clone)]
// struct IdxCoeff {
//     idx: usize,
//     coeff: f64,
// }

// struct Var {
//     vartype: VarType,
//     lb: f64,
//     ub: f64,
//     objective_coeff: f64,
//     coeffs: Vec<IdxCoeff>,
// }

// struct Constraint {
//     sense: RowType,
//     rhs: f64,
//     coeffs: Vec<IdxCoeff>,
//     weight: f64,
//     incumbent_lhs: f64,
//     violated_idx: Option<usize>,
// }

// impl Constraint {
//     fn score(&self, lhs: f64) -> f64 {
//         match self.sense {
//             RowType::Equal => -(lhs - self.rhs).abs(),
//             RowType::Lte => -(lhs - self.rhs).max(0.0),
//             RowType::Gte => -(self.rhs - lhs).max(0.0),
//         }
//     }
// }

// #[derive(Clone, Copy)]
// struct Move {
//     value: f64,
//     score: f64,
// }

// impl Move {
//     fn undef() -> Self {
//         Self {
//             value: f64::NAN,
//             score: f64::NEG_INFINITY,
//         }
//     }
// }

// struct LhsModification {
//     var_idx: usize,
//     constraint_idx: usize,
//     coeff: f64,
//     old_lhs: f64,
//     new_lhs: f64,
// }

// struct Problem {
//     vars: Vec<Var>,
//     constraints: Vec<Constraint>,
//     incumbent_assignment: Vec<f64>,
//     violated_constraints: Vec<usize>,
//     incumbent_objective: f64,
//     n_nonzeros: usize,
// }

// impl Problem {
//     fn new() -> Self {
//         Self {
//             vars: Vec::new(),
//             constraints: Vec::new(),
//             incumbent_assignment: Vec::new(),
//             violated_constraints: Vec::new(),
//             incumbent_objective: 0.0,
//             n_nonzeros: 0,
//         }
//     }

//     fn add_var(&mut self, vartype: VarType, lb: f64, ub: f64, obj: f64) -> usize {
//         let idx = self.vars.len();

//         self.vars.push(Var {
//             vartype,
//             lb,
//             ub,
//             objective_coeff: obj,
//             coeffs: Vec::new(),
//         });

//         self.incumbent_assignment.push(lb);
//         idx
//     }

//     fn reset_incumbent(&mut self, initial: Option<&[f64]>) {
//         if let Some(vals) = initial {
//             for (i, v) in vals.iter().enumerate() {
//                 self.incumbent_assignment[i] = *v;
//             }
//         }

//         self.incumbent_objective = 0.0;

//         for (i, var) in self.vars.iter().enumerate() {
//             self.incumbent_objective += var.objective_coeff * self.incumbent_assignment[i];
//         }

//         self.violated_constraints.clear();

//         for (i, c) in self.constraints.iter_mut().enumerate() {
//             let mut lhs = 0.0;

//             for coeff in &c.coeffs {
//                 lhs += coeff.coeff * self.incumbent_assignment[coeff.idx];
//             }

//             c.incumbent_lhs = lhs;

//             if c.score(lhs) < -VIOLATION_TOLERANCE {
//                 c.violated_idx = Some(self.violated_constraints.len());
//                 self.violated_constraints.push(i);
//             } else {
//                 c.violated_idx = None;
//             }
//         }
//     }

//     fn set_value<F>(&mut self, var_idx: usize, new_value: f64, mut f: F) -> usize
//     where
//         F: FnMut(LhsModification),
//     {
//         let old_value = self.incumbent_assignment[var_idx];
//         let delta = new_value - old_value;

//         self.incumbent_assignment[var_idx] = new_value;
//         self.incumbent_objective += self.vars[var_idx].objective_coeff * delta;

//         let mut dt = 0;

//         let coeffs = self.vars[var_idx].coeffs.clone();

//         for ccoeff in coeffs {
//             let c = &mut self.constraints[ccoeff.idx];

//             let old_lhs = c.incumbent_lhs;
//             let new_lhs = old_lhs + ccoeff.coeff * delta;

//             c.incumbent_lhs = new_lhs;

//             let new_cost = c.score(new_lhs);

//             if new_cost < -VIOLATION_TOLERANCE && c.violated_idx.is_none() {
//                 c.violated_idx = Some(self.violated_constraints.len());
//                 self.violated_constraints.push(ccoeff.idx);
//             }

//             if new_cost >= -VIOLATION_TOLERANCE && c.violated_idx.is_some() {
//                 let idx = c.violated_idx.unwrap();
//                 let last = self.violated_constraints.pop().unwrap();

//                 if idx < self.violated_constraints.len() {
//                     self.violated_constraints[idx] = last;
//                     self.constraints[last].violated_idx = Some(idx);
//                 }

//                 c.violated_idx = None;
//             }

//             for var_coeff in &c.coeffs {
//                 if var_coeff.idx != var_idx {
//                     f(LhsModification {
//                         var_idx: var_coeff.idx,
//                         constraint_idx: ccoeff.idx,
//                         coeff: var_coeff.coeff,
//                         old_lhs,
//                         new_lhs,
//                     });
//                 }
//             }

//             dt += c.coeffs.len();
//         }

//         dt
//     }
// }

// fn modify_move(modif: &LhsModification, problem: &Problem, mv: &mut Move) {
//     let c = &problem.constraints[modif.constraint_idx];

//     let incumbent = problem.incumbent_assignment[modif.var_idx];

//     let old_mod_lhs = modif.old_lhs + modif.coeff * (mv.value - incumbent);
//     let new_mod_lhs = modif.new_lhs + modif.coeff * (mv.value - incumbent);

//     let old_term = c.weight * (c.score(old_mod_lhs) - c.score(modif.old_lhs));
//     let new_term = c.weight * (c.score(new_mod_lhs) - c.score(modif.new_lhs));

//     mv.score += new_term - old_term;
// }

// struct JumpMove {
//     moves: Vec<Move>,
// }

// impl JumpMove {
//     fn new() -> Self {
//         Self { moves: Vec::new() }
//     }

//     fn init(&mut self, problem: &Problem) {
//         self.moves = vec![Move::undef(); problem.vars.len()];
//     }

//     fn for_each_var_move<F>(&mut self, var_idx: usize, mut f: F)
//     where
//         F: FnMut(&mut Move),
//     {
//         f(&mut self.moves[var_idx]);
//     }
// }
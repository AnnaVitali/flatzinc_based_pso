use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct ConstraintEvaluation {
    pub constraint_id: usize,
    pub violation: f64,
}

impl PartialEq for ConstraintEvaluation {
    fn eq(&self, other: &Self) -> bool {
        self.constraint_id == other.constraint_id
    }
}

impl Eq for ConstraintEvaluation {}

impl Hash for ConstraintEvaluation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.constraint_id.hash(state);
    }
}

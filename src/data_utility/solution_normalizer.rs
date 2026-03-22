use std::process::exit;

#[derive(Clone)]
pub struct SolutionNormalizer {
    bounds: Vec<(f64, f64)>,
}

impl SolutionNormalizer {
    pub fn new(bounds: Vec<(f64, f64)>) -> Self {
        SolutionNormalizer { bounds }
    }

    pub fn normalize(&self, solution: &Vec<f64>) -> Vec<f64> {
        let mut normalized_solution = Vec::new();
        for (i, value) in solution.iter().enumerate() {
            if let Some((min, max)) = self.bounds.get(i) {
                let range = (*max - *min).max(1.0);
                let normalized_value = (*value - *min) / range;
                normalized_solution.push(normalized_value);
            } else {
                println!("Warning: in normalization Variable {} has undefined bounds. Skipping normalization.", i);
                continue;
            }
        }
        normalized_solution
    }

    pub fn denormalize(&self, normalized_solution: &Vec<f64>) -> Vec<f64> {
        let mut denormalized_solution = Vec::new();
        for (i, normalized_value) in normalized_solution.iter().enumerate() {
            if let Some((min_value, max_value)) = self.bounds.get(i) {
                let range = (*max_value - *min_value).max(1.0);
                let n = (*normalized_value).min(1.0).max(0.0);
                let denormalized_value = n * range + *min_value;
                denormalized_solution.push(denormalized_value);
            } else {
                println!("Warning: in denormalization Variable {} has undefined bounds. Skipping denormalization.", i);
                continue;
            }
        }
        denormalized_solution
    }

    pub(crate) fn default() -> SolutionNormalizer {
        SolutionNormalizer { bounds: Vec::new() }
    }

}
/// A utility for normalizing and denormalizing solutions based on variable bounds.
///
/// This struct stores the lower and upper bounds for each variable and provides methods to
/// normalize a solution (mapping variable values to [0, 1]) and denormalize (mapping normalized values back to the original scale).
#[derive(Clone)]
pub struct SolutionNormalizer {
    /// A vector of (min, max) bounds for each variable.
    bounds: Vec<(f64, f64)>,
}

impl SolutionNormalizer {
    /// Creates a new `SolutionNormalizer` with the provided bounds.
    ///
    /// # Arguments
    /// * `bounds` - A vector of (min, max) bounds for each variable.
    pub fn new(bounds: Vec<(f64, f64)>) -> Self {
        SolutionNormalizer { bounds }
    }

    /// Normalizes a solution using the stored bounds, mapping each variable value to the [0, 1] range.
    ///
    /// # Arguments
    /// * `solution` - A vector of variable values.
    ///
    /// # Returns
    /// A vector of normalized values as `f64` in [0, 1].
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

    /// Denormalizes a solution from the [0, 1] range back to the original variable scale using the stored bounds.
    ///
    /// # Arguments
    /// * `normalized_solution` - A vector of normalized values as `f64` in [0, 1].
    ///
    /// # Returns
    /// A vector of denormalized values as `f64`.
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

    /// Creates a default `SolutionNormalizer` with no bounds set.
    pub(crate) fn default() -> SolutionNormalizer {
        SolutionNormalizer { bounds: Vec::new() }
    }

}
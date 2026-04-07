use std::collections::HashMap;

use crate::data_utility::types::VariableValue;

/// A utility for normalizing and denormalizing MiniZinc solutions based on variable bounds.
///
/// This struct stores the lower and upper bounds for each variable and provides methods to
/// normalize a solution (mapping variable values to [0, 1]) and denormalize (mapping normalized values back to the original scale).
#[derive(Clone)]
pub struct MiniZincSolutionNormalizer {
    /// A map from variable names to their (min, max) bounds as `VariableValue` pairs.
    bounds: HashMap<String, (VariableValue, VariableValue)>,
}

impl MiniZincSolutionNormalizer {
    /// Creates a new `MiniZincSolutionNormalizer` with the provided bounds.
    ///
    /// # Arguments
    /// * `bounds` - A map from variable names to their (min, max) bounds.
    pub fn new(bounds: HashMap<String, (VariableValue, VariableValue)>) -> Self {
        MiniZincSolutionNormalizer { bounds }
    }

    /// Normalizes a solution using the stored bounds, mapping each variable value to the [0, 1] range.
    ///
    /// # Arguments
    /// * `solution` - A map from variable names to their values.
    ///
    /// # Returns
    /// A map from variable names to their normalized values as `f64` in [0, 1].
    pub fn normalize(&self, solution: &HashMap<String, VariableValue>) -> HashMap<String, f64> {
        let mut normalized_solution = HashMap::new();
        for (var_name, var_value) in solution.iter() {
            let Some((min_value, max_value)) = self.bounds.get(var_name) else {
                println!(
                    "In normalization Variable {} has undefined bounds. Skipping normalization.",
                    var_name
                );
                continue;
            };

            match (var_value, min_value, max_value) {
                (VariableValue::Int(value), VariableValue::Int(min), VariableValue::Int(max)) => {
                    let range = (*max as f64 - *min as f64).max(1.0);
                    let normalized_value = (*value as f64 - *min as f64) / range;
                    normalized_solution.insert(var_name.clone(), normalized_value);
                }
                (
                    VariableValue::Float(value),
                    VariableValue::Float(min),
                    VariableValue::Float(max),
                ) => {
                    let range = (*max - *min).max(1.0);
                    let normalized_value = (*value - *min) / range;
                    normalized_solution.insert(var_name.clone(), normalized_value);
                }
                _ => {}
            }
        }

        normalized_solution
    }

    /// Denormalizes a solution from the [0, 1] range back to the original variable scale using the stored bounds.
    ///
    /// # Arguments
    /// * `normalized_solution` - A map from variable names to their normalized values as `f64` in [0, 1].
    ///
    /// # Returns
    /// A map from variable names to their denormalized `VariableValue`.
    pub fn denormalize(
        &self,
        normalized_solution: &HashMap<String, f64>,
    ) -> HashMap<String, VariableValue> {
        let mut denormalized_solution = HashMap::new();
        for (var_name, normalized_value) in normalized_solution.iter() {
            let Some((min_value, max_value)) = self.bounds.get(var_name) else {
                println!(
                    "In denormalization variable {} has undefined bounds. Skipping denormalization.",
                    var_name
                );
                continue;
            };
            match (min_value, max_value) {
                (VariableValue::Int(min), VariableValue::Int(max)) => {
                    let range = (*max as f64 - *min as f64).max(1.0);
                    let n = (*normalized_value).min(1.0).max(0.0);
                    let denormalized_value = (n * range + *min as f64).round() as i64;
                    denormalized_solution
                        .insert(var_name.clone(), VariableValue::Int(denormalized_value));
                }
                (VariableValue::Float(min), VariableValue::Float(max)) => {
                    let range = (*max - *min).max(1.0);
                    let n = (*normalized_value).min(1.0).max(0.0);
                    let denormalized_value = n * range + *min;
                    denormalized_solution
                        .insert(var_name.clone(), VariableValue::Float(denormalized_value));
                }
                _ => {}
            }
        }

        denormalized_solution
    }

    /// Creates a default `MiniZincSolutionNormalizer` with no bounds set.
    pub fn default() -> MiniZincSolutionNormalizer {
        MiniZincSolutionNormalizer {
            bounds: HashMap::new(),
        }
    }
}

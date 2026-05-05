use crate::data_utility::types::{Register, VariableValue};
use flatzinc_serde::{Domain, FlatZinc, Type};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
/// A struct responsible for providing solutions to variables defined in a FlatZinc model, based on the model's output specifications
/// and an optional .ozn file for mapping output variables to their sources.
/// The `SolutionProvider` maintains a mapping of variables that need to be defined, a set of already defined variables,
/// and a mapping of array elements to their corresponding variable names for handling array outputs.
pub struct SolutionProvider {
    /// A vector of registers corresponding to the decision variables that need to be defined in the solution.
    variable_register_map: HashMap<String, Register>,
    /// A vector that holds the current solution values for all variables, indexed by their register numbers. Each entry is an `Option<VariableValue>`,
    solution: Vec<Option<VariableValue>>,
}
/// Implementation of the `SolutionProvider` struct, providing methods to create a new provider,
/// provide values for different types of variables (integers, floats, booleans, sets, arrays), and retrieve the current solution map and defined variables.
impl SolutionProvider {
    /// Creates a new `SolutionProvider` instance by parsing the provided FlatZinc model and .ozn file, initializing the internal mappings for variables, arrays, and output specifications.
    pub fn new(fzn: FlatZinc) -> Self {
        let mut solution: Vec<Option<VariableValue>> = vec![None; fzn.variables.len()];

        let variable_register_map: HashMap<String, Register> = fzn
            .variables
            .iter()
            .enumerate()
            .map(|(i, var)| (var.0.clone(), (i) as Register))
            .collect();

        for (id, var) in &fzn.variables {
            let reg = *variable_register_map
                .get(id)
                .expect("Register not found for variable");
            if !var.defined {
                if var.introduced && matches!(var.ty, Type::IntSet) {
                    let set_value: HashSet<i64> = match &var.domain {
                        Some(Domain::Int(range)) => {
                            let lower = *range.lower_bound().unwrap_or(&0);
                            let upper = *range.upper_bound().unwrap_or(&0);
                            (lower..=upper).collect()
                        }
                        _ => HashSet::new(),
                    };
                    
                    solution[reg as usize] = Some(VariableValue::Set(set_value));
                    continue;
                }
            }
        }
        
        Self {
            variable_register_map,
            solution,
        }
    }

    /// Provides a solution for the variables defined in the FlatZinc model, updating the internal solution vector with the provided values.
    /// 
    /// # Arguments
    /// * `solution` - A slice of `Option<VariableValue>` representing the values for each variable, indexed by their register numbers.
    pub fn provide_solution(&mut self, solution: &[Option<VariableValue>]) {
        for (reg, value) in solution.iter().enumerate() {
            if let Some(val) = value {
                self.solution[reg] = Some(val.clone());
            }
        }
    }

    /// Retrieves the current solution map, which is a vector of `Option<VariableValue>` representing the values for each variable, indexed by their register numbers.
    /// 
    /// # Returns
    /// A vector of `Option<VariableValue>` representing the current solution for all variables.
    pub fn get_partial_solution(&self) -> Vec<Option<VariableValue>> {
        self.solution.clone()
    }

    /// Retrieves the mapping of variable names to their corresponding register numbers, allowing access to variable values based on their names.
    /// 
    /// # Returns
    /// A `HashMap<String, Register>` mapping variable names to their corresponding register numbers.
    pub fn get_vars_register_map(&self) -> HashMap<String, Register> {
        self.variable_register_map.clone()
    }

}

use crate::ozn_parser::ozn_parser::{OznMapping, OznParser};
use flatzinc_serde::{Array, Domain, FlatZinc, Identifier, Literal, Type};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Default)]
/// A struct responsible for providing solutions to variables defined in a FlatZinc model, based on the model's output specifications 
/// and an optional .ozn file for mapping output variables to their sources.
/// The `SolutionProvider` maintains a mapping of variables that need to be defined, a set of already defined variables, 
/// and a mapping of array elements to their corresponding variable names for handling array outputs.
pub struct SolutionProvider {
    /// The original FlatZinc model, containing the variables, arrays, and output specifications.
    fzn: FlatZinc,
    /// A hashmap that stores the variables that need to be defined, mapping variable names to their corresponding `VariableValue`.
    tobe_defined_vars_map: HashMap<String, VariableValue>,
    /// A set of variable names that have already been defined, used to track which variables have been assigned values and to prevent redefinition.
    defined_vars_map: HashSet<String>,
    /// A hashmap that maps variable names to their corresponding string references in the case of array elements.
    array_elements_map: HashMap<String, String>,
    /// A vector of identifiers representing the output variables specified in the FlatZinc model..
    output: Vec<Identifier>,
    /// A hashmap that maps identifiers to their corresponding arrays, used for resolving array references when providing solutions for array variables.
    arrays: HashMap<Identifier, Array>,
    /// An instance of `OznParser` used to parse .ozn files and map output variables to their sources.
    ozn_parser: OznParser,
}
/// Implementation of the `SolutionProvider` struct, providing methods to create a new provider,
/// provide values for different types of variables (integers, floats, booleans, sets, arrays), and retrieve the current solution map and defined variables.
impl SolutionProvider {

    /// Creates a new `SolutionProvider` instance by parsing the provided FlatZinc model and .ozn file, initializing the internal mappings for variables, arrays, and output specifications.
    pub fn new(fzn: FlatZinc, ozn: &Path) -> Self {
        let mut tobe_defined_vars_map = HashMap::new();
        let array_elements_map = HashMap::new();
        let arrays: HashMap<Identifier, Array> = fzn.arrays
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        let output = fzn.output.clone();

        let mut defined_vars_map: HashSet<String> = HashSet::with_capacity(fzn.variables.len());
        for (id, var) in &fzn.variables {
            if var.defined {
                defined_vars_map.insert(id.clone());
            }

            if var.introduced && matches!(var.ty, Type::IntSet) {
                let set_value: HashSet<i64> = match &var.domain {
                    Some(Domain::Int(range)) => {
                        let lower = *range.lower_bound().unwrap_or(&0);
                        let upper = *range.upper_bound().unwrap_or(&0);
                        if lower > upper {
                            HashSet::new()
                        } else {
                            (lower..=upper).collect()
                        }
                    }
                    _ => HashSet::new(),
                };

                tobe_defined_vars_map.insert(id.clone(), VariableValue::Set(set_value));
            }
        }
        
        let mut ozn_parser = OznParser::new();
        ozn_parser.parse(ozn);

        Self {
            fzn,
            tobe_defined_vars_map,
            defined_vars_map,
            array_elements_map,
            output,
            arrays,
            ozn_parser,
        }
    }

    /// Provides an integer value for a variable, updating the internal mapping of variables to be defined and marking the variable as defined.
    /// 
    /// # Arguments
    /// * `name` - The name of the variable for which the value is being provided
    /// * `value` - The integer value to be assigned to the variable
    pub fn provide_int(&mut self, name: String, value: i64) {
        let target_name = self.find_variable_name(&name);
        self.tobe_defined_vars_map.insert(target_name, VariableValue::Int(value));
    }

    /// Provides a float value for a variable, updating the internal mapping of variables to be defined and marking the variable as defined.
    /// 
    /// # Arguments
    /// * `name` - The name of the variable for which the value is being provided
    /// * `value` - The float value to be assigned to the variable
    pub fn provide_float(&mut self, name: String, value: f64) {
        let target_name = self.find_variable_name(&name);
        self.tobe_defined_vars_map.insert(target_name, VariableValue::Float(value));
    }

    /// Provides a boolean value for a variable, updating the internal mapping of variables to be defined and marking the variable as defined.
    ///
    /// # Arguments
    /// * `name` - Variable name
    /// * `value` - Boolean value to assign
    pub fn provide_bool(&mut self, name: String, value: bool) {
        let target_name = self.find_variable_name(&name);
        self.tobe_defined_vars_map.insert(target_name, VariableValue::Bool(value));
    }

    /// Provides a set value for a variable, updating the internal mapping of variables to be defined and marking the variable as defined.
    ///
    /// # Arguments
    /// * `name` - Variable name
    /// * `value` - Set value to assign
    pub fn provide_set(&mut self, name: String, value: HashSet<i64>) {
        let target_name = self.find_variable_name(&name);
        self.tobe_defined_vars_map.insert(target_name, VariableValue::Set(value));
    }

    /// Provides an array of integer values for a variable, updating the internal mapping of variables to be defined and marking the variable as defined.
    ///
    /// # Arguments
    /// * `name` - Array variable name
    /// * `value` - Vector of integer values
    pub fn provide_array_of_int(&mut self, name: String, value: Vec<i64>) {
        let target_array_name = self.find_variable_name(&name);

        let array_fzn = self.arrays.get(&target_array_name).expect(&format!(
            "FlatZinc array '{}' not found in symbol table",
            target_array_name
        ));

        for (id, elem) in array_fzn.contents.iter().enumerate() {
            if let Literal::Identifier(var_fzn_name) = elem {
                let val = value
                    .get(id)
                    .expect("Provided value vector is shorter than the array length");

                self.tobe_defined_vars_map
                    .insert(var_fzn_name.to_string(), VariableValue::Int(*val));

                let str_reference = format!("{}[{}]", name, id);
                self.array_elements_map
                    .insert(var_fzn_name.to_string(), str_reference);
            }
        }
    }

    /// Provides a matrix of integer values for a variable, updating the internal mapping of variables to be defined and marking the variable as defined.
    ///
    /// # Arguments
    /// * `name` - Matrix variable name
    /// * `value` - 2D vector of integer values
    pub fn provide_matrix_of_int(&mut self, name: String, value: Vec<Vec<i64>>) {
        let cols = value[0].len();

        for row in &value {
            if row.len() != cols {
                panic!("Provided matrix is ragged: all rows must have the same length");
            }
        }

        let mut flat: Vec<i64> = Vec::with_capacity(value.len() * cols);
        for row in value {
            for v in row {
                flat.push(v);
            }
        }

        self.provide_array_of_int(name, flat);
    }

    /// Provides an array of float values for a variable, updating the internal mapping of variables to be defined and marking the variable as defined.
    ///
    /// # Arguments
    /// * `name` - Array variable name
    /// * `value` - Vector of float values
    pub fn provide_array_of_float(&mut self, name: String, value: Vec<f64>) {
        let target_array_name = self.find_variable_name(&name);

        let array_fzn = self.arrays.get(&target_array_name).expect(&format!(
            "FlatZinc array '{}' not found in symbol table",
            target_array_name
        ));

        for (id, elem) in array_fzn.contents.iter().enumerate() {
            if let Literal::Identifier(var_fzn_name) = elem {
                let val = value
                    .get(id)
                    .expect("Provided value vector is shorter than the array length");

                self.tobe_defined_vars_map
                    .insert(var_fzn_name.to_string(), VariableValue::Float(*val));

                let str_reference = format!("{}[{}]", name, id);
                self.array_elements_map
                    .insert(var_fzn_name.to_string(), str_reference);
            }
        }
    }

    /// Provides a matrix of float values for a variable, updating the internal mapping of variables to be defined and marking the variable as defined.
    ///
    /// # Arguments
    /// * `name` - Matrix variable name
    /// * `value` - 2D vector of float values
    pub fn provide_matrix_of_float(&mut self, name: String, value: Vec<Vec<f64>>) {
        let cols = value[0].len();

        for row in &value {
            if row.len() != cols {
                panic!("Provided matrix is ragged: all rows must have the same length");
            }
        }

        let mut flat: Vec<f64> = Vec::with_capacity(value.len() * cols);
        for row in value {
            for v in row {
                flat.push(v);
            }
        }

        self.provide_array_of_float(name, flat);
    }

    /// Provides an array of boolean values for a variable, updating the internal mapping of variables to be defined and marking the variable as defined.
    ///
    /// # Arguments
    /// * `name` - Array variable name
    /// * `value` - Vector of boolean values
    pub fn provide_array_of_bool(&mut self, name: String, value: Vec<bool>) {
        let target_array_name = self.find_variable_name(&name);

        let array_fzn = self.arrays.get(&target_array_name).expect(&format!(
            "FlatZinc array '{}' not found in symbol table",
            target_array_name
        ));

        for (id, elem) in array_fzn.contents.iter().enumerate() {
            if let Literal::Identifier(var_fzn_name) = elem {
                let val = value
                    .get(id)
                    .expect("Provided value vector is shorter than the array length");

                self.tobe_defined_vars_map
                    .insert(var_fzn_name.to_string(), VariableValue::Bool(*val));

                let str_reference = format!("{}[{}]", name, id);
                self.array_elements_map
                    .insert(var_fzn_name.to_string(), str_reference);
            }
        }
    }

    /// Provides a matrix of boolean values for a variable, updating the internal mapping of variables to be defined and marking the variable as defined.
    ///
    /// # Arguments
    /// * `name` - Matrix variable name
    /// * `value` - 2D vector of boolean values
    pub fn provide_matrix_of_bool(&mut self, name: String, value: Vec<Vec<bool>>) {
        let cols = value[0].len();

        for row in &value {
            if row.len() != cols {
                panic!("Provided matrix is ragged: all rows must have the same length");
            }
        }

        let mut flat: Vec<bool> = Vec::with_capacity(value.len() * cols);
        for row in value {
            for v in row {
                flat.push(v);
            }
        }

        self.provide_array_of_bool(name, flat);
    }

    /// Provides an array of set values for a variable, updating the internal mapping of variables to be defined and marking the variable as defined.
    ///
    /// # Arguments
    /// * `name` - Array variable name
    /// * `value` - Vector of set values
    pub fn provide_array_of_set(&mut self, name: String, value: Vec<HashSet<i64>>) {
        let target_array_name = self.find_variable_name(&name);

        let array_fzn = self.arrays.get(&target_array_name).expect(&format!(
            "FlatZinc array '{}' not found in symbol table",
            target_array_name
        ));

        for (id, elem) in array_fzn.contents.iter().enumerate() {
            if let Literal::Identifier(var_fzn_name) = elem {
                let val = value
                    .get(id)
                    .expect("Provided value vector is shorter than the array length");

                self.tobe_defined_vars_map
                    .insert(var_fzn_name.to_string(), VariableValue::Set(val.clone()));

                let str_reference = format!("{}[{}]", name, id);
                self.array_elements_map
                    .insert(var_fzn_name.to_string(), str_reference);
            }
        }
    }

    fn find_variable_name(&mut self, name: &String) -> String {
        let target_name = if self.variable_exist(&name) {
            name.clone()
        } else {
            match self.ozn_parser.map_variable(&name) {
                OznMapping::Array { source } => source.clone(),
                _ => panic!("The provided variable is not recognized as an array"),
            }
        };
        target_name
    }

    fn variable_exist(&self, name: &str) -> bool {
        self.output.contains(&name.to_string())
            || self.fzn.variables.iter().any(|(id, _)| id == name)
    }

    /// Returns a reference to the map of variables to their values.
    pub fn solution_map(&self) -> &HashMap<String, VariableValue> {
        &self.tobe_defined_vars_map
    }

    /// Returns a reference to the set of defined variable names.
    pub fn defined_vars_map(&self) -> &HashSet<String> {
        &self.defined_vars_map
    }

    /// Returns a reference to the map of array element references.
    pub fn array_elements_map(&self) -> &HashMap<String, String> {
        &self.array_elements_map
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Set(HashSet<i64>),
}

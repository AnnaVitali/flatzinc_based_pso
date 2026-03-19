use crate::ozn_parser::ozn_parser::{OznMapping, OznParser};
use flatzinc_serde::{Array, Domain, FlatZinc, Identifier, Literal, Type};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct SolutionProvider {
    fzn: FlatZinc,
    tobe_defined_vars_map: HashMap<String, VariableValue>,
    defined_vars_map: HashSet<String>,
    array_elements_map: HashMap<String, String>,
    output: Vec<Identifier>,
    arrays: HashMap<Identifier, Array>,
    ozn_parser: OznParser,
}

impl SolutionProvider {
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

    pub fn provide_int(&mut self, name: String, value: i64) {
        let target_name = self.find_variable_name(&name);
        self.tobe_defined_vars_map.insert(target_name, VariableValue::Int(value));
    }

    pub fn provide_float(&mut self, name: String, value: f64) {
        let target_name = self.find_variable_name(&name);
        self.tobe_defined_vars_map.insert(target_name, VariableValue::Float(value));
    }

    pub fn provide_bool(&mut self, name: String, value: bool) {
        let target_name = self.find_variable_name(&name);
        self.tobe_defined_vars_map.insert(target_name, VariableValue::Bool(value));
    }

    pub fn provide_set(&mut self, name: String, value: HashSet<i64>) {
        let target_name = self.find_variable_name(&name);
        self.tobe_defined_vars_map.insert(target_name, VariableValue::Set(value));
    }

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

    pub fn solution_map(&self) -> &HashMap<String, VariableValue> {
        &self.tobe_defined_vars_map
    }

    pub fn defined_vars_map(&self) -> &HashSet<String> {
        &self.defined_vars_map
    }

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

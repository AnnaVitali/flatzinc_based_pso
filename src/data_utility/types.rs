use std::collections::HashSet;

pub(crate) type Register = u32;

#[derive(Debug, Clone, PartialEq)]
pub enum VariableValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Set(HashSet<i64>),
}

impl From<i64> for VariableValue {
    fn from(value: i64) -> Self {
        VariableValue::Int(value)
    }
}

impl From<f64> for VariableValue {
    fn from(value: f64) -> Self {
        VariableValue::Float(value)
    }
}

impl VariableValue {
    pub fn as_int(&self) -> i64 {
        match self {
            VariableValue::Int(value) => *value,
            VariableValue::Float(_) => unreachable!(),
            VariableValue::Bool(_) => unreachable!(),
            VariableValue::Set(_) => unreachable!(),
        }
    }

    pub fn as_float(&self) -> f64 {
        match self {
            VariableValue::Float(value) => *value,
            VariableValue::Int(_) => unreachable!(),
            VariableValue::Bool(_) => unreachable!(),
            VariableValue::Set(_) => unreachable!(),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            VariableValue::Bool(value) => *value,
            VariableValue::Int(_) => unreachable!(),
            VariableValue::Float(_) => unreachable!(),
            VariableValue::Set(_) => unreachable!(),
        }
    }

    pub fn as_set(&self) -> HashSet<i64> {
        match self {
            VariableValue::Set(set) => set.clone(),
            VariableValue::Int(_) => unreachable!(),
            VariableValue::Float(_) => unreachable!(),
            VariableValue::Bool(_) => unreachable!(),
        }
    }
}
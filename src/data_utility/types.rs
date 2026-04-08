use std::collections::HashSet;

/// A type alias for a register, represented as a `u32`.
pub(crate) type Register = u32;

#[derive(Debug, Clone, PartialEq)]
/// An enum representing the value of a variable, which can be an integer, float, boolean, or a set of integers.
pub enum VariableValue {
    /// An integer value.
    Int(i64),
    /// A floating-point value.
    Float(f64),
    /// A boolean value.
    Bool(bool),
    /// A set of integers.
    Set(HashSet<i64>),
}

/// Implementations for converting from primitive types to `VariableValue` and extracting values from `VariableValue`.
impl From<i64> for VariableValue {
    fn from(value: i64) -> Self {
        VariableValue::Int(value)
    }
}

/// Implement the `From` trait for `f64` to allow easy conversion to `VariableValue::Float`.
impl From<f64> for VariableValue {
    fn from(value: f64) -> Self {
        VariableValue::Float(value)
    }
}

/// Implement the `From` trait for `bool` to allow easy conversion to `VariableValue::Bool`.
impl VariableValue {

    /// Implement the `From` trait for `HashSet<i64>` to allow easy conversion to `VariableValue::Set`.
    /// 
    /// # Returns
    /// A `VariableValue::Set` containing the provided set of integers.
    pub fn as_int(&self) -> i64 {
        match self {
            VariableValue::Int(value) => *value,
            VariableValue::Float(_) => unreachable!(),
            VariableValue::Bool(_) => unreachable!(),
            VariableValue::Set(_) => unreachable!(),
        }
    }

    /// Extracts a float value from the `VariableValue`, panicking if the variant is not `Float`.
    /// 
    /// # Returns
    /// The float value contained in the `VariableValue::Float` variant.
    pub fn as_float(&self) -> f64 {
        match self {
            VariableValue::Float(value) => *value,
            VariableValue::Int(_) => unreachable!(),
            VariableValue::Bool(_) => unreachable!(),
            VariableValue::Set(_) => unreachable!(),
        }
    }

    /// Extracts a boolean value from the `VariableValue`, panicking if the variant is not `Bool`.
    /// 
    /// # Returns
    /// The boolean value contained in the `VariableValue::Bool` variant.
    pub fn as_bool(&self) -> bool {
        match self {
            VariableValue::Bool(value) => *value,
            VariableValue::Int(_) => unreachable!(),
            VariableValue::Float(_) => unreachable!(),
            VariableValue::Set(_) => unreachable!(),
        }
    }

    /// Extracts a set of integers from the `VariableValue`, panicking if the variant is not `Set`.
    /// 
    /// # Returns
    /// A `HashSet<i64>` containing the integers in the `VariableValue::Set` variant.
    pub fn as_set(&self) -> HashSet<i64> {
        match self {
            VariableValue::Set(set) => set.clone(),
            VariableValue::Int(_) => unreachable!(),
            VariableValue::Float(_) => unreachable!(),
            VariableValue::Bool(_) => unreachable!(),
        }
    }
}
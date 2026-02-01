//! Kiro Runtime - Shared types for Kiro-Rust FFI
//!
//! This crate defines the runtime value representation and error types
//! used at the boundary between Kiro code and Rust glue functions.

use std::collections::HashMap;
use std::convert::TryFrom;

/// Runtime value representation for Kiro types.
/// This enum is used by:
/// - Compiler-generated Rust code
/// - Rust glue functions in header.rs
#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeVal {
    Num(f64),
    Str(String),
    Bool(bool),
    List(Vec<RuntimeVal>),
    Map(HashMap<String, RuntimeVal>),
    Void,
}

/// Kiro error type for Rust glue functions.
/// The `name` field must match a Kiro error definition.
#[derive(Clone, Debug)]
pub struct KiroError {
    pub name: String,
}

impl KiroError {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl std::fmt::Display for KiroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::error::Error for KiroError {}

// --- Conversion: Rust types -> RuntimeVal ---

impl From<f64> for RuntimeVal {
    fn from(v: f64) -> Self {
        RuntimeVal::Num(v)
    }
}

impl From<String> for RuntimeVal {
    fn from(v: String) -> Self {
        RuntimeVal::Str(v)
    }
}

impl From<&str> for RuntimeVal {
    fn from(v: &str) -> Self {
        RuntimeVal::Str(v.to_string())
    }
}

impl From<bool> for RuntimeVal {
    fn from(v: bool) -> Self {
        RuntimeVal::Bool(v)
    }
}

impl From<()> for RuntimeVal {
    fn from(_: ()) -> Self {
        RuntimeVal::Void
    }
}

impl<T: Into<RuntimeVal>> From<Vec<T>> for RuntimeVal {
    fn from(v: Vec<T>) -> Self {
        RuntimeVal::List(v.into_iter().map(|x| x.into()).collect())
    }
}

// --- Conversion: RuntimeVal -> Rust types ---

impl RuntimeVal {
    pub fn as_str(&self) -> Result<&str, KiroError> {
        match self {
            RuntimeVal::Str(s) => Ok(s.as_str()),
            _ => Err(KiroError::new("TypeError")),
        }
    }

    pub fn as_num(&self) -> Result<f64, KiroError> {
        match self {
            RuntimeVal::Num(n) => Ok(*n),
            _ => Err(KiroError::new("TypeError")),
        }
    }

    pub fn as_bool(&self) -> Result<bool, KiroError> {
        match self {
            RuntimeVal::Bool(b) => Ok(*b),
            _ => Err(KiroError::new("TypeError")),
        }
    }
}

impl TryFrom<RuntimeVal> for String {
    type Error = KiroError;
    fn try_from(val: RuntimeVal) -> Result<Self, Self::Error> {
        match val {
            RuntimeVal::Str(s) => Ok(s),
            _ => Err(KiroError::new("TypeError")),
        }
    }
}

impl TryFrom<RuntimeVal> for f64 {
    type Error = KiroError;
    fn try_from(val: RuntimeVal) -> Result<Self, Self::Error> {
        match val {
            RuntimeVal::Num(n) => Ok(n),
            _ => Err(KiroError::new("TypeError")),
        }
    }
}

impl TryFrom<RuntimeVal> for bool {
    type Error = KiroError;
    fn try_from(val: RuntimeVal) -> Result<Self, Self::Error> {
        match val {
            RuntimeVal::Bool(b) => Ok(b),
            _ => Err(KiroError::new("TypeError")),
        }
    }
}

impl TryFrom<RuntimeVal> for () {
    type Error = KiroError;
    fn try_from(val: RuntimeVal) -> Result<Self, Self::Error> {
        match val {
            RuntimeVal::Void => Ok(()),
            _ => Err(KiroError::new("TypeError")),
        }
    }
}

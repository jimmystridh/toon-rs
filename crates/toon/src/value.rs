#![allow(dead_code)]

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

use crate::number::format_canonical_f64;

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    I64(i64),
    U64(u64),
    F64(f64),
}

impl core::fmt::Display for Number {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Number::I64(i) => write!(f, "{}", i),
            Number::U64(u) => write!(f, "{}", u),
            Number::F64(num) => f.write_str(&format_canonical_f64(*num)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Value {
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
        )
    }
}

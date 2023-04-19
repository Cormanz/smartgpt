use std::{collections::HashMap, fmt::Display, error::Error};
use async_recursion::async_recursion;

use crate::{Statement, Expression, Primitive, CommandContext, Plugin};

use super::Body;

#[derive(Debug)]
pub struct GPTRunError(pub String);

impl Display for GPTRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GPT Script Parse Error: {}", self.0)
    }
}

impl Error for GPTRunError {}

#[derive(Debug)]
pub struct CannotConvertError(pub String);

impl Display for CannotConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert to {}", self.0)
    }
}

impl Error for CannotConvertError {}

#[derive(Debug, Clone)]
pub enum ScriptValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<ScriptValue>),
    Dict(HashMap<String, ScriptValue>),
    None
}

impl From<ScriptValue> for Expression {
    fn from(value: ScriptValue) -> Self {
        match value {
            ScriptValue::String(string) => Expression::Primitive(Primitive::String(string)),
            ScriptValue::Int(int) => Expression::Primitive(Primitive::Int(int)),
            ScriptValue::Float(float) => Expression::Primitive(Primitive::Float(float)),
            ScriptValue::Bool(bool) => Expression::Primitive(Primitive::Bool(bool)),
            ScriptValue::List(list) => Expression::List(
                list.iter()
                    .map(|el| el.clone().into())
                    .collect::<Vec<_>>()
            ),
            ScriptValue::Dict(dict) => Expression::Dict(
                dict.iter()
                    .map(|(key, value)| (key.clone(), value.clone().into()))
                    .collect::<HashMap<_, _>>()
            ),
            ScriptValue::None => Expression::Primitive(Primitive::None)
        }
    }
}

impl TryFrom<ScriptValue> for String {
    type Error = CannotConvertError;

    fn try_from(value: ScriptValue) -> Result<Self, Self::Error> {
        match value {
            ScriptValue::String(text) => Ok(text),
            _ => Err(CannotConvertError("String".to_string()))
        }
    }
}

impl TryFrom<ScriptValue> for bool {
    type Error = CannotConvertError;

    fn try_from(value: ScriptValue) -> Result<Self, Self::Error> {
        match value {
            ScriptValue::Bool(bool) => Ok(bool),
            _ => Err(CannotConvertError("bool".to_string()))
        }
    }
}

impl TryFrom<ScriptValue> for i64 {
    type Error = CannotConvertError;

    fn try_from(value: ScriptValue) -> Result<Self, Self::Error> {
        match value {
            ScriptValue::Int(int) => Ok(int),
            _ => Err(CannotConvertError("i64".to_string()))
        }
    }
}

impl TryFrom<ScriptValue> for f64 {
    type Error = CannotConvertError;

    fn try_from(value: ScriptValue) -> Result<Self, Self::Error> {
        match value {
            ScriptValue::Float(float) => Ok(float),
            _ => Err(CannotConvertError("f64".to_string()))
        }
    }
}

impl TryFrom<ScriptValue> for Vec<ScriptValue> {
    type Error = CannotConvertError;

    fn try_from(value: ScriptValue) -> Result<Self, Self::Error> {
        match value {
            ScriptValue::List(list) => Ok(list),
            _ => Err(CannotConvertError("Vec<ScriptValue>".to_string()))
        }
    }
}

impl TryFrom<ScriptValue> for HashMap<String, ScriptValue> {
    type Error = CannotConvertError;

    fn try_from(value: ScriptValue) -> Result<Self, Self::Error> {
        match value {
            ScriptValue::Dict(dict) => Ok(dict),
            _ => Err(CannotConvertError("HashMap<String, ScriptValue>".to_string()))
        }
    }
}

impl From<String> for ScriptValue {
    fn from(string: String) -> Self {
        ScriptValue::String(string)
    }
}

impl From<i64> for ScriptValue {
    fn from(int: i64) -> Self {
        ScriptValue::Int(int)
    }
}

impl From<f64> for ScriptValue {
    fn from(float: f64) -> Self {
        ScriptValue::Float(float)
    }
}

impl From<bool> for ScriptValue {
    fn from(b: bool) -> Self {
        ScriptValue::Bool(b)
    }
}

impl From<Vec<ScriptValue>> for ScriptValue {
    fn from(list: Vec<ScriptValue>) -> Self {
        ScriptValue::List(list)
    }
}

impl From<HashMap<String, ScriptValue>> for ScriptValue {
    fn from(dict: HashMap<String, ScriptValue>) -> Self {
        ScriptValue::Dict(dict)
    }
}
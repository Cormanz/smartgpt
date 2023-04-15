use std::{collections::HashMap, fmt::Display, error::Error};
use async_recursion::async_recursion;

use crate::{Statement, Expression, Primitive, CommandContext};

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

#[async_recursion]
pub async fn get_value(ctx: &mut CommandContext, expr: Expression, top_level: bool) -> Result<ScriptValue, GPTRunError> {
    match expr {
        Expression::Name(name) => {
            let value = ctx.variables.get(&name)
                .ok_or(GPTRunError(format!("No variable {name}")))?;
            Ok(value.clone())
        }
        Expression::Primitive(primitive) => {
            Ok(match primitive {
                Primitive::String(string) => ScriptValue::String(string.clone()),
                Primitive::Int(int) => ScriptValue::Int(int),
                Primitive::Float(float) => ScriptValue::Float(float),
                Primitive::Bool(bool) => ScriptValue::Bool(bool),
                Primitive::None => ScriptValue::None
            })
        },
        Expression::List(list) => {
            let mut out: Vec<ScriptValue> = vec![];
            for expr in list {
                let expr = get_value(ctx, expr, false).await?;
                out.push(expr);
            }
            Ok(ScriptValue::List(out))          
        },
        Expression::Dict(dict) => {
            let mut map = HashMap::<String, ScriptValue>::new();

            for (key, value) in dict {
                let expr = get_value(ctx, value, false).await?;
                map.insert(key, expr);
            }

            Ok(ScriptValue::Dict(map))
        },
        Expression::FunctionCall(name, args) => {
            let mut value_args: Vec<ScriptValue> = vec![];
            for arg in args {
                value_args.push(get_value(ctx, arg, false).await?);
            }

            let result = ScriptValue::String("Yay!".to_string());

            if top_level {
                let args: Vec<Expression> = value_args.iter().map(|el| el.clone().into()).collect();
                let expr = Expression::FunctionCall(name.clone(), args);

                println!("Command {:?} returned: {:?}", expr, result);
            }

            Ok(result)
        }
    }
}

pub fn assign(ctx: &mut CommandContext, name: Expression, target: ScriptValue) -> Result<(), GPTRunError> {
    match name {
        Expression::Name(name) => {
            ctx.variables.insert(name, target);
        }
        Expression::List(names) => {
            match target {
                ScriptValue::List(list) => {
                    for (name, item) in names.iter().zip(list) {
                        assign(ctx, name.clone(), item)?
                    }
                },
                _ => {
                    return Err(GPTRunError(format!("Cannot save variables {:?} if the value isn't a list.", names)))
                }
            }
        }
        _ => {
            return Err(GPTRunError(format!("Cannot save variables with destructure-type {:?}", name)));
        }
    }

    Ok(())
}

#[async_recursion]
pub async fn run_body(ctx: &mut CommandContext, body: Body) -> Result<(), GPTRunError> {
    for statement in body {
        match statement {
            Statement::Assign(name, target) => {
                let value = get_value(ctx, target, false).await?;
                assign(ctx, name, value)?;
            }
            Statement::Expression(expr) => {
                get_value(ctx, expr, true).await?;
            }
            Statement::For(target, iter, body) => {
                let iter = get_value(ctx, iter, false).await?;
                let list = match iter {
                    ScriptValue::List(list) => list.clone(),
                    ScriptValue::Dict(dict) => dict.iter()
                        .map(|el| ScriptValue::List(vec![ 
                            ScriptValue::String(el.0.clone()), 
                            el.1.clone() ])
                        )
                        .collect::<Vec<_>>(),
                    _ => {
                        return Err(GPTRunError(format!("Cannot iter over {:?}", iter)));
                    }
                };
                for item in list {
                    assign(ctx, target.clone(), item)?;
                    run_body(ctx, body.clone()).await?;
                }
            }
        }
    }

    Ok(())
}
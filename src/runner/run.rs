use std::{collections::HashMap, fmt::Display, error::Error};
use async_recursion::async_recursion;

use crate::{Statement, Expression, Primitive};

use super::Body;

#[derive(Debug)]
pub struct GPTRunError(pub String);

impl Display for GPTRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GPT Script Parse Error: {}", self.0)
    }
}

impl Error for GPTRunError {}

#[derive(Debug, Clone)]
pub enum ScriptValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<ScriptValue>),
    Dict(HashMap<String, ScriptValue>)
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
            )
        }
    }
}

pub struct ScriptContext {
    pub variables: HashMap<String, ScriptValue>
}

#[async_recursion]
pub async fn get_value(ctx: &mut ScriptContext, expr: Expression, top_level: bool) -> Result<ScriptValue, GPTRunError> {
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

pub fn assign(ctx: &mut ScriptContext, name: Expression, target: ScriptValue) -> Result<(), GPTRunError> {
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
pub async fn run_body(ctx: &mut ScriptContext, body: Body) -> Result<(), GPTRunError> {
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
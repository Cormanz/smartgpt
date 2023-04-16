use std::{error::Error, collections::HashMap, fmt::{Display, Debug}};
use rustpython_parser::{parser::{self, parse_program}, ast::{self, StmtKind, ExprKind, Constant, Expr, Located}};
use num_traits::ToPrimitive;

#[derive(Debug)]
pub struct GPTParseError(pub String);

impl Display for GPTParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GPT Script Parse Error: {}", self.0)
    }
}

impl Error for GPTParseError {}

#[derive(Clone)]
pub enum Primitive {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    None
}

impl Debug for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::String(string) => {
                if string.len() > 100 {
                    let mut text = format!("{:?}", string).chars()
                        .take(100)
                        .map(|el| el.to_string())
                        .collect::<Vec<_>>()
                        .join("");
                    text.push_str(&r#"...""#);
                    write!(f, "{}", text)
                } else {
                    write!(f, "{:?}", string)
                }
            }
            Primitive::Bool(bool) => {
                write!(f, "{}", bool)
            }
            Primitive::Int(int) => {
                write!(f, "{}", int)
            }
            Primitive::Float(float) => {
                write!(f, "{}", float)
            }
            Primitive::None => {
                write!(f, "none")
            }
        }
    }
}

#[derive(Clone)]
pub enum Expression {
    Name(String),
    Primitive(Primitive),
    List(Vec<Expression>),
    Dict(HashMap<String, Expression>),
    FunctionCall(String, Vec<Expression>),
    GetAttr(Box<Expression>, Box<Expression>)
}

impl Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Primitive(primitive) => {
                write!(f, "{:?}", primitive)
            }
            Expression::Name(name) => {
                write!(f, "{}", name)
            }
            Expression::List(list) => {
                write!(f, "[ ")?;
                for (ind, item) in list.iter().enumerate() {
                    write!(f, "{:?}", item)?;
                    if ind < list.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "] ")
            }
            Expression::Dict(dict) => {
                write!(f, "{{ ")?;
                for (ind, (key, value)) in dict.iter().enumerate() {
                    write!(f, "{:?}: {:?}", key, value)?;
                    if ind < dict.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}} ")                
            }
            Expression::FunctionCall(name, args) => {
                write!(f, "{}(", name)?;
                for (ind, item) in args.iter().enumerate() {
                    write!(f, "{:?}", item)?;
                    if ind < args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")
            }
            Expression::GetAttr(expr, attr) => {
                write!(f, "{:?}[{:?}]", expr, attr)
            }
        }
    }
}

impl From<bool> for Expression {
    fn from(value: bool) -> Self {
        Expression::Primitive(Primitive::Bool(value))
    }
}

impl From<i64> for Expression {
    fn from(value: i64) -> Self {
        Expression::Primitive(Primitive::Int(value))
    }
}

impl From<f64> for Expression {
    fn from(value: f64) -> Self {
        Expression::Primitive(Primitive::Float(value))
    }
}

impl From<String> for Expression {
    fn from(value: String) -> Self {
        Expression::Primitive(Primitive::String(value.clone()))
    }
}

pub fn to_expr(node: ExprKind) -> Result<Expression, GPTParseError> {
    match node {
        ExprKind::Call { func, args, keywords } => {
            let func = match func.node {
                ExprKind::Name { id, .. } => Ok(id),
                other => Err(GPTParseError(format!("Cannot handle function call applied to {:?}", other)))
            }?;
            let mut arguments: Vec<Expression> = vec![];
            for arg in args {
                arguments.push(to_expr(arg.node)?);
            }

            Ok(Expression::FunctionCall(func, arguments))
        }
        ExprKind::Subscript { value, slice, ctx } => {
            let value = to_expr(value.node.clone())?;
            let slice = to_expr(slice.node.clone())?;

            Ok(Expression::GetAttr(Box::new(value), Box::new(slice)))
        }
        ExprKind::Constant { value, .. } => {
            match value {
                Constant::Bool(bool) => Ok(bool.into()),
                Constant::Int(int) => int.to_i64()
                    .map(|el| el.into())
                    .ok_or(GPTParseError(format!("Cannot parse into i64, {:?}", int))),
                Constant::Float(float) => float.to_f64()
                    .map(|el| el.into())
                    .ok_or(GPTParseError(format!("Cannot parse into f64, {:?}", float))),
                Constant::Str(string) => Ok(string.into()),
                _ => Err(GPTParseError(format!("Cannot parse constant {:?}", value)))
            }
        }
        ExprKind::Name { id, .. } => {
            Ok(Expression::Name(id.clone()))
        }
        ExprKind::List { elts, .. } => {
            let mut list: Vec<Expression> = vec![];
            for expr in elts {
                let expr = to_expr(expr.node)?;
                list.push(expr);
            }
            Ok(Expression::List(list))
        }
        ExprKind::Dict { keys, values } => {
            let mut parsed_keys: Vec<String> = vec![];
            for expr in keys {
                let expr = to_expr(expr.node)?;
                let name = match expr {
                    Expression::Primitive(primitive) => {
                        match primitive {
                            Primitive::String(text) => Ok(text),
                            _ => Err(GPTParseError(format!("Cannot handle map key, {:?}", primitive)))
                        }
                    },
                    _ => Err(GPTParseError(format!("Cannot handle map key, {:?}", expr)))
                }?;
                parsed_keys.push(name);
            }

            let mut parsed_values: Vec<Expression> = vec![];
            for expr in values {
                let expr = to_expr(expr.node)?;
                parsed_values.push(expr);
            }

            let hash_map: HashMap<String, Expression> = parsed_keys.into_iter()
                .zip(parsed_values.into_iter())
                .collect();

            Ok(Expression::Dict(hash_map))
        },
        other => Err(GPTParseError(format!("Cannot parse expression {:?}", other)))
    }
}

pub type Body = Vec<Statement>;

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Assign(Expression, Expression),
    For(Expression, Expression, Body)
}

pub fn to_statement(statement: StmtKind) -> Result<Statement, GPTParseError> {
    match statement {
        StmtKind::Expr { value } => {
            to_expr(value.node).map(|el| Statement::Expression(el))
        }
        StmtKind::Assign { targets, value, type_comment } => {
            let target = to_expr(targets[0].node.clone())?;
            let value = to_expr(value.node)?;
            Ok(Statement::Assign(target, value))
        }
        StmtKind::For { target, iter, body, .. } => {
            let target = to_expr(target.node)?;
            let iter = to_expr(iter.node)?;
            Ok(Statement::For(target, iter, to_body(body)?))
        },
        other => Err(GPTParseError(format!("Cannot parse statement {:?}", other)))
    }
}

pub fn to_body(body: Vec<Located<StmtKind>>) -> Result<Body, GPTParseError> {
    let mut statements: Vec<Statement> = vec![];
    for statement in body {
        statements.push(to_statement(statement.node)?);
    } 
    Ok(statements)
}

pub fn parse_gptscript(code: &str) -> Result<Body, Box<dyn Error>> {
    let python_ast = parse_program(code, "smartgpt.gs")?;
    let body = to_body(python_ast)?;
    Ok(body)
}
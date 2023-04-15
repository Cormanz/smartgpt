use std::{error::Error, collections::HashMap};
use rustpython_parser::{parser, ast::{self, StmtKind, ExprKind, Constant, Expr, Located}};
use num_traits::ToPrimitive;

#[derive(Debug, Clone)]
pub enum Primitive {
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool)
}

#[derive(Debug, Clone)]
pub enum Expression {
    Name(String),
    Primitive(Primitive),
    Tuple(Vec<Expression>),
    List(Vec<Expression>),
    Dict(HashMap<String, Expression>)
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
        Expression::Primitive(Primitive::Text(value.clone()))
    }
}

pub fn to_expr(node: ExprKind) -> Option<Expression> {
    match node {
        ExprKind::Constant { value, .. } => {
            match value {
                Constant::Bool(bool) => Some(bool.into()),
                Constant::Int(int) => int.to_i64().map(|el| el.into()),
                Constant::Float(float) => float.to_f64().map(|el| el.into()),
                Constant::Str(text) => Some(text.into()),
                _ => None
            }
        }
        ExprKind::Name { id, .. } => {
            Some(Expression::Name(id.clone()))
        }
        ExprKind::List { elts, .. } => {
            let mut list: Vec<Expression> = vec![];
            for expr in elts {
                let expr = to_expr(expr.node)?;
                list.push(expr);
            }
            Some(Expression::List(list))
        }
        ExprKind::Dict { keys, values } => {
            let mut parsed_keys: Vec<String> = vec![];
            for expr in keys {
                let expr = to_expr(expr.node)?;
                let name = match expr {
                    Expression::Primitive(primitive) => {
                        match primitive {
                            Primitive::Text(text) => Some(text),
                            _ => None
                        }
                    },
                    _ => None
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

            Some(Expression::Dict(hash_map))
        },
        _ => None
    }
}

pub type Body = Vec<Statement>;

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Assign(Expression, Expression),
    For(Expression, Expression, Body)
}

pub fn to_statement(statement: StmtKind) -> Option<Statement> {
    match statement {
        StmtKind::Expr { value } => {
            to_expr(value.node).map(|el| Statement::Expression(el))
        }
        StmtKind::Assign { targets, value, type_comment } => {
            let target = to_expr(targets[0].node.clone())?;
            let value = to_expr(value.node)?;
            Some(Statement::Assign(target, value))
        }
        StmtKind::For { target, iter, body, .. } => {
            let target = to_expr(target.node)?;
            let iter = to_expr(iter.node)?;
            Some(Statement::For(target, iter, to_body(body)?))
        },
        _ => None
    }
}

pub fn to_body(body: Vec<Located<StmtKind>>) -> Option<Body> {
    let mut statements: Vec<Statement> = vec![];
    for statement in body {
        statements.push(to_statement(statement.node)?);
    } 
    Some(statements)
}

pub fn test_runner() -> Result<(), Box<dyn Error>> {
    let code = r#"x = 1
for [ x, y ] in map:
    2"#;

    let python_ast = parser::parse_program(code, "smartgpt.gs")?;
    println!("{:#?}", to_body(python_ast));

    Ok(())
}
use std::{fmt::Display, collections::HashMap, error::Error};

use serde::{Serialize, ser::{SerializeSeq, SerializeMap}, Deserialize, Deserializer};

fn gen_indent(indent: usize) -> String {
    " ".repeat(4 * indent)
}

pub trait Format {
    fn format(&self, in_command: bool, indent: usize) -> String;
}

pub enum Primitive {
    Text(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Primitive>),
    Map(HashMap<String, Primitive>)
}

impl Primitive {
    pub fn from_str(text: &str) -> Primitive {
        Primitive::Text(text.to_string())
    }
}

impl Serialize for Primitive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        
        match self {
            Primitive::Text(text) => {
                serializer.serialize_str(text)
            }
            Primitive::Integer(int) => {
                serializer.serialize_i64(*int)
            }
            Primitive::Float(float) => {
                serializer.serialize_f64(*float)
            }
            Primitive::Bool(bool) => {
                serializer.serialize_bool(*bool)
            }
            Primitive::List(data) => {
                let mut seq = serializer.serialize_seq(Some(data.len()))?;
                for element in data {
                    seq.serialize_element(element)?;
                }
                seq.end()
            }
            Primitive::Map(data) => {
                let mut map = serializer.serialize_map(Some(data.len()))?;
                for (key, value) in data {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Primitive {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PrimitiveVisitor;

        impl<'de> serde::de::Visitor<'de> for PrimitiveVisitor {
            type Value = Primitive;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a primitive value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(Primitive::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(Primitive::Integer(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_i64(value as i64)
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                Ok(Primitive::Float(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Primitive::Text(value.to_string()))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut values = HashMap::new();

                while let Some((key, value)) = map.next_entry::<String, Primitive>()? {
                    values.insert(key, value);
                }

                Ok(Primitive::Map(values))
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: serde::de::SeqAccess<'de>,
            {
                let mut values = Vec::new();

                while let Some(value) = seq.next_element::<Primitive>()? {
                    values.push(value);
                }

                Ok(Primitive::List(values))
            }
        }

        deserializer.deserialize_any(PrimitiveVisitor)
    }
}

impl Format for Primitive {
    fn format(&self, in_command: bool, indent: usize) -> String {
        match self {
            Primitive::Text(text) => {
                format!("{:?}", text)
            }
            Primitive::Integer(int) => {
                format!("{:?}", int)
            }
            Primitive::Float(float) => {
                format!("{:?}", float)
            }
            Primitive::Bool(bool) => {
                format!("{:?}", bool)
            }
            Primitive::List(list) => {
                let mut out = String::new();
                out.push_str("[ ");
                for (ind, item) in list.iter().enumerate() {
                    out.push_str(&item.format(in_command, indent));
                    if ind < list.len() - 1 {
                        out.push_str(", ");
                    }
                }
                out.push_str(" ]");
                out
            }
            Primitive::Map(map) => {
                let mut out = String::new();
                out.push_str("{ ");

                let mut sorted_keys: Vec<_> = map.keys().cloned().collect();
                sorted_keys.sort();
                
                for (ind, key) in sorted_keys.iter().enumerate() {
                    let value = map.get(key).unwrap();

                    out.push_str(key);
                    out.push_str(": ");
                    out.push_str(&value.format(in_command, indent));
                    if ind < map.len() - 1 {
                        out.push_str(" ");
                    }   
                }

                out.push_str(" }");
                out
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Expression {
    Command {
        name: String,
        args: Vec<Expression>
    },
    Lambda {
        args: Vec<String>,
        query: CommandQuery
    },
    Var(String),
    SetVar {
        var: String,
        expression: Box<Expression>
    },
    Primitive(Primitive)
}

impl Format for Expression {
    fn format(&self, in_command: bool, indent: usize) -> String {
        match self {
            Expression::Command { name, args } => {
                if args.len() == 0 {
                    return format!("{name}()");
                }

                let arg_str = args.iter()
                    .map(|arg| arg.format(true, indent))
                    .collect::<Vec<_>>()
                    .join(" ");

                if in_command {
                    format!("({name} {arg_str})")
                } else {
                    format!("{name} {arg_str}")
                }
            }
            Expression::Lambda { args, query } => {
                let arg_str = args.iter()
                    .map(|arg| format!("${arg}"))
                    .collect::<Vec<_>>()
                    .join(" ");

                let query_str = query.format(false, indent + 1);

                format!("|{arg_str}| {{\n{query_str}{}\n}}", gen_indent(indent))
            }
            Expression::Var(var) => {
                format!("${var}")
            }
            Expression::SetVar { var, expression } => {
                format!("${var} = {}", expression.format(in_command, indent))
            }
            Expression::Primitive(primitive) => {
                primitive.format(in_command, indent)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CommandQuery(pub Vec<Expression>);

impl Format for CommandQuery {
    fn format(&self, in_command: bool, indent: usize) -> String {
        self.0.iter()
            .map(|el| format!("{}{}", gen_indent(indent), el.format(in_command, indent)))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
use std::fmt::Display;

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
    List(Vec<Primitive>)
}

impl Primitive {
    pub fn from_str(text: &str) -> Primitive {
        Primitive::Text(text.to_string())
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
            Primitive::List(data) => {
                let mut out = String::new();
                out.push_str("[ ");
                for (ind, item) in data.iter().enumerate() {
                    out.push_str(&item.format(in_command, indent));
                    if ind < data.len() - 1 {
                        out.push_str(", ");
                    }
                }
                out.push_str(" ]");
                out
            }
        }
    }
}

pub enum Expression {
    Command {
        name: String,
        args: Vec<Expression>
    },
    Lambda {
        args: Vec<String>,
        query: Query
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

pub struct Query(pub Vec<Expression>);

impl Format for Query {
    fn format(&self, in_command: bool, indent: usize) -> String {
        self.0.iter()
            .map(|el| format!("{}{}", gen_indent(indent), el.format(in_command, indent)))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn test_runner() {
    let expression = Expression::Command {
        name: "for".to_string(),
        args: vec![
            Expression::Primitive(Primitive::List(vec![
                Primitive::Integer(3),
                Primitive::Integer(4),
                Primitive::Integer(6)
            ])),
            Expression::Lambda {
                args: vec![ "i".to_string() ],
                query: Query(vec![
                    Expression::Command {
                        name: "file-append".to_string(),
                        args: vec![
                            Expression::Primitive(Primitive::from_str("all.txt")),
                            Expression::Command {
                                name: "file-read".to_string(),
                                args: vec![
                                    Expression::Command {
                                        name: "concat".to_string(),
                                        args: vec![
                                            Expression::Primitive(Primitive::from_str("file-")),
                                            Expression::Var("i".to_string()),
                                            Expression::Primitive(Primitive::from_str(".txt"))
                                        ]
                                    },
                                ]
                ,           },
                        ]
                    }
                ])
            }
        ]
    };

    println!("{}", expression.format(false, 0));
}
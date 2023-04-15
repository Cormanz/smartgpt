mod parse;
mod run;

use std::{error::Error, collections::HashMap};

pub use parse::*;
pub use run::*;

pub fn test_runner() -> Result<(), Box<dyn Error>> {
    let expression = CommandQuery(vec![
        Expression::SetVar {
            var: "map".to_string(),
            expression: Box::new(Expression::Primitive(Primitive::Map(HashMap::from_iter(vec![
                ("x".to_string(), Primitive::Integer(1)),
                ("y".to_string(), Primitive::Integer(2))
            ]))))
        },
        Expression::Command {
            name: "for".to_string(),
            args: vec![
                Expression::Primitive(Primitive::List(vec![
                    Primitive::Integer(3),
                    Primitive::Integer(4),
                    Primitive::Integer(6)
                ])),
                Expression::Lambda {
                    args: vec![ "i".to_string() ],
                    query: CommandQuery(vec![
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
        }
    ]);

    println!("{}", expression.format(false, 0));
    println!("{}", serde_json::to_string_pretty(&expression)?);

    Ok(())
}
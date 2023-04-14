use std::{collections::HashMap, error::Error, fmt::Display, process};

use async_trait::async_trait;

use crate::{Plugin, Command, CommandContext, CommandImpl, PluginCycle, EmptyCycle};
use std::fs;

#[derive(Debug, Clone)]
pub struct ShutdownNoOutputError;

impl Display for ShutdownNoOutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "'shutdown' did not receive an output.")
    }
}

impl Error for ShutdownNoOutputError {}

pub async fn shutdown(ctx: &mut CommandContext, args: HashMap<String, String>) -> Result<String, Box<dyn Error>> {
    let output = args.get("output").ok_or(ShutdownNoOutputError)?;

    fs::write(format!("files/out"), output)?;

    process::exit(1);
}

pub struct Shutodwn;

#[async_trait]
impl CommandImpl for Shutodwn {
    async fn invoke(&self, ctx: &mut CommandContext, args: HashMap<String, String>) -> Result<String, Box<dyn Error>> {
        shutdown(ctx, args).await
    }
}

pub fn create_shutdown() -> Plugin {
    Plugin {
        name: "Shutdown".to_string(),
        dependencies: vec![],
        cycle: Box::new(EmptyCycle),
        commands: vec![
            Command {
                name: "shutdown".to_string(),
                purpose: "Shutdown the program with the output.".to_string(),
                args: vec![
                    ("output".to_string(), "The output that the program ends with".to_string())
                ],
                run: Box::new(Shutodwn)
            }
        ]
    }
}
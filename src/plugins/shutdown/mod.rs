use std::{collections::HashMap, error::Error, fmt::Display, process};

use async_trait::async_trait;

use crate::{Plugin, Command, CommandContext, CommandImpl, PluginCycle, EmptyCycle, ScriptValue};
use std::fs;

#[derive(Debug, Clone)]
pub struct ShutdownNoOutputError;

impl Display for ShutdownNoOutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "'shutdown' did not receive an output.")
    }
}

impl Error for ShutdownNoOutputError {}

pub async fn shutdown(ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
    let output: String = args.get(0).ok_or(ShutdownNoOutputError)?.clone().try_into()?;

    fs::write(format!("files/out"), output)?;

    process::exit(1);
}

pub struct Shutodwn;

#[async_trait]
impl CommandImpl for Shutodwn {
    async fn invoke(&self, ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
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
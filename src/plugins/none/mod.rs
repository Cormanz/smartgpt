use std::{collections::HashMap, error::Error, fmt::Display};

use async_trait::async_trait;

use crate::{Plugin, Command, CommandContext, CommandImpl, PluginCycle, EmptyCycle};
use std::fs;


pub async fn none(ctx: &mut CommandContext, args: HashMap<String, String>) -> Result<String, Box<dyn Error>> {
    Ok(format!("Skipped command."))
}

pub struct NoneImpl;

#[async_trait]
impl CommandImpl for NoneImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: HashMap<String, String>) -> Result<String, Box<dyn Error>> {
        none(ctx, args).await
    }
}

pub fn create_none() -> Plugin {
    Plugin {
        name: "None".to_string(),
        dependencies: vec![],
        cycle: Box::new(EmptyCycle),
        commands: vec![
            Command {
                name: "none".to_string(),
                purpose: "Do nothing.".to_string(),
                args: vec![],
                run: Box::new(NoneImpl)
            }
        ]
    }
}
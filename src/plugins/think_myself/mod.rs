use std::{collections::HashMap, error::Error, fmt::Display};

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::{Plugin, Command, CommandContext, CommandImpl, PluginCycle, EmptyCycle, ScriptValue, CommandResult};
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct SelfThoughts {
    solution: String
}

pub async fn think_myself(ctx: &mut CommandContext, args: ScriptValue) -> Result<ScriptValue, Box<dyn Error>> {
    Ok(ScriptValue::None)
}

pub struct NoneImpl;

#[async_trait]
impl CommandImpl for NoneImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: ScriptValue) -> Result<CommandResult, Box<dyn Error>> {
        Ok(CommandResult::ScriptValue(think_myself(ctx, args).await?))
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub fn create_none() -> Plugin {
    Plugin {
        name: "None".to_string(),
        dependencies: vec![],
        cycle: Box::new(EmptyCycle),
        commands: vec![
            Command {
                name: "think_myself".to_string(),
                purpose: "Think myself.".to_string(),
                args: vec![],
                return_type: "None".to_string(),
                run: Box::new(NoneImpl)
            }
        ]
    }
}
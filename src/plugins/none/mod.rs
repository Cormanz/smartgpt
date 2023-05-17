use std::error::Error;

use async_trait::async_trait;

use crate::{Command, CommandContext, CommandImpl, EmptyCycle, Plugin, ScriptValue};

pub async fn none(
    _ctx: &mut CommandContext,
    _args: Vec<ScriptValue>,
) -> Result<ScriptValue, Box<dyn Error>> {
    Ok(ScriptValue::None)
}

pub struct NoneImpl;

#[async_trait]
impl CommandImpl for NoneImpl {
    async fn invoke(
        &self,
        ctx: &mut CommandContext,
        args: Vec<ScriptValue>,
    ) -> Result<ScriptValue, Box<dyn Error>> {
        none(ctx, args).await
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
        commands: vec![Command {
            name: "none".to_string(),
            purpose: "Do nothing.".to_string(),
            args: vec![],
            return_type: "None".to_string(),
            run: Box::new(NoneImpl),
        }],
    }
}

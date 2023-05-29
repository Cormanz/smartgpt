use std::{error::Error};

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::{Plugin, Tool, CommandContext, CommandImpl, EmptyCycle, ScriptValue, CommandResult, ToolArgument, ToolType};

#[derive(Serialize, Deserialize)]
pub struct BrainstormArgs {
    pub lines: Vec<String>
}

pub async fn brainstorm() -> Result<ScriptValue, Box<dyn Error>> {
    Ok(ScriptValue::None)
}

pub struct BrainstormImpl;

#[async_trait]
impl CommandImpl for BrainstormImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: ScriptValue) -> Result<CommandResult, Box<dyn Error>> {
        Ok(CommandResult::ScriptValue(brainstorm().await?))
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub fn create_brainstorm() -> Plugin {
    Plugin {
        name: "Brainstorm".to_string(),
        dependencies: vec![],
        cycle: Box::new(EmptyCycle),
        tools: vec![
            Tool {
                name: "brainstorm".to_string(),
                purpose: "Think of an idea or generate content manually.".to_string(),
                args: vec![
                    ToolArgument::new("lines", r#"[ "line 1", "line 2" ]"#)
                ],
                run: Box::new(BrainstormImpl),
                tool_type: ToolType::Resource
            }
        ]
    }
}
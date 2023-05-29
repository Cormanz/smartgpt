use std::{error::Error, collections::HashMap};

use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{ScriptValue, Tool, CommandContext, Expression, GPTRunError, CommandResult};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Action {
    pub tool: String,
    pub args: Option<ScriptValue>
}

pub async fn run_tool(
    out: &mut String,
    name: String, tool: Tool, 
    context: &mut CommandContext, args: ScriptValue
) -> Result<CommandResult, Box<dyn Error>> {
    let result = tool.run.invoke(context, args.clone()).await?;
    let _args: Expression = args.clone().into();

    let json = match &result {
        CommandResult::Text(string) => Ok(string.clone()),
        CommandResult::ScriptValue(value) => serde_yaml::to_string(value)
    }
        .map_err(|_| GPTRunError("Could not parse ScriptValue as JSON.".to_string()))?;

    let text = format!("Tool use {name} returned:\n{}", json);
    out.push_str(&text);

    Ok(result)
}

pub fn run_action_sync(context: &mut CommandContext, action: Action) -> Result<String, Box<dyn Error>> {
    let tool = context.plugins.iter()
        .flat_map(|el| &el.tools)
        .find(|el| el.name == action.tool)
        .map(|el| el.box_clone());

    let mut out = String::new();
    match tool {
        Some(tool) => {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                run_tool(
                    &mut out, 
                    action.tool.clone(), 
                    tool.box_clone(), 
                    context, 
                    action.args.unwrap_or(HashMap::new().into())
                ).await
            })?;

        },
        None => {
            let error_str = format!("Error: No such tool named '{}'.", action.tool.clone());
            out.push_str(&error_str)
        }
    }

    Ok(out)
}
use std::{sync::{Mutex, Arc}, error::Error};

use crate::{ScriptValue, ProgramInfo, Command, CommandContext, Expression, GPTRunError};

pub async fn run_command(
    out: &mut String,
    name: String, command: Command, 
    context: &mut CommandContext, args: ScriptValue
) -> Result<ScriptValue, Box<dyn Error>> {
    let result = command.run.invoke(context, args.clone()).await?;
    let args: Expression = args.clone().into();

    let json = serde_yaml::to_string(&result)
        .map_err(|_| GPTRunError("Could not parse ScriptValue as JSON.".to_string()))?;

    let text = format!("Tool use {name} {:?} returned:\n{}", args, json);
    out.push_str(&text);

    Ok(result)
}
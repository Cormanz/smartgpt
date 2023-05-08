use std::{sync::{Mutex, Arc}, error::Error};

use crate::{ScriptValue, ProgramInfo, Command, CommandContext, Expression, GPTRunError};

pub async fn run_command(
    out: &mut String,
    name: String, command: Command, 
    context: &mut CommandContext, args: Vec<ScriptValue>
) -> Result<ScriptValue, Box<dyn Error>> {
    let result = command.run.invoke(context, args.clone()).await?;

    let args: Vec<Expression> = args.iter().map(|el| el.clone().into()).collect();
    let expr = Expression::FunctionCall(name.clone(), args);

    let json = serde_json::to_string(&result)
        .map_err(|_| GPTRunError("Could not parse ScriptValue as JSON.".to_string()))?;

    let text = format!("Command {:?} returned:\n{}", expr, json);
    out.push_str(&text);
    println!("{}", text);

    Ok(result)
}
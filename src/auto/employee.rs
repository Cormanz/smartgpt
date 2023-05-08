use std::{error::Error, sync::{Arc, Mutex}, fmt::Display, ascii::AsciiExt};

use colored::Colorize;
use mlua::{Value, Variadic, Lua, Result as LuaResult, FromLua, ToLua, Error as LuaError};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{ProgramInfo, generate_commands, Message, Agents, ScriptValue, GPTRunError, Expression, Command, CommandContext, auto::try_parse_json, LLM};

use super::{run::run_command, ParsedResponse};

#[derive(Debug, Clone)]
pub struct EmployeeError(pub String);

impl Display for EmployeeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MinionError: {}", self.0)
    }
}

impl Error for EmployeeError {}

#[derive(Serialize, Deserialize)]
pub struct EmployeeAction {
    pub command: String,
    pub args: Option<Vec<ScriptValue>>
}

#[derive(Serialize, Deserialize)]
pub struct EmployeeThought {
    #[serde(rename = "previous command success")]
    previous_success: Option<bool>,
    thoughts: String,
    reasoning: String,
    #[serde(rename = "long term plan")]
    plan: String,
    action: EmployeeAction
}

pub fn run_employee<T>(program: &mut ProgramInfo, task: &str, end: impl Fn(&mut LLM) -> T) -> Result<T, Box<dyn Error>> {
    let ProgramInfo { 
        context, plugins, personality,
        disabled_commands, .. 
    } = program;
    let mut context = context.lock().unwrap();

    let cmds = generate_commands(plugins, disabled_commands);

    context.agents.employee.llm.prompt.clear();
    context.agents.employee.llm.message_history.clear();

    context.agents.employee.llm.prompt.push(Message::System(format!(
r#"
Personality: {personality}

You will be given one task.
Your goal is to complete that task, one command at a time.
Do it as fast as possible.
"#
    )));

    context.agents.employee.llm.end_prompt.push(Message::User(format!(
"You have access to these commands:
{}
    finish() -> None
        A special command. Use this command when you are done with your assignment.",
        cmds
    )));
    
    context.agents.employee.llm.prompt.push(Message::User(format!(r#"
Your task is: {task}
"#,
    )));

    context.agents.employee.llm.prompt.push(Message::User(format!(r#"
Reply in this format:

```json
{{
    "previous command success": true / false / null,
    "thoughts": "...",
    "reasoning": "...",
    "long term plan": "...",
    "action": {{
        "command": "...",
        "args": [
            "..."
        ]
    }}
}}
```

Reply in that exact JSON format exactly.
Make sure every field is filled in detail.
Keep every field in that exact order.
"#)));

    context.agents.employee.llm.message_history.push(Message::User(format!(
        r#"Please run your next command. If you are done, keep your 'command name' field as 'finish'"#
    )));

    let dashes = "--".white();
    let employee = "Employee".blue();

    loop {
        let thoughts = try_parse_json::<EmployeeThought>(&context.agents.employee.llm, 2, Some(400))?;
        let ParsedResponse { data: thoughts, raw } = thoughts;

        println!("{dashes} {employee} {dashes}");
        println!();
        println!("{}", serde_yaml::to_string(&thoughts)?);
        println!();

        let command_name = thoughts.action.command.clone();
        let args = thoughts.action.args.clone().unwrap_or(vec![]);

        if command_name == "finish" {
            break;
        }

        let command = plugins.iter()
            .flat_map(|el| &el.commands)
            .find(|el| el.name == command_name)
            .ok_or(EmployeeError(format!("Cannot find command {}", command_name)))?;

        let mut out = String::new();
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            run_command(
                &mut out, 
                command_name.clone(), 
                command.box_clone(), 
                &mut context, 
                args
            ).await
        })?;

        context.agents.employee.llm.message_history.push(Message::Assistant(raw));
        context.agents.employee.llm.message_history.push(Message::User(out));
        context.agents.employee.llm.message_history.push(Message::User(format!(
            r#"Please run your next command. If you are done, keep your 'command name' field as 'finish'"#
        )));
    }

    Ok(end(&mut context.agents.employee.llm))
}

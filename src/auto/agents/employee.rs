use std::{error::Error, sync::{Arc, Mutex}, fmt::Display, ascii::AsciiExt};

use colored::Colorize;
use mlua::{Value, Variadic, Lua, Result as LuaResult, FromLua, ToLua, Error as LuaError};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{ProgramInfo, generate_commands, Message, Agents, ScriptValue, GPTRunError, Expression, Command, CommandContext, auto::{try_parse_json, ParsedResponse, run::run_command, agents::findings::to_points}, LLM, AgentInfo, Weights};

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

pub fn run_employee<T>(program: &mut ProgramInfo, task: &str, end: impl Fn(&mut AgentInfo) -> T) -> Result<T, Box<dyn Error>> {
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

    let AgentInfo { observations, llm, .. } = &mut context.agents.employee;
    let observations = observations.get_memories_sync(
        &llm,
        task,
        200,
        Weights {
            recall: 1.,
            recency: 1.,
            relevance: 1.
        },
        50
    )?;
    let observations = if observations.len() == 0 {
        None
    } else {
        Some(observations.iter().enumerate()
            .map(|(ind, observation)| format!("{ind}. {}", observation.content))
            .collect::<Vec<_>>()
            .join("\n"))
    };
    
    context.agents.employee.llm.prompt.push(Message::User(format!(
"You have access to these resources as commands:
{}
    finish() -> None
        A special command. Use this command when you are done with your assignment.
        
You may only use these commands.
These are the only commands available. Do not use any other commands.",
        cmds
    )));

    if let Some(observations) = observations {
        context.agents.employee.llm.prompt.push(Message::User(format!(
"Here are your long-term memories:

{observations}"
        )))
    }

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

Focus on reasoning regarding your commands. 
Try to break down your problems in terms of what commands can be used.

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

        println!();
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
            .find(|el| el.name == command_name);

        let mut out = String::new();
        match command {
            Some(command) => {
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
        
            },
            None => {
                out.push_str(&format!(
"No such command named '{command_name}.' 
These are your commands: {}",
    to_points(&plugins.iter().flat_map(|el| &el.commands).map(|el| el.name.clone()).collect::<Vec<_>>())))
            }
        }

        context.agents.employee.llm.message_history.push(Message::Assistant(raw));
        context.agents.employee.llm.message_history.push(Message::User(out));
        context.agents.employee.llm.message_history.push(Message::User(format!(
            r#"Please run your next command. If you are done, set the 'command' field to 'finish'"#
        )));
    }

    Ok(end(&mut context.agents.employee))
}

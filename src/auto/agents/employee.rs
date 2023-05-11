use std::{error::Error, sync::{Arc, Mutex}, fmt::Display, ascii::AsciiExt};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{ProgramInfo, generate_commands, Message, Agents, ScriptValue, GPTRunError, Expression, Command, CommandContext, auto::{try_parse_json, ParsedResponse, run::run_command, agents::findings::{to_points, ask_for_findings}}, LLM, AgentInfo, Weights, generate_commands_short};

use super::findings::get_observations;

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
    #[serde(rename = "am I done")]
    done: bool,
    thoughts: String,
    reasoning: String,
    criticism: String,
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
    let cmds_short = generate_commands_short(plugins, disabled_commands);

    context.agents.employee.llm.prompt.clear();
    context.agents.employee.llm.message_history.clear();

    context.agents.employee.llm.prompt.push(Message::System(format!(
r#"
Personality: {personality}

Remember that you are a large language model. Play to your strengths.

You will be given one task.
Try to work that task out, step by step, one command at a time.
Complete this in as minimal tasks as possible.
"#
    )));

    let observations = get_observations(&mut context.agents.employee, task)?
        .unwrap_or("None found.".to_string());
    
    context.agents.employee.llm.prompt.push(Message::User(format!(
"You have access to these resources as commands:
{}
    finish() -> None
        A special command. Use this command when you are done with your assignment.
        
You may only use these commands.
These are the only commands available. Do not use any other commands.",
        cmds
    )));

    context.agents.employee.llm.prompt.push(Message::User(format!(
"Here are your long-term memories:

{observations}"
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
    "am I done": true / false / null,
    "thoughts": "...",
    "reasoning": "...",
    "criticism": "...",
    "plan": "...",
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
        let thoughts = try_parse_json::<EmployeeThought>(&context.agents.employee.llm, 2, Some(1000))?;
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
                let error_str = format!(
"No such command named '{command_name}.' 
These are your commands: {cmds_short}");
                out.push_str(&error_str)
            }
        }

        context.agents.employee.llm.message_history.push(Message::Assistant(raw));
        context.agents.employee.llm.message_history.push(Message::User(out));
        context.agents.employee.llm.message_history.push(Message::User(format!(
            r#"Decide whether or not you are done. If done, use the 'finish' command. Otherwise, proceed onto your next command. Ensure your response is the exact JSON format, no plaintext."#
        )));

        let remaining_tokens = context.agents.employee.llm.get_tokens_remaining(
            &context.agents.employee.llm.get_messages()
        )?;

        if remaining_tokens < 1450 {
            ask_for_findings(&mut context.agents.employee)?;
            context.agents.employee.llm.crop_to_tokens_remaining(2600)?;

            let observations = get_observations(&mut context.agents.employee, task)?
                .unwrap_or("None found.".to_string());
            context.agents.employee.llm.prompt[2].set_content(&format!(
"Here are your long-term memories:

{observations}"
            ));
        }
    }

    Ok(end(&mut context.agents.employee))
}

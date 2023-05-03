use std::{error::Error, sync::{Arc, Mutex}, fmt::Display, ascii::AsciiExt};

use colored::Colorize;
use mlua::{Value, Variadic, Lua, Result as LuaResult, FromLua, ToLua, Error as LuaError};
use serde::{Deserialize, Serialize, __private::de};

use crate::{ProgramInfo, generate_commands, Message, Agents, ScriptValue, GPTRunError, Expression, Command, CommandContext, agents::{process_response, LINE_WRAP, minion::{create_letter, MinionError, run_script}, create_findings_prompt, run_command, try_parse_json}};

use super::{super::try_parse_yaml, MinionResponse};

#[derive(Serialize, Deserialize)]
pub struct MinionUpdate {
    #[serde(rename = "command name")] command: String,
    args: Option<Vec<ScriptValue>>
}

pub fn run_continuous_minion(
    program: &mut ProgramInfo, task: &str, new_prompt: bool
) -> Result<(String, MinionResponse), Box<dyn Error>> {
    let ProgramInfo { 
        context, plugins, personality,
        disabled_commands, .. 
    } = program;
    let mut context = context.lock().unwrap();

    let cmds = generate_commands(plugins, disabled_commands);

    context.agents.minion.llm.prompt.clear();
    context.agents.minion.llm.message_history.clear();

    context.agents.minion.llm.prompt.push(Message::System(format!(
r#"
Run one command at a time to complete your task:
{}

When running a command, reply in this format exactly:

```json
{{
    "progress towards task": null / "So far...",
    "remaining work to complete task": "I still must... / I am done.",
    "reasoning": "As...",
    "idea": "I should...",
    "command name": "Command Name",
    "args": [
      "A",
      "B"
    ]
}}
```

Use this exact format exactly, including all fields.

Escape all quotes inside of your arguments.

Only respond with the command. NOTHING ELSE!
Only use ONE COMMAND at a time.
Always use a command.
"#,
        task
    )));

    context.agents.minion.llm.end_prompt.push(Message::User(format!(
"Reminder that these are your commands:
{}
    done() -> AssignmentIsDone
        A special command. Use this command when you are done with your assignment.

You can only use those commands.
ONLY USE THOSE COMMANDS.

Please complete your task succintly, one command at a time.
Ensure your response is valid JSON.",
        cmds
    )));

    context.agents.minion.llm.message_history.push(Message::User(format!(
        r#"Please run your next command. If you are done, set your 'command name' to "done""#
    )));
        
    drop(context);

    loop {        
        let mut is_done = false;

        for i in 0..3 {
            let mut messages: Vec<Message> = vec![];

            let ProgramInfo { 
                context, ..
            } = program;
            let mut context = context.lock().unwrap();

            let (response, update) = try_parse_json::<MinionUpdate>(&context.agents.minion.llm, 3, Some(1000))?;

            let processed_script = process_response(&response, LINE_WRAP);
            messages.push(Message::Assistant(response.clone()));
        
            println!("{}", "MINION".blue());
            println!("{}", format!("The minion has written a command.").white());
            println!();
            println!("{processed_script}");
            println!();

            if update.command.to_ascii_lowercase().trim() == "done" {
                is_done = true;
                break;
            }

            let command = plugins.iter()
                .flat_map(|el| &el.commands)
                .find(|el| el.name == update.command)
                .ok_or(MinionError(format!("Cannot find command {}", update.command)));

            let command = match command {
                Err(err) => {
                    println!("{}", "MINION".blue());
                    println!("{}", 
                        format!("The minion errored on attempt {}. Trying again.", i + 1)
                            .white()
                    );
                    println!();
                    println!("{err}");
                    println!();
                    continue;
                },
                Ok(cmd) => cmd
            };

            let mut out = String::new();
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                run_command(
                    &mut out, 
                    command.name.clone(), 
                    command.box_clone(), 
                    &mut context, 
                    update.args.unwrap_or(vec![])
                ).await
            });

            drop(context);

            match result {
                Err(err) => {
                    println!("{}", "MINION".blue());
                    println!("{}", 
                        format!("The minion errored on attempt {}. Trying again.", i + 1)
                            .white()
                    );
                    println!();
                    println!("{err}");
                    println!();
                    continue;
                },
                Ok(_) => {}
            };

            let ProgramInfo { 
                context, ..
            } = program;
            let mut context = context.lock().unwrap();

            messages.push(Message::User(out));  

            context.agents.minion.llm.crop_to_tokens_remaining(1800);

            messages.push(Message::User(format!(
                r#"Please run your next command. If you are done, use the "done" command."#
            )));

            context.agents.minion.llm.message_history.extend(messages.clone());
        }

        if is_done {
            break;
        }
    }

    let ProgramInfo { 
        context, ..
    } = program;
    let mut context = context.lock().unwrap();

    context.agents.minion.llm.message_history.push(Message::User(create_findings_prompt()));
    
    let (response, decision) = try_parse_json::<MinionResponse>(&context.agents.minion.llm, 3, Some(1000))?;
    context.agents.minion.llm.message_history.push(Message::Assistant(response.clone()));

    let letter = create_letter(&decision.findings, &decision.changes);

    Ok((letter, decision))
}
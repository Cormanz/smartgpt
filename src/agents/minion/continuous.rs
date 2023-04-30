use std::{error::Error, sync::{Arc, Mutex}, fmt::Display, ascii::AsciiExt};

use colored::Colorize;
use mlua::{Value, Variadic, Lua, Result as LuaResult, FromLua, ToLua, Error as LuaError};
use serde::{Deserialize, Serialize, __private::de};

use crate::{ProgramInfo, generate_commands, Message, Agents, ScriptValue, GPTRunError, Expression, Command, CommandContext, agents::{process_response, LINE_WRAP, minion::{create_letter, MinionError, run_script}, create_findings_prompt}};

use super::{super::try_parse, MinionResponse};

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

This is psuedocode. You will need to run this one command at a time.
When running a command, reply in this format exactly:

command_name(STRING, NUMBER...)

Escape all quotes inside of your arguments.
Your arguments cannot be other commands. Only primitive datatypes.

Only respond with the command. NOTHING ELSE!
Only use ONE COMMAND at a time.
"#,
        task
    )));

    context.agents.minion.llm.end_prompt.push(Message::User(format!(
"Reminder that these are your commands:
{}

You can only use those commands.
ONLY USE THOSE COMMANDS.

Please complete your task, one command at a time.",
        cmds
    )));

    context.agents.minion.llm.message_history.push(Message::User(format!(
        r#"Please run your next command. If you are done, reply with "Done" exactly."#
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

            let script = context.agents.minion.llm.model.get_response_sync(
                &context.agents.minion.llm.get_messages(),
                Some(1000),
                Some(0.3)
            )?;

            let processed_script = process_response(&script, LINE_WRAP);
            messages.push(Message::Assistant(script.clone()));
        
            println!("{}", "MINION".blue());
            println!("{}", format!("The minion has written a command.").white());
            println!();
            println!("{processed_script}");
            println!();

            let lower = script.trim().to_ascii_lowercase();
            if vec![ "done.", "done" ].contains(&lower.trim()) {
                is_done = true;
                break;
            }

            drop(context);

            let out = run_script(program, &script, &Lua::new());
            let out = match out {
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
                Ok(out) => out
            };

            let ProgramInfo { 
                context, ..
            } = program;
            let mut context = context.lock().unwrap();

            messages.push(Message::User(out));  

            context.agents.minion.llm.crop_to_tokens_remaining(1800);

            messages.push(Message::User(format!(
                r#"Please run your next command. If you are done, reply with "DONE""#
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
    
    let (response, decision) = try_parse::<MinionResponse>(&context.agents.minion.llm, 3, Some(1000))?;
    context.agents.minion.llm.message_history.push(Message::Assistant(response.clone()));

    let letter = create_letter(&decision.findings, &decision.changes);

    Ok((letter, decision))
}
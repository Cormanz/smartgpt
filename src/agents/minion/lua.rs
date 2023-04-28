use std::{error::Error, sync::{Arc, Mutex}, fmt::Display};

use colored::Colorize;
use mlua::{Value, Variadic, Lua, Result as LuaResult, FromLua, ToLua, Error as LuaError};
use serde::{Deserialize, Serialize, __private::de};

use crate::{ProgramInfo, generate_commands, Message, Agents, ScriptValue, GPTRunError, Expression, Command, CommandContext, agents::{process_response, LINE_WRAP, minion::{create_letter, MinionError, run_script}, create_findings_prompt}};

use super::{super::try_parse, MinionResponse};

pub fn run_lua_minion(
    program: &mut ProgramInfo, task: &str, new_prompt: bool
) -> Result<(String, MinionResponse), Box<dyn Error>> {
    let mut last_err: Result<String, Box<dyn Error>> = Ok("".to_string());
    for i in 0..3 {
        let ProgramInfo { 
            context, plugins, personality,
            disabled_commands, .. 
        } = program;
        let mut context = context.lock().unwrap();

        let cmds = generate_commands(plugins, disabled_commands);
    
        if i == 0 {
            context.agents.minion.llm.prompt.clear();
            context.agents.minion.llm.message_history.clear();

            context.agents.minion.llm.prompt.push(Message::System(format!(
        r#"
Using these commands and ONLY these commands:
{}

Write a script to complete this task:
{}

Use the exact commands mentioned in the task.
ONLY USE THOSE COMMANDS.
Do not use any other commands.

Keep it as SIMPLE, MINIMAL, and SHORT as possible. IT MUST BE VERY SIMPLE AND SHORT.
Pay very close attention to the TYPE of each command.
Whenever you save a file, use ".txt" for the extension.

Your script will be in the LUA Scripting Language. LUA.
        "#,
                cmds, task
            )));

            println!("{}", context.agents.minion.llm.prompt.last()
                .ok_or(MinionError("Could not get last element.".to_string()))?
            );
        }
    
        let script = context.agents.minion.llm.model.get_response_sync(
            &context.agents.minion.llm.get_messages(),
            Some(300),
            Some(0.3)
        )?;
    
        let processed_script = process_response(&script, LINE_WRAP);
    
        println!("{}", "MINION".blue());
        println!("{}", format!("The minion has created a script. Attempt {}", i + 1).white());
        println!();
        println!("{processed_script}");
        println!();

        drop(context);
        let out = run_script(program, &script, &Lua::new());

        let ProgramInfo { 
            context, ..
        } = program;
        let mut context = context.lock().unwrap();
        
        last_err = match &out {
            Ok(_) => {
                break;
            }
            Err(err) => {
                context.agents.employee.llm.message_history.push(Message::Assistant(script.clone()));

                context.agents.minion.llm.message_history.push(Message::User(format!(
"Unfortunately, when running your code, it did not work.
The error was: {err}\n
Explain the reasoning for this error, and suggest a potential fix (BUT DO NOT IMPLEMENT IT.)"
                )));

                let explanation = context.agents.minion.llm.model.get_response_sync(
                    &context.agents.minion.llm.get_messages(),
                    Some(300),
                    Some(0.3)
                )?;
                
                context.agents.employee.llm.message_history.push(Message::Assistant(explanation.clone()));

                println!("{}", "MINION".blue());
                println!("{}", format!("The minion has created an explanation of what went wrong.").white());
                println!();
                println!("{explanation}");
                println!();
                println!("{}", format!("The error was:").white());
                println!();
                println!("{err}");
                println!();

                context.agents.minion.llm.message_history.push(Message::User(format!(
"Please try again in the exact same format with a fixed LUA script
Respond ONLY with a LUA script. 
Do not provide any additional code commentary. Only reply with LUA code.
Ensure your response is exactly valid LUA and can be parsed as valid LUA."
)));
                out
            }
        };
        
        drop(context);
    }

    match last_err {
        Err(err) => {
            Err(err)
        }
        Ok(result) => {
            println!("{}", "MINION".blue());
            println!("{}", format!("The minion has ran the script.").white());
            println!();

            let ProgramInfo { 
                context, plugins, personality,
                disabled_commands, .. 
            } = program;
            let mut context = context.lock().unwrap();

            context.agents.minion.llm.message_history.push(Message::System(create_findings_prompt()));
            context.agents.minion.llm.message_history.push(Message::User(result));
            
            let (response, decision) = try_parse::<MinionResponse>(&context.agents.employee.llm, 3, Some(1000))?;
            context.agents.employee.llm.message_history.push(Message::Assistant(response.clone()));

            let letter = create_letter(&decision.findings, &decision.changes);

            Ok((letter, decision))
        }
    }
}
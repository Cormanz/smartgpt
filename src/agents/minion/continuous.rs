use std::{error::Error, sync::{Arc, Mutex}, fmt::Display};

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
Using these commands and ONLY these commands:
{}

Run one command at a time to complete this task:
{}

This is psuedocode. You will need to run this one command at a time.

Format your commands like so: command(ARG1, ARG2...)
Escape all quotes inside of your arguments.
Only respond with the command. NOTHING ELSE!
Only use ONE COMMAND at a time.

If you are done, reply with "DONE"
"#,
        cmds, task
    )));

    drop(context);

    loop {        
        let ProgramInfo { 
            context, ..
        } = program;
        let mut context = context.lock().unwrap();

        let script = context.agents.minion.llm.model.get_response_sync(
            &context.agents.minion.llm.get_messages(),
            Some(300),
            Some(0.3)
        )?;

        let processed_script = process_response(&script, LINE_WRAP);
    
        println!("{}", "MINION".blue());
        println!("{}", format!("The minion has ran a command.").white());
        println!();
        println!("{processed_script}");
        println!();

        if script == "DONE" {
            break;
        }

        drop(context);

        let out = run_script(program, &script, &Lua::new())?;

        let ProgramInfo { 
            context, ..
        } = program;
        let mut context = context.lock().unwrap();

        context.agents.minion.llm.prompt.push(Message::User(format!(
"{out}\nProceed to your next command."
        )));  
    }

    let ProgramInfo { 
        context, ..
    } = program;
    let mut context = context.lock().unwrap();

    context.agents.minion.llm.message_history.push(Message::User(create_findings_prompt()));
    
    let (response, decision) = try_parse::<MinionResponse>(&context.agents.employee.llm, 3, Some(1000))?;
    context.agents.employee.llm.message_history.push(Message::Assistant(response.clone()));

    let letter = create_letter(&decision.findings, &decision.changes);

    Ok((letter, decision))
}
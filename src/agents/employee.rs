use std::error::Error;
use crate::{prompt::generate_commands, ProgramInfo, AgentLLMs, Agents, Message, agents::{process_response, LINE_WRAP, Choice, try_parse, CannotParseError, minion::run_minion}};
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EmployeeDecision {
    pub choice: String,
    pub report: Option<String>,
    #[serde(rename = "command query")] pub command_query: Option<String>
}

pub fn run_employee(
    program: &mut ProgramInfo, task: &str, new_prompt: bool
) -> Result<String, Box<dyn Error>> {
    let ProgramInfo { context, plugins, personality, disabled_commands, .. } = program;
    let mut context = context.lock().unwrap();

    context.agents.employee.prompt.push(Message::System(format!(
        "You are The Employee, a large language model. 

Personality: {}

Your goal is take advantage of access to commands to provide answers to questions.

You have a minion named The Minion, who will turn your request into a script and run it.
You will turn the Boss's request into simple, tiny psuedocode.

You have been given one task from The Boss.",
        personality
    )));

    let commands = plugins.iter()
        .flat_map(|el| el.commands.iter())
        .map(|el| el.name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    let prompt = format!(r#"
You have access to these commands:
{}

Your task is {:?}

Write a psuedocode for the task.
Keep your psuedocode as SHORT and SIMPLE as possible.
Include the EXACT COMMANDS in your psuedocode.
ONLY USE COMMANDS, NOTHING ELSE.

Whenever you want to do some sort of task involving writing, creating/processing text, use the `ask_chatgpt` command.

Avoid any DATA EXTRACTION in your psuedocode.
- Don't try to extract information from an article. Leave the article content as-is."#,
        commands, task
    );

    context.agents.employee.prompt.push(Message::User(prompt));

    let response = context.agents.employee.model.get_response_sync(&context.agents.employee.get_messages(), None, None)?;
    context.agents.employee.message_history.push(Message::Assistant(response.clone()));

    let task_list = process_response(&response, LINE_WRAP);

    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has created some psuedocode to achieve its goal.".white());
    println!();
    println!("{task_list}");
    println!();

    drop(context);
    let out = run_minion(program, task, new_prompt)?;

    let ProgramInfo { context, plugins, personality, disabled_commands, .. } = program;
    let mut context = context.lock().unwrap();

    let prompt = format!(r#"Your query gave the following output:
    
{out}

Please write a response to The Boss with your findings. 
Only discuss RESULTS. 

Provide very specific information.
What exact information was saved? What was the name of the file? What sources were they?
Be specific.

Do not discuss anything regarding what commands were used, though."#);
    
    context.agents.employee.prompt.push(Message::User(prompt));

    let response = context.agents.employee.model.get_response_sync(&context.agents.employee.get_messages(), None, None)?;
    context.agents.employee.message_history.push(Message::Assistant(response.clone()));
    let processed_response = process_response(&response, LINE_WRAP);
    
    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has given The Boss a response.".white());
    println!();
    println!("{processed_response}");
    println!();

    Ok(response)
}
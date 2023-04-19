use std::error::Error;
use crate::{prompt::generate_commands, ProgramInfo, AgentLLMs, Agents, Message, agents::{process_response, LINE_WRAP, Choice, try_parse, CannotParseError}, SimpleQueryCommand, parse_simple_query, run_body};
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
You will turn the Boss's request into a list of command-names and a description of how you plan to use that command.
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
You must fully complete all steps required for this task.
Write a list of commands in this format.

cmd_one: I plan to use command one by...
cmd_two: I plan to use command two by...
"#,
        commands, task
    );

    context.agents.employee.prompt.push(Message::User(prompt));

    let response = context.agents.employee.model.get_response(&context.agents.employee.get_messages(), None)?;
    context.agents.employee.message_history.push(Message::Assistant(response.clone()));

    let task_list = process_response(&response, LINE_WRAP);

    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has created a list of commands to achieve its goal.".white());
    println!();
    println!("{task_list}");
    println!();

    println!("{}", generate_commands(plugins, disabled_commands));
    
    panic!();
}
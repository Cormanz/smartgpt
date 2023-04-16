use std::error::Error;
use crate::{ProgramInfo, AgentLLMs, Agents, Message, agents::{process_response, LINE_WRAP, run_boss}};
use colored::Colorize;

pub async fn run_manager(
    program: &mut ProgramInfo
) -> Result<(), Box<dyn Error>> {
    let ProgramInfo { context, task, .. } = program;
    let Agents { manager, .. } = &mut context.agents;

    manager.message_history.push(Message::System(
"You are The Manager, an LLM. 
Your goal is take advantage of your planning and self-criticism skills to plan out your task.
You have access to an employee named The Boss, who will carry out those steps."
        .to_string()
    ));

    manager.message_history.push(Message::User(format!(
"Hello, The Manager.

Your task is {:?}

Break it down into a list of short, high-level, one-sentence tasks.",
        task
    )));

    let response = manager.model.get_response(&manager.get_messages()).await?;
    manager.message_history.push(Message::Assistant(response.clone()));

    let task_list = process_response(&response, LINE_WRAP);

    println!("{}", "MANAGER".blue());
    println!("{}", "The manager has planned a list of tasks.".white());
    println!();
    println!("{task_list}");
    println!();

    manager.message_history.push(Message::User(
"Assign The Boss the first step in one paragraph".to_string()
    ));
    
    let response = manager.model.get_response(&manager.get_messages()).await?;
    let boss_request = process_response(&response, LINE_WRAP);

    println!("{}", "MANAGER".blue());
    println!("{}", "The manager has assigned a task to its employee, The Boss.".white());
    println!();
    println!("{boss_request}");
    println!();

    run_boss(program, &boss_request).await?;

    Ok(())
}
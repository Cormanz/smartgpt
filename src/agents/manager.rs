use std::error::Error;
use crate::{ProgramInfo, AgentLLMs, Agents, Message, agents::{process_response, LINE_WRAP, run_boss, Choice}};
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

    let mut first_prompt = true;

    loop {
        let ProgramInfo { context, task, .. } = program;
        let Agents { manager, .. } = &mut context.agents;
        
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
    
        let boss_response = run_boss(program, &boss_request, first_prompt, false).await?;
        first_prompt = false;
        let output = format!(
r#"The Boss has responded:
{}

You now have two choices.
A. The Boss was successful in finishing this step.
B. The Boss was incomplete in finishing this step. I shall provide feedback.

Provide your response in this format:

reasoning: Reasoning
choice: Choice # "A", "B" exactly.

Do not surround your response in code-blocks. Respond with pure YAML only.
"#,
                    boss_response
            );
            
        let ProgramInfo { context, .. } = program;
        let Agents { manager, .. } = &mut context.agents;
    
        manager.message_history.push(Message::User(output));
        
        let response = manager.model.get_response(&manager.get_messages()).await?;
        let manager_response = process_response(&response, LINE_WRAP);
    
        manager.message_history.push(Message::Assistant(response.clone()));
    
        println!("{}", "MANAGER".blue());
        println!("{}", "The Manager has made a decision on whether or not The Boss successfully completed the task.".white());
        println!();
        println!("{manager_response}");
        println!();
        
        let response: Choice = serde_yaml::from_str(&response)?;
    
        if response.choice == "A" {
            manager.message_history.push(Message::User(format!(
                "Remove the first task from your list. Then, once again, list all of the tasks."
            )));
            
            let response = manager.model.get_response(&manager.get_messages()).await?;
            manager.message_history.push(Message::Assistant(response.clone()));
        
            let task_list = process_response(&response, LINE_WRAP);
        
            println!("{}", "MANAGER".blue());
            println!("{}", "The manager has updated the list of tasks.".white());
            println!();
            println!("{task_list}");
            println!();
        } else {
            loop {
                let ProgramInfo { context, .. } = program;
                let Agents { manager, .. } = &mut context.agents;
                
                manager.message_history.push(Message::User(format!(
                    "Provide a list of feedback to provide to the boss."
                )));
                
                let response = manager.model.get_response(&manager.get_messages()).await?;
                manager.message_history.push(Message::Assistant(response.clone())); 

                let boss_response = run_boss(program, &response, first_prompt, true).await?;
                let output = format!(
r#"The Boss has responded:
{}

You now have two choices.
A. The Boss was successful in finishing this step.
B. The Boss was incomplete in finishing this step. I shall provide feedback.

Provide your response in this format:

reasoning: Reasoning
choice: Choice # "A", "B" exactly.

Do not surround your response in code-blocks. Respond with pure YAML only.
"#,
                    boss_response
                );
                    
                let ProgramInfo { context, .. } = program;
                let Agents { manager, .. } = &mut context.agents;
            
                manager.message_history.push(Message::User(output));
                
                let response = manager.model.get_response(&manager.get_messages()).await?;
                let manager_response = process_response(&response, LINE_WRAP);
            
                manager.message_history.push(Message::Assistant(response.clone()));
            
                println!("{}", "MANAGER".blue());
                println!("{}", "The Manager has made a decision on whether or not The Boss successfully completed the task.".white());
                println!();
                println!("{manager_response}");
                println!();
                
                let response: Choice = serde_yaml::from_str(&response)?;                       
            }
        }
    }

    Ok(())
}
use std::error::Error;
use crate::{ProgramInfo, AgentLLMs, Agents, Message, agents::{process_response, LINE_WRAP, run_boss, Choice, Task}};
use colored::Colorize;

pub fn run_manager(
    program: &mut ProgramInfo
) -> Result<(), Box<dyn Error>> {
    let ProgramInfo { context, task, personality, .. } = program;
    let mut context = context.lock().unwrap();

    context.agents.manager.llm.message_history.push(Message::System(format!(
"You are The Manager.

Personality: {}

Your goal is take advantage of your planning and self-criticism skills to plan out your task.
You have access to an employee named The Boss, who will carry out those steps.",
        personality
    )));

    context.agents.manager.llm.message_history.push(Message::User(format!(
"Hello, The Manager.

Your task is {:?}

You have no other information other than to complete this task.

Break it down into a list of short, high-level, one-sentence tasks.
Try to minimize the amount of tasks needed. Aim for five or less tasks.",
        task
    )));

    let response = context.agents.manager.llm.model.get_response_sync(&context.agents.manager.llm.get_messages(), None, None)?;
    context.agents.manager.llm.message_history.push(Message::Assistant(response.clone()));

    let task_list = process_response(&response, LINE_WRAP);

    println!("{}", "MANAGER".blue());
    println!("{}", "The manager has planned a list of tasks.".white());
    println!();
    println!("{task_list}");
    println!();

    let mut first_prompt = true;

    drop(context);

    loop {
        let ProgramInfo { context, task, personality, .. } = program;
        let mut context = context.lock().unwrap();

        context.agents.manager.llm.message_history.push(Message::User(
            "Assign The Boss the first step in one paragraph. Only tell The Boss about that step. Do not tell The Boss about your task.".to_string()
        ));
        
        let response = context.agents.manager.llm.model.get_response_sync(&context.agents.manager.llm.get_messages(), None, None)?;
        let boss_request = process_response(&response, LINE_WRAP);
    
        println!("{}", "MANAGER".blue());
        println!("{}", "The manager has assigned a task to its employee, The Boss.".white());
        println!();
        println!("{boss_request}");
        println!();

        drop(context);

        first_prompt = false;
        let boss_response = run_boss(program, Task::Task(boss_request))?;

        let ProgramInfo { context, task, personality, .. } = program;
        let mut context = context.lock().unwrap();

        let output = format!(
r#"The Boss has responded:
{}

You now have two choices.
A. The Boss was successful in finishing this step.
B. The Boss was incomplete in finishing this step. I shall provide feedback.

Provide your response in this format:

```json
{{
    "reasoning": "Reasoning",
    "choice": "Choice"
}}
```

Do not surround your response in code-blocks. Respond with pure JSON only.
"#,
                    boss_response
            );
    
        context.agents.manager.llm.message_history.push(Message::User(output));
        
        let response = context.agents.manager.llm.model.get_response_sync(&context.agents.manager.llm.get_messages(), None, None)?;
        let manager_response = process_response(&response, LINE_WRAP);
    
        context.agents.manager.llm.message_history.push(Message::Assistant(response.clone()));
    
        println!("{}", "MANAGER".blue());
        println!("{}", "The Manager has made a decision on whether or not The Boss successfully completed the task.".white());
        println!();
        println!("{manager_response}");
        println!();
        
        let response: Choice = serde_json::from_str(&response)?;
    
        if response.choice == "A" {
            context.agents.manager.llm.message_history.push(Message::User(format!(
                "Remove the first task from your list. Then, once again, list all of the tasks."
            )));
            
            let response = context.agents.manager.llm.model.get_response_sync(&context.agents.manager.llm.get_messages(), None, None)?;
            context.agents.manager.llm.message_history.push(Message::Assistant(response.clone()));
        
            let task_list = process_response(&response, LINE_WRAP);
        
            println!("{}", "MANAGER".blue());
            println!("{}", "The manager has updated the list of tasks.".white());
            println!();
            println!("{task_list}");
            println!();
        }
    }

    Ok(())
}
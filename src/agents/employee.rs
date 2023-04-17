use std::error::Error;
use crate::{prompt::generate_commands, ProgramInfo, AgentLLMs, Agents, Message, agents::{process_response, LINE_WRAP, Choice, try_parse}, QueryCommand, parse_query, run_body};
use colored::Colorize;

pub async fn try_again_employee(
    program: &mut ProgramInfo
) -> Result<bool, Box<dyn Error>> {
    for i in 0..2 {
        let ProgramInfo { 
            context, plugins, ..
        } = program;
        let Agents { employee, .. } = &mut context.agents;

        let queries_left = 3 - (i + 1);
        let prompt = format!(
r"You have {} command queries left, so finish up shortly. Please continue and write another command query.

You may only use one command.
Respond with pure YAML only.",
            queries_left
        ).trim().to_string();
        
        employee.message_history.push(Message::User(prompt));

        let (response, query): (_, QueryCommand) = try_parse(employee, 3).await?;
        employee.message_history.push(Message::Assistant(response.clone()));
        let response = process_response(&response, LINE_WRAP);

        println!("{}", "EMPLOYEE".blue());
        println!("{}", "The employee has created a command query.".white());
        println!();
        println!("{response}");
        println!();
        
        context.command_out.clear();

        let query = parse_query(vec![ query ]);
        run_body(context, &plugins, query).await?;
        
        let Agents { employee, .. } = &mut context.agents;

        println!("{}", "EXECUTOR".red());
        println!("{}", "The executor has ran the query..".white());
        println!();

        for item in &context.command_out {
            println!("{}", item);
        }

        println!();

        let output = format!(
    r#"{}

You now have three choices.
A. My query was successful, and I am done. I will provide my response to The Boss.
B. My query was successful, but I am not done. I will continue with another query.
C. My query was not successful, so I will continue with another query.

Provide your response in this format:
```
reasoning: Reasoning
choice: A
```
"#,
            context.command_out.join("\n")
    );

        employee.message_history.push(Message::User(output));

        let (response, choice): (_, Choice) = try_parse(employee, 3).await?;
        employee.message_history.push(Message::Assistant(response.clone()));
        let response = process_response(&response, LINE_WRAP);

        println!("{}", "EMPLOYEE".blue());
        println!("{}", "The employee has made a decision on whether to keep going.".white());
        println!();
        println!("{response}");
        println!();
        
        if choice.choice == "A" {
            return Ok(false);
        }
    }

    Ok(true)
}

pub async fn run_employee(
    program: &mut ProgramInfo, task: &str, new_prompt: bool
) -> Result<String, Box<dyn Error>> {
    let ProgramInfo { 
        context, plugins, 
        disabled_commands, .. 
    } = program;
    let Agents { employee, .. } = &mut context.agents;

    let commands = generate_commands(plugins, disabled_commands);

    if new_prompt {
        employee.prompt.push(Message::System(
            "You are The Employee, an LLM. 
            Your goal is take advantage of access to commands to provide answers to questions.
            You have been given one task from The Boss."
                .to_string()
        ));
    
        let prompt = format!("
You have access to these commands:
{}

Your task is {:?}
You will write a command query in this format.

```
name: command_name
args:
- !Data Arg
```

Always use the `!Data` annotation, no matter the datatype.

Please write a command query for to complete the task, in the given format above.
You may only use one command.
Respond with pure YAML only.", commands, task
        );
    
        employee.prompt.push(Message::User(prompt));
    } else {
        employee.message_history.push(Message::User(
            format!("
The Boss has assigned a new task: {:?}

Please write a command query for it, in the same format as before. 
You may only use one command.
Respond with pure YAML only.",
            task
        )));
    }
    
    employee.crop_to_tokens(2000)?;

    let (response, query): (_, QueryCommand) = try_parse(employee, 3).await?;
    employee.message_history.push(Message::Assistant(response.clone()));
    let response = process_response(&response, LINE_WRAP);

    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has created a command query.".white());
    println!();
    println!("{response}");
    println!();
    
    context.command_out.clear();

    let query = parse_query(vec![ query ]);
    run_body(context, &plugins, query).await?;
    
    let Agents { employee, .. } = &mut context.agents;

    println!("{}", "EXECUTOR".red());
    println!("{}", "The executor has ran the query..".white());
    println!();

    for item in &context.command_out {
        println!("{}", item);
    }

    println!();

    let output = format!(
r#"{}

You now have three choices.
A. My query was successful, and I am done. I will provide my response to The Boss.
B. My query was successful, but I am not done. I will continue with another query.
C. My query was not successful, so I will continue with another query.

Provide your response in this format:

```
reasoning: Reasoning
choice: A
```

Respond with pure YAML only.
"#,
        context.command_out.join("\n")
);

    employee.message_history.push(Message::User(output));

    let (response, choice): (_, Choice) = try_parse(employee, 3).await?;
    employee.message_history.push(Message::Assistant(response.clone()));
    let response = process_response(&response, LINE_WRAP);

    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has made a decision on whether to keep going.".white());
    println!();
    println!("{response}");
    println!();

    let mut employee_finished_abruptly = false;
    if choice.choice != "A" {
        employee_finished_abruptly = try_again_employee(program).await?;
    }

    let ProgramInfo { 
        context, plugins, 
        disabled_commands, .. 
    } = program;
    let Agents { employee, .. } = &mut context.agents;
    if employee_finished_abruptly {
        employee.message_history.push(Message::User(format!(
"You have finished abruptly: you were still working on finishing your task, but could not finish in three queries. Make sure you tell this to the boss.

Provide a reponse that answers the initial task to the Boss based on your findings."
        )));
    } else {
        employee.message_history.push(Message::User(format!(
"Provide a reponse that answers the initial task to the Boss based on your findings."
        )));     
    }

    let response = employee.model.get_response(&employee.get_messages()).await?;
    employee.message_history.push(Message::Assistant(response.clone()));

    let employee_response = process_response(&response, LINE_WRAP);
    
    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has given The Boss a response..".white());
    println!();
    println!("{employee_response}");
    println!();

    Ok(response)
}
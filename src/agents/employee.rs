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

pub async fn try_again_employee(
    program: &mut ProgramInfo,
    mut command_query: String
) -> Result<(String, bool), Box<dyn Error>> {
    for i in 0..2 {
        let ProgramInfo { 
            context, plugins, ..
        } = program;
        let Agents { employee, .. } = &mut context.agents;

        let (response, query): (_, SimpleQueryCommand) = try_parse(employee, 3, Some(1000)).await?;
        employee.message_history.push(Message::Assistant(response.clone()));
        let response = process_response(&response, LINE_WRAP);

        println!("{}", "EMPLOYEE".blue());
        println!("{}", "The employee has created a command query.".white());
        println!();
        println!("{response}");
        println!();
        
        context.command_out.clear();

        let query = parse_simple_query(vec![ query ]);
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

You now have two choices.
A. I am completely done with my task from The Boss.
B. I am almost done with my task from The Boss.

Provide your response in one of these formats depending on the choice:

reasoning: Reasoning
choice: A
report: |-
    Dear Boss...

reasoning: Reasoning
choice: B
command query:
    command name: A
    args:
    - Arg

Respond with pure YAML only. Ensure your response can be parsed by serde_yaml.
    "#,
        context.command_out.join("\n")
    );

        employee.message_history.push(Message::User(output));

        let (response, choice): (_, EmployeeDecision) = try_parse(employee, 3, Some(1000)).await?;
        employee.message_history.push(Message::Assistant(response.clone()));
        let response = process_response(&response, LINE_WRAP);

        println!("{}", "EMPLOYEE".blue());
        println!("{}", "The employee has made a decision on whether to keep going.".white());
        println!();
        println!("{response}");
        println!();
        
        if choice.choice == "A" {
            return Ok((
                choice.report.ok_or(CannotParseError)?, 
                false
            ));
        }

        command_query = choice.command_query.ok_or(CannotParseError)?;
    }

    let ProgramInfo { 
        context, plugins, ..
    } = program;
    let Agents { employee, .. } = &mut context.agents;

    employee.message_history.push(Message::User(format!(
        "You have finished abruptly: you were still working on finishing your task, but could not finish in three queries. Make sure you tell this to the boss.
        
        Provide a reponse that answers the initial task to The Boss based on your findings.
        Do not give The Boss any details on what specific commands you used. Only discuss your findings."
    )));

    Ok((
        employee.model.get_response(&employee.get_messages(), Some(1000)).await?,
        true
    ))
}

pub async fn run_employee(
    program: &mut ProgramInfo, task: &str, new_prompt: bool
) -> Result<String, Box<dyn Error>> {
    let ProgramInfo { 
        context, plugins, personality, 
        disabled_commands, .. 
    } = program;
    let Agents { employee, .. } = &mut context.agents;

    let commands = generate_commands(plugins, disabled_commands);

    if new_prompt {
        employee.prompt.push(Message::System(format!(
            "You are The Employee, a large language model. 

Personality: {}

Your goal is take advantage of access to commands to provide answers to questions.
You have been given one task from The Boss.",
            personality
        )));
    
        let prompt = format!(r#"
You have access to these commands:
{}

Your task is {:?}
You must fully complete all steps required for this task.
You will write a command query in this format.

```yml
name: command_name
args:
- "Argument 1"
```

Use this exact format as described.
YOU MAY ONLY USE ONE COMMAND FOR THIS QUERY.

If you are asked to save information to a file or do some other additional task, please do that before you report back to The Boss.

Respond with pure YAML only. Ensure your response can be parsed by serde_yaml"#,
            commands, task
        );
    
        employee.prompt.push(Message::User(prompt));
    } else {
        employee.message_history.push(Message::User(
            format!(r#"
The Boss has assigned a new task: {:?}
You must fully complete all steps required for this task.
You will write a command query in this format.

```yml
name: command_name
args:
- "Argument 1"
```

Use this exact format as described.
YOU MAY ONLY USE ONE COMMAND FOR THIS QUERY.

If you are asked to save information to a file or do some other additional task, please do that before you report back to The Boss.

Respond with pure YAML only. Ensure your response can be parsed by serde_yaml"#,
            task
        )));
    }
    
    employee.crop_to_tokens(2500)?;

    let (response, query): (_, SimpleQueryCommand) = try_parse(employee, 3, Some(1000)).await?;
    employee.message_history.push(Message::Assistant(response.clone()));
    let response = process_response(&response, LINE_WRAP);

    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has created a command query.".white());
    println!();
    println!("{response}");
    println!();
    
    context.command_out.clear();

    let query = parse_simple_query(vec![ query ]);
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

You now have two choices.
A. I am completely done with my task from The Boss.
B. I am almost done with my task from The Boss.

Provide your response in one of these formats depending on the choice:

reasoning: Reasoning
choice: A
report: |-
    Dear Boss...

reasoning: Reasoning
choice: B
command query:
    command name: A
    args:
    - Arg

Respond with pure YAML only. Ensure your response can be parsed by serde_yaml.
"#,
        context.command_out.join("\n")
);

    employee.message_history.push(Message::User(output));

    let (response, decision): (_, EmployeeDecision) = try_parse(employee, 3, Some(1000)).await?;
    employee.message_history.push(Message::Assistant(response.clone()));
    let response = process_response(&response, LINE_WRAP);

    employee.crop_to_tokens(2500)?;

    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has made a decision on whether to keep going.".white());
    println!();
    println!("{response}");
    println!();

    let mut employee_finished_abruptly: (String, bool) = ("".to_string(), false);
    if decision.choice != "A" {
        employee_finished_abruptly = try_again_employee(
            program, 
            decision.command_query.clone().ok_or(CannotParseError)?
        ).await?;
    }

    Ok(employee_finished_abruptly.0)
}
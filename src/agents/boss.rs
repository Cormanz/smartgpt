use std::{error::Error, fmt::Display};
use crate::{ProgramInfo, AgentLLMs, Agents, Message, agents::{process_response, LINE_WRAP, run_employee, Choice, try_parse}};
use colored::Colorize;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct BossDecision {
    pub choice: String,
    pub report: Option<String>,
    #[serde(rename = "new request")] pub new_request: Option<String>
}

#[derive(Debug, Clone)]
pub struct NoManagerRequestError;

impl Display for NoManagerRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "could not parse.")
    }
}

impl Error for NoManagerRequestError {}

pub fn run_boss(
    program: &mut ProgramInfo, task: &str, first_prompt: bool, feedback: bool,
) -> Result<String, Box<dyn Error>> {
    let ProgramInfo { context, plugins, personality, .. } = program;
    let mut context = context.lock().unwrap();

    if first_prompt {
        let commands = plugins.iter()
            .flat_map(|el| el.commands.iter())
            .map(|el| el.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        context.agents.boss.prompt.push(Message::System(format!(
"You are The Boss, a large language model.

Personality: {}

You have been assigned one task by The Manager, a large language model. You will use your loose planning and adaptability to complete this task.
Your goal is to quickly and efficiently get the task done without refining it too much. You just want to get a sort of quicker, shallower answer.
Complete your task as quickly as possible.

You have access to one employee named The Employee, a large language model, who can run commands for you.
These commands are: {}

Your Employee is not meant to do detailed work, but simply to help you find information.

Only ask The Employee for one thing at a time.
Keep your Employee requests very simple.
Make sure to tell the Employee to save important information to files!

You cannot do anywork on your own. You will do all of your work through your Employee."
            , personality, commands
        )));
    }

    if feedback {
        context.agents.boss.message_history.push(Message::User(format!(
"Hello, The Boss.

The Manager has provided you with the following feedback: {:?}

Continue to work with The Employee to complete your task based on this feedback.",
                task
            )));
    } else if first_prompt {
        context.agents.boss.message_history.push(Message::User(format!(
"Hello, The Boss.

Your task is {:?}
Don't worry if you're unable to complete this task as an LLM, you will complete this task through your Employee.

Write a 2-sentence loose plan of how you will achieve this.",
                task
            )));
    } else {
        context.agents.employee.prompt.clear();
        context.agents.employee.message_history.clear();

        context.agents.boss.message_history.push(Message::User(format!(
            "Hello, The Boss.

Your task is {:?}

Keep in mind that you have been given a new Employee. You may need to brief them on any details they need to complete their tasks.

Write a 2-sentence loose plan of how you will achieve this.",
                task
        )));
    }

    context.agents.boss.crop_to_tokens(1000)?;

    let response = context.agents.boss.model.get_response_sync(&context.agents.boss.get_messages(), None, None)?;
    context.agents.boss.message_history.push(Message::Assistant(response.clone()));

    let task_list = process_response(&response, LINE_WRAP);

    println!("{}", "BOSS".blue());
    println!("{}", "The boss has created a loose plan to achieve its goal.".white());
    println!();
    println!("{task_list}");
    println!();

    let mut new_prompt = true;
    let mut new_request: Option<String> = None;

    drop(context);
    loop {
        let response = match &new_request {
            Some(request) => request.clone(),
            None => {
                let ProgramInfo { context, plugins, personality, .. } = program;
                let mut context = context.lock().unwrap();
        
                context.agents.boss.message_history.push(Message::User(
                    "Create one simple request for The Employee. 
        Do not give your employee specific commands, simply phrase your request with natural language.
        Provide a very narrow and specific request for the Employee.
        Remember: Your Employee is not meant to do detailed work, but simply to help you find information.
        Make sure to tell the Employee to save important information to files!"
                        .to_string()
                ));
        
                let response = context.agents.boss.model.get_response_sync(&context.agents.boss.get_messages(), None, None)?;
                let boss_request = process_response(&response, LINE_WRAP);

                println!("{}", "BOSS".blue());
                println!("{}", "The boss has assigned a task to its employee, The Employee.".white());
                println!();
                println!("{boss_request}");
                println!();

                drop(context);

                response
            }
        };

        let employee_response = run_employee(program, &response, new_prompt)?;
        new_prompt = false;

        let output = format!(
r#"The Employee has responded:
{}

You now have three choices.
A. I have finished the task The Manager provided me with. I shall report back with the information to The Manager.
B. I have not finished the task. The Employee's response provided me with plenty of new information, so I will update my loose plan.
C. I have not finished the task. I shall proceed onto asking the Employee my next request.

Provide your response in one of these formats depending on the choice:

reasoning: Reasoning
choice: A
report: |-
    Dear Manager...

reasoning: Reasoning
choice: B
new loose plan: |-
    First...
new request: |-
    Can you try...

reasoning: Reasoning
choice: C
new request: |-
    Can you try...

Do not surround your response in code-blocks. Respond with pure YAML only. Ensure your response can be parsed by serde_yaml.
"#,
        employee_response
);

        let ProgramInfo { context, plugins, personality, .. } = program;
        let mut context = context.lock().unwrap();

        context.agents.boss.message_history.push(Message::User(output));
        
        let (response, decision): (_, BossDecision) = try_parse(&context.agents.boss, 3, Some(300))?;
        context.agents.boss.message_history.push(Message::Assistant(response.clone()));
        let response = process_response(&response, LINE_WRAP);

        println!("{}", "BOSS".blue());
        println!("{}", "The boss has made a decision on whether to keep going.".white());
        println!();
        println!("{response}");
        println!();
    
        if decision.choice == "A" {
            return Ok(decision.report.ok_or(NoManagerRequestError)?)
        }

        new_request = decision.new_request;
    }
}
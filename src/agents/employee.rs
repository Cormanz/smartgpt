use std::error::Error;
use crate::{prompt::generate_commands, ProgramInfo, AgentLLMs, Agents, Message, agents::{process_response, LINE_WRAP, Choice, try_parse, CannotParseError, minion::run_minion}, AgentInfo, Weights};
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EmployeeDecision {
    pub psuedocode: String,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct EmployeeResponse {
    #[serde(rename = "memory query")] pub memory_query: Option<String>,
    pub observations: Option<Vec<String>>,
    #[serde(rename = "report to boss")] pub report: String
}

pub fn run_employee(
    program: &mut ProgramInfo, task: &str, new_prompt: bool,
    memory_query: Option<String>,
) -> Result<EmployeeResponse, Box<dyn Error>> {
    let ProgramInfo { context, plugins, personality, disabled_commands, .. } = program;
    let mut context = context.lock().unwrap();

    let commands = plugins.iter()
        .flat_map(|el| el.commands.iter())
        .map(|el| el.name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    context.agents.employee.llm.prompt.clear();
    context.agents.employee.llm.message_history.clear();

    context.agents.employee.llm.prompt.push(Message::System(format!(
        "You are The Employee, a large language model. 

Personality: {}

You have access to these commands:
{}

Your goal is take advantage of access to commands to provide answers to questions.

You have a minion named The Minion, who will turn your psuedocode into a script and run it.
You will turn the Boss's request into simple, tiny psuedocode.
Keep your psuedocode idea very short and concise. Do not make something complicated.",
        personality, commands
    )));

    let AgentInfo { llm, observations, .. } = &mut context.agents.employee;
    let observations = observations.get_memories_sync(
        &llm,
        memory_query.as_deref().unwrap_or("None"),
        200,
        Weights {
            recall: 1.,
            recency: 1.,
            relevance: 1.
        },
        50
    )?;
    let observation_text = if observations.len() == 0 {
        "None found.".to_string()
    } else {
        observations.iter().enumerate()
            .map(|(ind, observation)| format!("{ind}. {}", observation.content))
            .collect::<Vec<_>>()
            .join("\n")
    };

    drop(llm);
    drop(observations);

    let formatted_observations = process_response(&observation_text, LINE_WRAP);

    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has found its observations.".white());
    println!();
    println!("{formatted_observations}");
    println!();


    context.agents.employee.llm.message_history.push(Message::System(format!(
"TASK
{task:?}

OBSERVATIONS
{observation_text}"
    )));

    let prompt = format!(r#"
Info on Psuedocode:
Your psuedocode should be human readable, but follow the flow of logic that code does.
Whenever you save a file, use ".txt" for the extension.

Respond in this format:

```yml
psuedocode: |-
    ask ChatGPT for...
```

All fields must be specified exactly as shown above.
If you do not want to put a specific field, put the field, but set its value to `null`.

Ensure your response is in the exact YAML format as specified."#);  

    context.agents.employee.llm.prompt.push(Message::User(prompt));

    let (response, decision) = try_parse::<EmployeeDecision>(&context.agents.employee.llm, 2, Some(1000))?;
    context.agents.employee.llm.message_history.push(Message::Assistant(response.clone()));

    let formatted_response = process_response(&response, LINE_WRAP);

    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has nade a decision and some psuedocode.".white());
    println!();
    println!("{formatted_response}");
    println!();

    drop(context);
    let out = run_minion(program, &decision.psuedocode, new_prompt)?;

    let ProgramInfo { context, plugins, personality, disabled_commands, .. } = program;
    let mut context = context.lock().unwrap();

    let prompt = format!(r#"Your query gave the following output:
    
{out}

Info on Report to Boss:
Please write a response to The Boss with your findings. 
Only discuss RESULTS. 
Provide very specific information.
What exact information was saved? What was the name of the file? What sources were they?
Be specific.
Do not discuss anything regarding what commands were used.

Info on Memory Queries:
Your memory query is a very short summary of every topic in your mind that is relevant at this moment.
Think of it like a search query.
Your memory query will be used to help you find relevant observations and reflections.

Respond in this format:

```yml
observations: # can be `null`
- A
- B

memories query: |-
    I am working on...

report to boss: |-
    Dear Boss, I managed...
```"#);
    
    context.agents.employee.llm.prompt.push(Message::User(prompt));

    let (response, decision) = try_parse::<EmployeeResponse>(&context.agents.employee.llm, 2, Some(1000))?;
    context.agents.employee.llm.message_history.push(Message::Assistant(response.clone()));

    let formatted_response = process_response(&response, LINE_WRAP);
    
    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has given The Boss a response.".white());
    println!();
    println!("{formatted_response}");
    println!();

    Ok(decision)
}
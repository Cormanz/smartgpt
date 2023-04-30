use std::error::Error;
use crate::{prompt::generate_commands, ProgramInfo, AgentLLMs, Agents, Message, agents::{process_response, LINE_WRAP, Choice, try_parse, CannotParseError, minion::run_minion}, AgentInfo, Weights};
use colored::Colorize;
use serde::{Deserialize, Serialize, __private::de};

#[derive(Serialize, Deserialize)]
pub struct EmployeeDecision {
    pub request: String,
}

#[derive(Serialize, Deserialize)]
pub struct EmployeeQueryDecision {
    #[serde(rename = "memory query")] pub memory_query: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EmployeeResponse {
    #[serde(rename = "memory query")] pub memory_query: String,
    pub observations: Vec<String>,
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

You have a minion named The Minion, who will turn your request into a script and run it.
You will turn the Boss's request into simple, tiny, natural language request.
Keep your request idea very short and concise. Do not make something complicated.",
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
Your request should be human readable.
Keep it very, very straightforward.
Whenever you save a file, use ".txt" for the extension.
Include SPECIFIC command names.

Respond in this format:

```yml
idea: |-
    Maybe I could...
request: |-
    Use google_search to find information on bunnies. Then, on the first three results, use browse_website, and save a file with the content.```
```

Use natural language for your request.

List of commands: {}

All fields must be specified exactly as shown above.
If you do not want to put a specific field, put the field, but set its value to `null`.

Ensure your response is in the exact YAML format as specified."#,
        plugins.iter().flat_map(|el| &el.commands).map(|el| el.name.clone()).collect::<Vec<_>>().join(", ")
    );  

    context.agents.employee.llm.prompt.push(Message::User(prompt));

    let (response, decision) = try_parse::<EmployeeDecision>(&context.agents.employee.llm, 3, Some(1000))?;
    context.agents.employee.llm.message_history.push(Message::Assistant(response.clone()));

    let formatted_response = process_response(&response, LINE_WRAP);

    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has made a decision and some request.".white());
    println!();
    println!("{formatted_response}");
    println!();

    drop(context);
    let (letter, details) = run_minion(program, &decision.request, new_prompt)?;

    let ProgramInfo { context, plugins, personality, disabled_commands, .. } = program;
    let mut context = context.lock().unwrap();
    
    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has given The Boss a response.".white());
    println!();
    println!("{letter}");
    println!();

    let new_observations = details.findings.iter()
        .chain(details.changes.iter())
        .map(|el| el.clone())
        .collect::<Vec<_>>();

    context.agents.employee.llm.prompt.clear();
    context.agents.employee.llm.message_history.clear();

    context.agents.employee.llm.prompt.push(Message::System(format!(
r#"You have a list of relevant observations and reflections from the previous query.

Create a new memory query. It should discuss all currently relevant topics, and be brief. It will be used to find new memories. Memory queries are like google searches.

Example of a memory query: "Spongebob Show, Squidward, Crusty Crab"

Reply in this format:
```yml
reasoning: [...]
memory query: [...]
```"#
    )));

    context.agents.employee.llm.message_history.push(Message::User(format!(
"PREVIOUS BOSS REQUEST
{task}

PREVIOUS EMPLOYEE RESPONSE
{letter}

OBSERVATIONS
{observation_text}"
    )));

    let (response, query_decision) = try_parse::<EmployeeQueryDecision>(&context.agents.employee.llm, 3, Some(1000))?;
    context.agents.employee.llm.message_history.push(Message::Assistant(response.clone()));

    let formatted_response = process_response(&response, LINE_WRAP);

    println!("{}", "EMPLOYEE".blue());
    println!("{}", "The employee has made a decision on its next memory query.".white());
    println!();
    println!("{formatted_response}");
    println!();

    drop(context);

    for observation in &new_observations {
        let mut context = program.context.lock().unwrap();
        let AgentInfo { llm, observations, .. } = &mut context.agents.employee;
        observations.store_memory_sync(llm, &observation)?;
    }

    Ok(EmployeeResponse {
        memory_query: query_decision.memory_query.clone(),
        observations: new_observations,
        report: letter
    })
}
use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{ProgramInfo, Message, auto::{try_parse_json, agents::findings::get_observations}, LLM, AgentInfo};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Brainwave {
    pub thoughts: String,
    pub request_for_executor: String
}

pub fn prompt_brainstormer(agent: &mut AgentInfo, personality: &str, task: &str) -> Result<(), Box<dyn Error>> {
    agent.llm.prompt.push(Message::System(format!(
r#"
Role: 
{personality}

You are The Brainstormer. Your goal is to complete the task one idea at a time, and then have the Executor use one of its resources to run your idea.
Keep in mind that both you are a large language model.
"#
    )));

    let observations = get_observations(agent, task)?
        .unwrap_or("None found.".to_string());
    
    agent.llm.prompt.push(Message::User(format!(
r#"
Tools:
google_search {{ "request": "..." }}
browse_url {{ "url": "..." }}

Task: 
{task}

Observations:
{observations}

You'll try to brainstorm a thought on how you can get closer to completing your goal.
Make use of your observations, they're your memory.

Then, give the Executor a one-sentence instruction.

{{
    "thoughts": "...",
    "action": {{
        "tool": "...",
        "query": {{ ... }}
    }}
}}

Respond in this JSON format.
"#
    )));

    Ok(())
}

pub fn brainstorm(program: &mut ProgramInfo, task: &str) -> Result<Brainwave, Box<dyn Error>> {
    let ProgramInfo { 
        context, plugins, personality,
        disabled_commands, .. 
    } = program;
    let mut context = context.lock().unwrap();

    context.agents.employee.llm.prompt.clear();
    context.agents.employee.llm.message_history.clear();

    prompt_brainstormer(&mut context.agents.employee, &personality, task)?;


    try_parse_json(&context.agents.employee.llm, 2, Some(400)).map(|res| res.data)
}
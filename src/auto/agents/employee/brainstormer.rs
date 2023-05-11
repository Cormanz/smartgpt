use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{ProgramInfo, Message, auto::{try_parse_json, agents::findings::get_observations}, LLM, AgentInfo, ScriptValue};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Action {
    pub tool: String,
    pub args: ScriptValue
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Brainwave {
    pub thoughts: String,
    pub action: Action
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandObservations {
    #[serde(rename = "was the command successful")]
    pub command_success: bool,
    #[serde(rename = "permanent changes")]
    pub changes: Option<Vec<String>>,
    #[serde(rename = "mental notes")]
    pub notes: Option<Vec<String>>
}

pub fn prompt_brainstormer(agent: &mut AgentInfo, personality: &str) {
    agent.llm.prompt.push(Message::System(format!(
        r#"
        Role: 
        {personality}
        
        You are The Brainstormer. Your goal is to complete the task one idea at a time, and then have the Executor use one of its resources to run your idea.
        Keep in mind that both you are a large language model.
        "#
    )));
}

pub fn prompt_brainstorm_session(agent: &mut AgentInfo, personality: &str, task: &str) -> Result<(), Box<dyn Error>> {
    prompt_brainstormer(agent, personality);

    let observations = get_observations(agent, task)?
        .unwrap_or("None found.".to_string());
    
    agent.llm.prompt.push(Message::User(format!(
r#"
Tools:
google_search {{ "query": "..." }}
browse_url {{ "url": "..." }}

Task: 
{task}

Observations:
These are observations you have saved for use later. Think about them, and build on them.
{observations}

You'll try to brainstorm a thought on how you can get closer to completing your goal.
Make use of your observations, they're your memory.

Then, give the Executor a one-sentence instruction.

{{
    "thoughts": "...",
    "action": {{
        "tool": "...",
        "args": {{ ... }}
    }}
}}

Respond in this JSON format.
"#
    )));

    Ok(())
}

pub fn prompt_collect_observations(agent: &mut AgentInfo, out: &str) -> Result<(), Box<dyn Error>> {
    agent.llm.message_history.push(Message::User(out.into()));
    agent.llm.message_history.push(Message::User(format!(
r#"
Collect mental notes and permanent changes from your command.
Inside of each individual mental note, cite exact sources with URLs if possible.
You can only have up to three mental notes.

Permanent changes should only be for information such as file reading.

If the command was not successful, you should make an observation about that.

{{
    "was the command successful": true / false,
    "permanent changes": [
        "..."
    ],
    "mental notes": [
        "..."
    ]
}}

Respond in this JSON format.
"#
    )));

    Ok(())
}

pub fn brainstorm(program: &mut ProgramInfo, task: &str) -> Result<Brainwave, Box<dyn Error>> {
    let ProgramInfo { 
        context, personality, ..
    } = program;
    let mut context = context.lock().unwrap();

    context.agents.employee.llm.prompt.clear();
    context.agents.employee.llm.message_history.clear();

    prompt_brainstorm_session(&mut context.agents.employee, &personality, task)?;

    try_parse_json(&context.agents.employee.llm, 2, Some(400))
        .map(|res| {
            context.agents.employee.llm.message_history.push(Message::User(res.raw));
            res.data
        })
}

pub fn collect_observations(program: &mut ProgramInfo, out: &str) -> Result<CommandObservations, Box<dyn Error>> {
    let ProgramInfo { 
        context, personality, ..
    } = program;
    let mut context = context.lock().unwrap();

    prompt_collect_observations(&mut context.agents.employee, out)?;

    try_parse_json(&context.agents.employee.llm, 2, Some(400)).map(|res| res.data)
}
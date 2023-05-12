use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{ProgramInfo, Message, auto::{try_parse_json, agents::findings::{get_observations, get_reflections}}, LLM, AgentInfo, ScriptValue, Weights};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Action {
    pub tool: String,
    pub args: ScriptValue
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Brainwave {
    pub thoughts: String,
    #[serde(rename = "tool-oriented idea")]
    pub idea: String,
    #[serde(rename = "action based on idea")]
    pub action: Action
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandObservations {
    #[serde(rename = "was I successful")]
    pub tool_success: bool,
    #[serde(rename = "what did I do")]
    pub explanation: String,
    #[serde(rename = "mental notes from tool use output")]
    pub notes: Option<Vec<String>>,
    #[serde(rename = "permanent changes")]
    pub changes: Option<Vec<String>>
}

pub fn prompt_brainstormer(agent: &mut AgentInfo, personality: &str) {
    agent.llm.prompt.push(Message::System(format!(
r#"
Role: 
{personality}

Your goal is to brainstorm effective and creative solutions to the task given, and then execute them using task-oriented actions.
You keep track of memories using short-term observations and long-term reflections.
You are a large language model.
"#
    )));
}

pub fn prompt_brainstorm_session(agent: &mut AgentInfo, personality: &str, task: &str) -> Result<(), Box<dyn Error>> {
    prompt_brainstormer(agent, personality);

    let observations = get_observations(agent, task, 25, Default::default())?
        .unwrap_or("None found.".to_string());

    let reflections = get_reflections(agent, task, 4, Weights {
        recall: 1.,
        recency: 1.5,
        relevance: 1.
    })?
        .unwrap_or("None found.".to_string());
    
    agent.llm.prompt.push(Message::User(format!(
r#"
Tools:
google_search {{ "query": "..." }}
wolfram {{ "query": "solve ..." }}
    Use pure mathematical equations, don't give wolfram any additional context
browse_url {{ "url": "..." }}
    You can only read paragraph-only content from websites, you cannot interact with them.
file_append {{ "path": "...", "content": "..." }}
none {{}}

Task: 
{task}

Observations:
These are observations you have saved. Think about them, and build on them.
{observations}


Reflections:
These are long-term reflections you have saved. Reflections are very important; analyze these critically, and build on them.
{reflections}

Make use of your observations and reflections, they're your memory.

You'll try to brainstorm a thought on how you can make progress on your task.

You'll then choose one short idea of how you can make said progress, listing one specific tool that you will use.

Then, based on that idea, choose an action with a tool.

If your task is complete, use the tool of "none."

{{
    "thoughts": "...",
    "tool-oriented idea": "...",
    "action based on idea": {{
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
Tell me exactly what tool you used and why."

Collect mental notes and permanent changes from the response you got from your tool use.
Inside of each individual mental note, cite exact sources with URLs if possible.

Permanent changes should only be for information such as file reading.
If the tool use was not successful, you should make an observation about that.

Make sure every mental note is directly related to your task in some way.
You must have four mental notes which are all detailed.

Please note that the "permanent changes" field can be `null`.

{{
    "was I successful": true / false,
    "what did I do": "...",
    "mental notes from tool use output": [
        "... (citation: ...)"
    ],
    "permanent changes": [
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
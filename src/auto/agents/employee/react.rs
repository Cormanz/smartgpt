use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{run::Action, try_parse_json}};

use super::use_tool;

#[derive(Serialize, Deserialize)]
pub struct Thoughts {
    pub thoughts: String,
    pub reasoning: String,
    pub criticism: String,
    #[serde(rename = "is my task complete")]
    pub done: bool,
    pub action: Action
}

pub fn get_next_action(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
) -> Result<Thoughts, Box<dyn Error>> {
    Ok(
        try_parse_json(&get_agent(context).llm, 2, Some(400))
            .map(|res| res.data)?
    )
}

pub enum ActionResults {
    TaskComplete(String),
    Results(String)
}

pub fn run_react_action(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str
) -> Result<ActionResults, Box<dyn Error>> {
    let thoughts: Thoughts = get_next_action(context, get_agent)?;
    if thoughts.done {
        panic!("E");
    }

    Ok(ActionResults::Results(
        use_tool(context, get_agent, thoughts.action)?
    ))
}

pub fn run_react_agent(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str
) {
    let agent = get_agent(context);
    agent.llm.clear_history();

    agent.llm.message_history.push(Message::System(format!("
You are an Agent.
You will complete your task, one tool at a time.
")));

    agent.llm.message_history.push(Message::User(format!(r#"
Tools:
google_search {{ "query": "..." }}
wolfram {{ "query": "solve ..." }}
    Use pure mathematical equations, don't give wolfram any additional context
browse_url {{ "url": "..." }}
    You can only read paragraph-only content from websites, you cannot interact with them.
file_append {{ "path": "...", "content": "..." }}
llm_process {{ "data": "...", "request": "..." }}
    This uses a large language model to process a given set of data for insights.

Task:
{task}

Complete your task as fast as possible.
Use as minimal tools as possible.
Do not try to constantly refine your output; move to the next tool.

Do not put "action" if you are done.
Respond in this format:

{{
    "thoughts": "...",
    "reasoning": "...",
    "criticism": "...",
    "is my task complete": bool,
    "action": {{
        "tool": "...",
        "args": {{ ... }}
    }}
}}
"#)));


}
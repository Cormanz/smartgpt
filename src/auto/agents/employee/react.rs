use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{run::Action, try_parse_json}};

use super::use_tool;

pub fn log_yaml<T: Serialize>(data: &T) -> Result<(), Box<dyn Error>> {
    println!("{}", serde_yaml::to_string(&data)?);

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct Thoughts {
    #[serde(rename = "is my task complete")]
    pub done: bool,
    pub thoughts: String,
    pub reasoning: String,
    pub criticism: String,
    pub action: Option<Action>
}

pub fn explain_results(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo
) -> Result<String, Box<dyn Error>> {
    let agent = get_agent(context);
   agent.llm.message_history.push(Message::System(format!(
"Now that you have finished your task, write a detailed, readable and simple Markdown response.
Your response should be easily understandable for a human, and convey all information in an accessible format.
Respond in exact plaintext; no JSON."
    )));

    agent.llm.model.get_response_sync(&agent.llm.get_messages(), Some(600), None)
}

pub fn get_thoughts(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
) -> Result<Thoughts, Box<dyn Error>> {
    Ok(
        try_parse_json(&get_agent(context).llm, 2, Some(1000))
            .map(|res| {
                get_agent(context).llm.message_history.push(Message::Assistant(res.raw));
                res.data
            })?
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
    let thoughts: Thoughts = get_thoughts(context, get_agent)?;
    log_yaml(&thoughts)?;

    match thoughts.action {
        Some(action) => {
            Ok(ActionResults::Results(
                use_tool(context, &|context| &mut context.agents.fast, action)?
            ))
        }
        None => {
            Ok(ActionResults::TaskComplete(
                explain_results(context, &get_agent)?
            ))
        }
    }
}

pub fn run_react_agent(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str
) -> Result<String, Box<dyn Error>> {
    let agent = get_agent(context);
    agent.llm.clear_history();

    agent.llm.message_history.push(Message::System(format!("
You are an Agent.
You will complete your task, one tool at a time.
")));

    agent.llm.message_history.push(Message::User(format!(r#"
Tools:
google_search {{ "query": "..." }}
wolfram {{ "query": "..." }}
browse_url {{ "url": "..." }}
    You can only read paragraph-only content from websites, you cannot interact with them.
file_append {{ "path": "...", "content": "..." }}
llm_process {{ "data": "...", "request": "..." }}
    This uses a large language model to process a given set of data for insights.

Task:
{task}

Your goal is to complete your task.
Complete your task as fast as possible.
If you have completed your task, explain.

Respond in this format:

```json
{{
    "is my task complete": true / false,
    "thoughts": "...",
    "reasoning": "...",
    "criticism": "...",
    "action": {{ 
        "tool": "...",
        "args": {{ ... }}
    }} or null
}}
```
"#)));

    loop {
        let results = run_react_action(context, get_agent, task)?;
        let agent = get_agent(context);

        match results {
            ActionResults::Results(results) => {
                println!("{results}");

                agent.llm.message_history.push(Message::User(format!(
r#"Your tool use gave the following result:

{results}

Please decide on your next action (or, determine that the task is complete.)"#
                )));
            },
            ActionResults::TaskComplete(completion_message) => {
                return Ok(completion_message);
            }
        }
    }
}
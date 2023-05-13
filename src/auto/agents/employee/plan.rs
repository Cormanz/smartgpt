use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{Message, AgentInfo, auto::try_parse_json, ScriptValue};

use super::prompt;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Action {
    tool: String,
    args: ScriptValue
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskLabel {
    #[serde(rename = "arguments for trivial and complex")]
    pub arguments: String,
    #[serde(rename = "final conclusion with reasoning")]
    pub conclusion: String,
    #[serde(rename = "exact label")]
    pub label: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskBreakdown {
    pub steps: Vec<String>
}

pub fn truncate(task: &str) -> String {
    if task.len() > 25 {
        let truncated = task
            .chars()
            .take(25)
            .map(|el| el.to_string())
            .collect::<Vec<_>>()
            .join("");
        format!("{}...", truncated)
    } else {
        task.to_string()
    }
}

pub fn decompose_task(agent: &mut AgentInfo, task: &str) -> Result<TaskBreakdown, Box<dyn Error>> {
    agent.llm.clear_history();

    agent.llm.message_history.push(Message::User(format!(r#"
Task:
{task}

Create a simple, surface-level plan to complete the task.
There should be roughly three very simple steps.

Reply in this JSON format:

{{
    "steps": []
}}
"#)));
    try_parse_json(&agent.llm, 2, Some(400))
        .map(|res| {
            agent.llm.message_history.push(Message::Assistant(res.raw));
            res.data
        })
}
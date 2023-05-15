use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{AgentInfo, CommandContext, Message, auto::try_parse_json};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RefineInfo {
    pub expert: String,
    #[serde(rename = "ideal outcome, one paragraph")]
    pub outcome: String,
    #[serde(rename = "refined task")]
    pub task: String,
}

pub fn refine(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str
) -> Result<RefineInfo, Box<dyn Error>> {
    let agent = get_agent(context);
    agent.llm.clear_history();
    agent.llm.message_history.push(Message::User(format!(
r#"
Task:
{task}

First, tell me about a subject matter expert in the task.
Then, write a full paragraph explaining the ideal outcome of the task, as if you were that expert.
Then, use that to write a refined, more detailed version of the original task.

Respond in this exact JSON format:

```json
{{
    "expert": "...",
    "ideal outcome, one paragraph": "...",
    "refined task": "..."
}}
```
"#
    )));

    Ok(try_parse_json(&agent.llm, 2, Some(400))?.data)
}
use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{ProgramInfo, Message};

use super::try_parse_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Classification {
    #[serde(rename = "thoughts on how to classify it")]
    thoughts: String,
    
    #[serde(rename = "message classification")]
    classification: String,
}

pub fn is_task(program: &mut ProgramInfo, task: &str) -> Result<bool, Box<dyn Error>> {
    let ProgramInfo { 
        context, plugins, personality,
        disabled_commands, .. 
    } = program;
    let mut context = context.lock().unwrap();
    
    context.agents.fast.llm.prompt.clear();
    context.agents.fast.llm.message_history.clear();
    
    context.agents.fast.llm.prompt.push(Message::Assistant(format!(r#"
Given a message respond with one of the following.

"conversational": A conversational message
"task": A task or request
"#)));


    context.agents.fast.llm.message_history.push(Message::User(format!(r#"
Respond in this format:

```json
{{
    "thoughts on how to classify it": "...",
    "message classification": "..."
}}
```"#)));

    context.agents.fast.llm.message_history.push(Message::User(format!(
        "Request to Classify: {task}"
    )));

    let classification = try_parse_json::<Classification>(&context.agents.fast.llm, 2, Some(250))?;
        
    Ok(classification.data.classification == "task")
}
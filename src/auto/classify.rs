use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{Message, ProgramInfo};

use super::try_parse_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Classification {
    #[serde(rename = "thoughts on how to classify it")]
    thoughts: String,

    #[serde(rename = "message classification")]
    classification: String,
}

pub fn is_task(program: &mut ProgramInfo, task: &str) -> Result<bool, Box<dyn Error>> {
    let ProgramInfo { context, .. } = program;
    let mut context = context.lock().unwrap();

    context.agents.fast.llm.prompt.clear();
    context.agents.fast.llm.message_history.clear();

    context
        .agents
        .fast
        .llm
        .prompt
        .push(Message::Assistant(String::from(
            "Respond with either 'conversational' for a chat message or 'task' for a request.",
        )));

    context
        .agents
        .fast
        .llm
        .message_history
        .push(Message::User(String::from(
            r#"Respond in JSON:
{{
"thoughts on how to classify it": "<thoughts_here>",
"message classification": "<classification_here>"
}}"#,
        )));

    context
        .agents
        .fast
        .llm
        .message_history
        .push(Message::User(format!("Request to Classify: {task}")));

    let classification = try_parse_json::<Classification>(&context.agents.fast.llm, 2, Some(250))?;

    Ok(classification.data.classification == "task")
}

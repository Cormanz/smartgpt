use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{AgentInfo, Message};

use super::try_parse_json;

#[derive(Serialize, Deserialize)]
pub struct Response {
    response: String,
}

pub fn create_runner_prompt() -> String {
    format!("Write a concise response detailing actions, outcomes, and any lasting changes.")
}

pub fn ask_for_responses(agent: &mut AgentInfo) -> Result<String, Box<dyn Error>> {
    agent
        .llm
        .message_history
        .push(Message::User(create_runner_prompt()));

    let response =
        agent
            .llm
            .model
            .get_response_sync(&agent.llm.get_messages(), Some(1000), None)?;

    Ok(response)
}

pub fn create_assistant_prompt(context: &str, request: &str) -> String {
    format!(
        r#"
Considering the task findings and conversation context:

{context}

Respond concisely to: "{request}", using the JSON format below, and include the source of your information in parentheses:

{{
    "response": "<Your Response (Source)>"
}}
"#
    )
}

pub fn ask_for_assistant_response(
    agent: &mut AgentInfo,
    context: &str,
    request: &str,
    token_limit: Option<u16>,
) -> Result<String, Box<dyn Error>> {
    agent
        .llm
        .message_history
        .push(Message::User(create_assistant_prompt(context, request)));

    let response = try_parse_json::<Response>(&agent.llm, 2, token_limit)?
        .data
        .response;

    agent.llm.message_history.pop();

    Ok(response)
}

use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{LLM, Message, AgentInfo};

use super::try_parse_json;

#[derive(Serialize, Deserialize)]
pub struct Response {
    response: String
}

pub fn create_runner_prompt() -> String {
    format!(
r#"Now, please write a response back to the user. Tell the user, in detail, everything you did, the outcome, and any permanent changes that were carried out."#)
}

pub fn ask_for_responses(agent: &mut AgentInfo) -> Result<String, Box<dyn Error>> {
    agent.llm.message_history.push(Message::User(create_runner_prompt()));

    let response = agent.llm.model.get_response_sync(
        &agent.llm.get_messages(), Some(1000), None
    )?;

    Ok(response)
}

pub fn create_assistant_prompt(context: &str, request: &str) -> String {
    format!(
r#"Given the findings from your task, and this conversation context:

{context}

Write a response back to the user. The original message they sent was: "{request}"
Reply in this JSON format: 

{{
    "response": "..."
}}

Respond in that exact JSON format exactly.

Provide a response as an assistant to the initial request in the above format.
Make sure you include where you got the information from in your response, in parantheses."#)
}

pub fn ask_for_assistant_response(agent: &mut AgentInfo, context: &str, request: &str) -> Result<String, Box<dyn Error>> {
    agent.llm.message_history.push(Message::User(create_assistant_prompt(context, request)));

    let response = try_parse_json::<Response>(&agent.llm, 2, Some(200))?.data.response;

    agent.llm.message_history.pop();

    Ok(response)
}
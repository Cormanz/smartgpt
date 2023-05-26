use std::error::Error;


use serde::{Serialize};

use crate::{CommandContext, AgentInfo, Message, auto::{run::Action}};

use super::use_tool;

pub enum ActionResults {
    TaskComplete(String),
    Results(String)
}

pub fn log_yaml<T: Serialize>(data: &T) -> Result<(), Box<dyn Error>> {
    println!("{}", serde_yaml::to_string(&data)?);

    Ok(())
}

pub fn explain_results(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo
) -> Result<String, Box<dyn Error>> {
    let agent = get_agent(context);
   agent.llm.message_history.push(Message::System(format!(
"Now that you have finished your task, write a detailed, readable and simple Markdown response.
Your response should be easily understandable for a human, and convey all information in an accessible format.
Ensure that sources are linked in the Markdown representation.
Respond in exact plaintext; no JSON.
Keep your response at four paragraphs or less."
    )));

    agent.llm.model.get_response_sync(&agent.llm.get_messages(), Some(600), None)
}

pub fn run_react_action(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    _task: &str,
    action: Option<Action>
) -> Result<ActionResults, Box<dyn Error>> {
    match action {
        Some(action) => {
            Ok(if action.tool == "done" {
                ActionResults::TaskComplete(
                    explain_results(context, &get_agent)?
                )
            } else {
                ActionResults::Results(
                    use_tool(context, &|context| &mut context.agents.fast, action)?
                )
            })
        }
        None => {
            Ok(ActionResults::TaskComplete(
                explain_results(context, &get_agent)?
            ))
        }
    }
}
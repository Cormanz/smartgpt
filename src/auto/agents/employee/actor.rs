use std::error::Error;

use serde::{de::DeserializeOwned, Serialize};

use crate::{AgentInfo, CommandContext, auto::run::{run_command, run_action_sync, Action}, Message};

pub fn use_tool(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    action: Action,
) -> Result<String, Box<dyn Error>> {
    let out = run_action_sync(context, action)?;

    let agent = get_agent(context);
    agent.llm.clear_history();
    agent.llm.message_history.push(Message::System(format!(
"Convert the information provided by the tool below into a readable and simple Markdown representation.
Include all URLs needed."
    )));
    agent.llm.message_history.push(Message::User(out));

    agent.llm.model.get_response_sync(&agent.llm.get_messages(), Some(300), None)
}
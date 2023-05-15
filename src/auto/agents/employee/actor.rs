use std::error::Error;

use colored::Colorize;
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
"Rewrite the information provided by the tool below into a readable and simple Markdown representation.
Ensure all of the information is included.

Include all links and URLs to sources if they are included in the tool output!
Format sources like [Source](URL).

Keep your response at one or two paragraphs."
    )));
    agent.llm.message_history.push(Message::User(out));
    
    println!("{}", "Processing Command Output...".yellow());

    agent.llm.model.get_response_sync(&agent.llm.get_messages(), Some(600), None)
}
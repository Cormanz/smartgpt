use std::error::Error;

use colored::Colorize;
use serde::{de::DeserializeOwned, Serialize};

use crate::{AgentInfo, CommandContext, auto::run::{run_command, run_action_sync, Action}, Message};

pub fn use_tool(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    action: Action,
) -> Result<String, Box<dyn Error>> {
    let out = run_action_sync(context, action.clone())?;

    let agent = get_agent(context);
    agent.llm.clear_history();

    if action.tool == "google_search" {
        agent.llm.message_history.push(Message::System(format!(
"Rewrite the information provided by the google search into this format:

### [WEBSITE NAME](WEBSITE URL)
[WEBSITE SNIPPET]

Ensure it is readable and compressed."
        )));
    } else if action.tool == "browse_url" {
        return Ok(out);
    } else {
        agent.llm.message_history.push(Message::System(format!(
"Rewrite the information provided by the tool below into a readable and simple Markdown representation.
Ensure all of the information is included.
Keep your response at one or two paragraphs.
Remember that if returns 'null', the command was successful and just has no output."
        )));
    }

    agent.llm.message_history.push(Message::User(out));
    
    println!("{}", "Processing Command Output...".yellow());

    agent.llm.model.get_response_sync(&agent.llm.get_messages(), Some(600), None)
}
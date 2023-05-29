use std::error::Error;




use crate::{AgentInfo, CommandContext, auto::run::{run_action_sync, Action}};

pub fn use_tool(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    action: Action,
) -> Result<String, Box<dyn Error>> {
    let out = run_action_sync(context, action.clone())?;

    let agent = get_agent(context);
    agent.llm.clear_history();

    return Ok(out);
}
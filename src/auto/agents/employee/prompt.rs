use crate::{AgentInfo, Message, auto::try_parse_json};

pub fn prompt(agent: &mut AgentInfo, personality: &str) {
    agent.llm.clear_history();
    
    agent.llm.prompt.push(Message::System(format!(
r#"
"Role:
{personality}

You are Agent PLQW09.
You must complete the task assigned to you.
"#
    )));
}

pub fn init_agent(agent: &mut AgentInfo, personality: &str) {
    prompt(agent, personality);
    
    agent.llm.prompt.push(Message::User(format!(
r#"
Tools:
google_search {{ "query": "..." }}
wolfram {{ "query": "solve ..." }}
    Use pure mathematical equations, don't give wolfram any additional context
browse_url {{ "url": "..." }}
    You can only read paragraph-only content from websites, you cannot interact with them.
file_append {{ "path": "...", "content": "..." }}
think_myself {{ "solution": "..." }}
    The "solution" should be a complete answer to the task. 
"#
    )));
}
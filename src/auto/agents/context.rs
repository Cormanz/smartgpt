use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{AgentInfo, CommandContext, Message, auto::try_parse_yaml};

use super::employee::log_yaml;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CanAnswerInfo {
    pub thoughts: String,
    #[serde[rename = "enough information to answer request"]] pub can_answer: bool
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MissingInfo {
    #[serde(rename = "missing information")] pub information: Vec<String>
}

pub fn request_context(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    request: &str
) -> Result<Option<Vec<String>>, Box<dyn Error>> {
    let agent = get_agent(context);
    agent.llm.clear_history();

    agent.llm.message_history.push(Message::User(format!(
"Request: {request}

Do you have enough information for it to be possible for you to answer the request?

Reply in this YAML format:
```yml
thoughts: thoughts
enough information to answer request: true / false
```
```"
    )));

    let can_answer_info = try_parse_yaml::<CanAnswerInfo>(&agent.llm, 2, Some(400), Some(0.0))?;
    agent.llm.message_history.push(Message::User(can_answer_info.raw));
    let can_answer_info = can_answer_info.data;
    log_yaml(&can_answer_info);
    
    if can_answer_info.can_answer {
        return Ok(None);
    }

    agent.llm.message_history.push(Message::User(format!(
"List all missing information that would be needed to answer the request. Keep this list as minimal as possible.

Reply in this YAML format:
```yml
missing information:
- A
```"
    )));

    let missing_info = try_parse_yaml::<MissingInfo>(&agent.llm, 2, Some(400), Some(0.3))?.data;
    log_yaml(&missing_info);

    Ok(Some(
        missing_info.information
    ))
}
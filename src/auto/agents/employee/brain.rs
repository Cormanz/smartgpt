use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{try_parse_yaml, agents::employee::log_yaml}};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitialPlan {
    pub steps: Vec<String>
}

pub fn run_brain_agent(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str
) -> Result<String, Box<dyn Error>> {
    let agent = get_agent(context);

    agent.llm.prompt.push(Message::User(format!(r#"
Given this task: {task}

Plan how the task could be completed, step by step.
Your last step will be giving the response back to the user.

Respond in this YML format:

```yml
steps:
- A
- B
```
    "#)));   

    let plan = try_parse_yaml::<InitialPlan>(&agent.llm, 2, Some(300))?;
    agent.llm.prompt.push(Message::Assistant(plan.raw));
    let plan = plan.data;

    log_yaml(&plan)?;

    panic!("E");
}
use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{try_parse_yaml, agents::employee::{log_yaml, run_method_agent}}};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BrainThoughts {
    pub thoughts: String,
    pub reasoning: String,
    pub action: Option<String>,
    pub done: Option<String>
}

pub fn run_brain_agent(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str
) -> Result<String, Box<dyn Error>> {
    let agent = get_agent(context);

    agent.llm.prompt.push(Message::User(format!(r#"
Given this task: {task}

Your goal is to complete the task by running actions.
Each action will help you complete the task.

Focus on using thoughts, reasoning, and self-criticism to complete your goals.

You will give an instruction to an agent powered by a large language model, with access to external tools.
Keep your instructions very simple.

Respond in this exact YML format:
```yml
thoughts: thoughts
reasoning: reasoning
action: simple instruction in one natural sentence
done: null # `null` because you are still working
```
"#).trim().to_string()));

    let thoughts = try_parse_yaml::<BrainThoughts>(&agent.llm, 2, Some(1000), Some(0.5))?;
    agent.llm.message_history.push(Message::Assistant(thoughts.raw));
    let thoughts = thoughts.data;  

    log_yaml(&thoughts)?;

    drop(agent);
    let mut response = run_method_agent(context, &|ctx| &mut ctx.agents.react, &thoughts.action.unwrap())?;

    loop {
        let agent = get_agent(context);
        agent.llm.message_history.push(Message::User(format!(r#"
Your agent gave back this response:
{response}

Please give another instruction, or give the final response to the user.
Respond in this exact YML format:
```yml
thoughts: thoughts
reasoning: reasoning
action: simple instruction in one natural sentence # if you are done with your task, put `null`
done: final response for user # if you are still working on your task, put `null`
```
        "#).trim().to_string()));

        let thoughts = try_parse_yaml::<BrainThoughts>(&agent.llm, 2, Some(1000), Some(0.5))?;
        agent.llm.message_history.push(Message::Assistant(thoughts.raw));
        let thoughts = thoughts.data; 

        log_yaml(&thoughts)?;

        if let Some(done) = thoughts.done {
            return Ok(done);
        }

        response = run_method_agent(context, &|ctx| &mut ctx.agents.react, &thoughts.action.unwrap())?; 
    }

    panic!("E");
}
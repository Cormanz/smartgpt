use std::{error::Error, ops::Deref};

use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{try_parse_yaml, agents::employee::{log_yaml, run_method_agent}}, ScriptValue, SelfThoughts};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BrainThoughts {
    pub thoughts: String,
    pub reasoning: String,
    pub decision: Decision
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BrainstormArgs {
    pub lines: Vec<String>   
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActionArgs {
    pub instruction: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FinalResponseArgs {
    pub response: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Decision {
    #[serde(rename = "type")] pub decision_type: String,
    pub args: ScriptValue
}

pub fn get_response(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    thoughts: &BrainThoughts
) -> Result<String, Box<dyn Error>> {
    match thoughts.decision.decision_type.deref() {
        "action" => {
            let ActionArgs { instruction } = thoughts.decision.args.parse()?;
            run_method_agent(context, get_agent, &instruction, None)
        },
        "brainstorm" => {
            Ok(format!("Successfully brainstormed."))
        }
        _ => {
            panic!("Unknown Decision Type");
        }
    }
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

You make a decision. Here are the types of decisions alongside their `args` schema:

action {{ "instruction": "Natural language instruction" }} - Delegate a task to the Agent. Keep it simple. 
brainstorm {{ "lines": [] }} - Brainstorm an idea, or generate a response based on the information given yourself.
final_response {{ "response": "response" }} - Give a response to the user.

You may only have one thought.

Respond in this exact YML format:
```yml
thoughts: thoughts
reasoning: reasoning
decision:
    type: decision type
    args: {{ ... }}
```
"#).trim().to_string()));

    let thoughts = try_parse_yaml::<BrainThoughts>(&agent.llm, 2, Some(1000), Some(0.5))?;
    agent.llm.message_history.push(Message::Assistant(thoughts.raw));
    let thoughts = thoughts.data;  

    log_yaml(&thoughts)?;

    drop(agent);
    let mut response = get_response(context, &|ctx| &mut ctx.agents.react, &thoughts)?;
    
    loop {
        let agent = get_agent(context);
        agent.llm.message_history.push(Message::User(format!(r#"
Your previous request gave back the response:
{response}

You may now make another decision, either `action`, `brainstorm`, or `final_response`.
Try to use `thoughts` to think about what your previous response gave you, your long-term ideas, and where to go next.

```yml
thoughts: thoughts
reasoning: reasoning
decision:
    type: decision type
    args: {{ ... }}
```
        "#).trim().to_string()));

        let thoughts = try_parse_yaml::<BrainThoughts>(&agent.llm, 2, Some(1000), Some(0.5))?;
        agent.llm.message_history.push(Message::Assistant(thoughts.raw));
        let thoughts = thoughts.data; 

        log_yaml(&thoughts)?;

        response = get_response(context, &|ctx| &mut ctx.agents.react, &thoughts)?;
    }

    panic!("E");
}
use std::{error::Error, ops::Deref};
use colored::Colorize;
use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{try_parse_yaml, agents::{employee::{log_yaml, run_method_agent}}}, ScriptValue};

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
    pub instruction: String,
    pub assets: Vec<String>
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssetInfo {
    pub assets: Vec<String>
}

pub fn get_response(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    thoughts: &BrainThoughts,
    personality: &str
) -> Result<String, Box<dyn Error>> {
    match thoughts.decision.decision_type.deref() {
        "spawn_agent" => {
            let ActionArgs { instruction, assets } = thoughts.decision.args.parse()?;

            let mut data: Option<String> = None;

            if assets.len() > 0 {
                data = Some(
                    assets.iter()
                    .map(|el| format!("{el}: {}", context.assets[el]))
                    .collect::<Vec<_>>()
                    .join("\n")
                );
            }

            let out = run_method_agent(context, get_agent, &instruction, data, personality)?;
            println!("\n{out}\n");
            Ok(out)
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
    task: &str,
    personality: &str
) -> Result<String, Box<dyn Error>> {
    let agent = get_agent(context);
    
    agent.llm.prompt.push(Message::System(format!("Personality:\n{personality}")));

    agent.llm.prompt.push(Message::User(format!(r#"
Here is the task given by the user:
{task}

Your goal is to appropiately complete the task by spawning agents.
Focus on using thoughts, reasoning, and self-criticism to complete your goals.

You make a decision. Here are the types of decisions alongside their `args` schema:

spawn_agent {{ "instruction": "Natural language instruction", "assets": [ "asset_name" ] }} - Delegate a task to the Agent. Keep it simple.
brainstorm {{ "lines": [] }} - Brainstorm an idea, or generate a response based on the information given yourself.
final_response {{ "response": "response" }} - Give a response to the user.

Assets:
No assets.

An agent may save assets, and you can give those assets to new agents. You may only pass in assets in the above list.
Only include one `thoughts`, `reasoning`, `decision`.

Respond in this exact YML format exactly, with every field in order:
```yml
thoughts: thoughts
reasoning: reasoning
decision:
    type: decision type
    args: ...
```
"#).trim().to_string()));

    println!("{}\n", "Dynamic Agent".blue().bold());

    let thoughts = try_parse_yaml::<BrainThoughts>(&agent.llm, 2, Some(1000), Some(0.3))?;
    agent.llm.message_history.push(Message::Assistant(thoughts.raw));
    let thoughts = thoughts.data;  

    log_yaml(&thoughts)?;

    drop(agent);
    let mut response = get_response(context, &|ctx| &mut ctx.agents.react, &thoughts, personality)?;
    
    loop {
        let cloned_assets = context.assets.clone();
        let asset_list = if cloned_assets.len() == 0 {
            format!("No assets.")
        } else {
            cloned_assets
                .keys()
                .map(|asset| asset.clone())
                .collect::<Vec<_>>()
                .join(", ")
        };
        let agent = get_agent(context);

        agent.llm.message_history.push(Message::User(format!(r#"
Your previous request gave back the response:
{response}

You may now make another decision, either `spawn_agent`, `brainstorm`, or `final_response`.
Try to use `thoughts` to think about what your previous response gave you, your long-term ideas, and where to go next.

Assets: 
{asset_list}

You may only provide these assets when spawning agents.

```yml
thoughts: thoughts
reasoning: reasoning
decision:
    type: decision type
    args: {{ ... }}
```
        "#).trim().to_string()));

        println!("{}\n", "Dynamic Agent".blue().bold());

        let thoughts = try_parse_yaml::<BrainThoughts>(&agent.llm, 2, Some(1000), Some(0.5))?;
        agent.llm.message_history.push(Message::Assistant(thoughts.raw));
        let thoughts = thoughts.data; 

        log_yaml(&thoughts)?;

        response = get_response(context, &|ctx| &mut ctx.agents.react, &thoughts, &personality)?;
    }

    panic!("E");
}
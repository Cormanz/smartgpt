use std::{error::Error, ops::Deref, fmt::Display};
use colored::Colorize;
use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{try_parse_json, agents::{employee::{log_yaml, run_method_agent}}}, ScriptValue};

#[derive(Debug, Clone)]
pub struct NoDecisionTypeError(pub String);

impl Display for NoDecisionTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}' is not a valid decision type", self.0)
    }
}

impl Error for NoDecisionTypeError {}

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
    pub subtask: String,
    pub assets: Vec<String>,
    #[serde(rename = "desired_response")] pub desire: String
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DynamicPlan {
    #[serde(rename = "concise plan on how you will complete the task")]
    pub plan: String
}

pub fn get_response(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    get_planner_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    thoughts: &BrainThoughts,
    personality: &str
) -> Result<String, Box<dyn Error>> {
    match thoughts.decision.decision_type.deref() {
        "spawn_agent" => {
            let ActionArgs { subtask: instruction, assets, desire } = thoughts.decision.args.parse()?;

            let mut data: Option<String> = None;

            if assets.len() > 0 {
                data = Some(
                    assets.iter()
                        .map(|el| format!("## Asset `${el}`:\n{}", context.assets[el]))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }

            let out = run_method_agent(context, get_agent, get_planner_agent, &instruction, &desire, data, personality)?;
            println!("\n{out}\n");
            Ok(out)
        },
        "brainstorm" => {
            Ok(format!("Successfully brainstormed."))
        }
        "final_response" => {
            let FinalResponseArgs { response } = thoughts.decision.args.parse()?;

            Ok(response)
        },
        decision_type => {
            return Err(Box::new(NoDecisionTypeError(decision_type.to_string()))) 
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
This is your task:
{task}

Make a concise, one-sentence plan on you can complete this task.
Remember that you have access to external tools, so you can do any task.

Respond in this JSON format:
```json
{{
    "concise plan on how you will complete the task": "plan"
}}
```
"#).trim().to_string()));

    println!("{}\n", "Dynamic Agent".blue().bold());

    let plan = try_parse_json::<DynamicPlan>(&agent.llm, 2, Some(1000), Some(0.3))?;
    agent.llm.message_history.push(Message::Assistant(plan.raw));
    let plan = plan.data;  

    log_yaml(&plan)?;

    agent.llm.message_history.push(Message::User(format!(r#"
Your goal is to complete the task by spawning agents to complete smaller subtasks.
Focus on using thoughts, reasoning, and self-criticism to complete your goals.

You make a decision. Here are the types of decisions alongside their `args` schema:

spawn_agent {{ "subtask": "subtask in natural language with all context and details", "assets": [ "asset_name" ], "desired_response": "all specific information desired" }} - Delegate a task to the Agent. Keep it simple.
brainstorm {{ "lines": [ "line 1", "line 2" ] }} - Brainstorm an idea, or generate a response based on the information given yourself.
final_response {{ "response": "response" }} - Give a response to the user.

Assets:
No assets.

As you have no assets, you must pass "assets" as [] when spawning an agent.

Ensure you adhere to your plan:
{}

You should try to spawn agents to complete your task.

Only include one `thoughts`, `reasoning`, `decision`.

Respond in this exact JSON format exactly, with every field in order:
```json
{{
    "thoughts": "thoughts",
    "reasoning": "reasoning",
    "decision": {{
        "type": "decision type",
        "args": "..."
    }}
}}
```"#, plan.plan)));

    println!("{}\n", "Dynamic Agent".blue().bold());

    let thoughts = try_parse_json::<BrainThoughts>(&agent.llm, 2, Some(1000), Some(0.3))?;
    agent.llm.message_history.push(Message::Assistant(thoughts.raw));
    let thoughts = thoughts.data;  

    log_yaml(&thoughts)?;

    drop(agent);
    let mut response = get_response(
        context, 
        &|ctx| &mut ctx.agents.static_agent, 
        &|ctx| &mut ctx.agents.planner, 
        &thoughts, 
        &personality
    )?;

    if thoughts.decision.decision_type == "final_response" {
        return Ok(response);
    }
    
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

```json
{{
    "thoughts": "thoughts",
    "reasoning": "reasoning",
    "decision": {{
        "type": "decision type",
        "args": "..."
    }}
}}
```
        "#).trim().to_string()));

        println!("{}\n", "Dynamic Agent".blue().bold());

        let thoughts = try_parse_json::<BrainThoughts>(&agent.llm, 2, Some(1000), Some(0.5))?;
        agent.llm.message_history.push(Message::Assistant(thoughts.raw));
        let thoughts = thoughts.data; 

        log_yaml(&thoughts)?;

        response = get_response(
            context, 
            &|ctx| &mut ctx.agents.static_agent, 
            &|ctx| &mut ctx.agents.planner, 
            &thoughts, 
            &personality
        )?;

        if thoughts.decision.decision_type == "final_response" {
            return Ok(response);
        }
    }
}
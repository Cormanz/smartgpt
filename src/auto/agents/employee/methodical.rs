use std::{error::Error, collections::{HashSet}};

use colored::Colorize;
use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{run::Action, try_parse_json, agents::employee::create_tool_list}, Weights, Command};

use super::{log_yaml, use_tool};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalThoughts {
    pub thoughts: String,
    pub action: Action
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalAction {
    pub tool: String,
    pub purpose: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalStep {
    pub idea: String,
    pub action: MethodicalAction
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalPlan {
    pub thoughts: String,
    pub steps: Vec<MethodicalStep>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RevisedMethodicalPlan {
    pub thoughts: String,
    pub solution: String,
    #[serde(rename = "revised remaining steps")]
    pub steps: Vec<MethodicalStep>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FinalResponse {
    pub response: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Memories {
    pub actions: Vec<String>,
    pub observations: Vec<String>
}

pub fn add_memories(agent: &mut AgentInfo) -> Result<(), Box<dyn Error>> {
    agent.llm.message_history.push(Message::User(format!(r#"
Please summarize all important actions you took out.
Please also summarize all observations of information you have collected.

Be cocnise.

Respond in this JSON format:
```json
{{
    "actions": [
        "what tool you used and why"
    ],
    "observations": [
        "what you learned"
    ]
}}
```
    "#).trim().to_string()));

    let memories = try_parse_json::<Memories>(&agent.llm, 2, Some(700), Some(0.5))?.data;
    log_yaml(&memories)?;

    for memory in memories.actions.iter().chain(memories.observations.iter()) {
        agent.observations.store_memory_sync(&agent.llm, memory)?;
    }

    Ok(())
}

pub fn run_method_agent(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    get_planner_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str,
    desire: &str,
    assets: Option<String>,
    personality: &str
) -> Result<String, Box<dyn Error>> {
    let commands: Vec<&Command> = context.plugins.iter()
        .flat_map(|plugin| &plugin.commands)
        .collect();
    
    let tools = create_tool_list(&commands);
    
    let cloned_assets = context.assets.clone();
    let assets_before: HashSet<&String> = cloned_assets.keys().collect();

    get_agent(context).llm.clear_history();

    let planner = get_planner_agent(context);

    planner.llm.clear_history();

    planner.llm.prompt.push(Message::System(format!(r#"
Personality: 
{personality}
"#).trim().to_string()));

    let observations = planner.observations.get_memories_sync(
        &planner.llm, task, 100, Weights {
            recall: 1.,
            recency: 1.,
            relevance: 1.
        }, 30
    )?;

    let observations = if observations.len() == 0 {
        "None found.".to_string()
    } else {
        observations.iter()
            .map(|obs| format!("- {}", obs.content))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let data = assets.unwrap_or(format!("No assets."));

    println!("{} {}\n", "Static Agent".yellow().bold(), "| Plan".white());

    planner.llm.prompt.push(Message::User(format!(r#"
{tools}

You have been given these tools.
You may use these tools, and only these tools.

Here is your new task:
{task}

Here is a list of your memories:
{observations}

Here is a list of assets previously saved:
{data}

Create a list of steps of what you need to do and which tool you will use.
Only use one tool for each step.

Your goal is to give a response with the following information:
{desire}

You must save that precise information through assets.

Do not specify arguments.
Do not "repeat steps".

Keep your plan at as low steps as possible.

Use `save_asset` for all information you want to save.
You can build off of previous assets if needed.

Respond in this JSON format:
```json
{{
    "thoughts": "thoughts regarding steps and assets",
    "steps": [
        {{
            "idea": "idea",
            "action": {{
                "tool": "a single tool name",
                "purpose": "purpose"
            }}
        }}
    ]
}}
```

Be concise!
"#).trim().to_string()));

    let plan = try_parse_json::<MethodicalPlan>(&planner.llm, 2, Some(600), Some(0.3))?;
    planner.llm.message_history.push(Message::Assistant(plan.raw));
    let plan = plan.data;
    log_yaml(&plan)?;

    let prompt = planner.llm.prompt.clone();
    let message_history = planner.llm.message_history.clone();

    drop(planner);

    let agent = get_agent(context);
    agent.llm.prompt = prompt;
    agent.llm.message_history = message_history;

    for (_ind, step) in plan.steps.iter().enumerate() {
        let agent = get_agent(context);
        
        println!();
        println!("{} {}\n", "Static Agent".yellow().bold(), "| Selecting Step".white());

        let step_text = serde_yaml::to_string(&step)?;
        println!("{}", step_text);
        
        println!();
        println!("{} {}\n", "Static Agent".yellow().bold(), "| Running Step".white());

        agent.llm.message_history.push(Message::User(format!(r#"
Now you will carry out the next step: 
{step_text}

You must carry out this step with one entire action.
Include ALL information.

Assets:
No assets.

Respond in this JSON format:
```json
{{
    "thoughts": "thoughts",
    "action": {{
        "tool": "tool",
        "args": {{}}
    }}
}}
thoughts: thoughts
action:
    tool: tool
    args: {{}}
```
"#).trim().to_string()));

        let thoughts = try_parse_json::<MethodicalThoughts>(&agent.llm, 2, Some(1000), Some(0.5))?;
        agent.llm.message_history.push(Message::Assistant(thoughts.raw));
        let thoughts = thoughts.data;

        log_yaml(&thoughts)?;

        drop(agent);

        let out = use_tool(context, &|context| &mut context.agents.fast, thoughts.action.clone());
            
        println!();
        match out {
            Ok(out) => {
                let agent = get_agent(context);

                println!("{out}");
                agent.llm.message_history.push(Message::User(out));

                let tokens = agent.llm.get_tokens_remaining(&agent.llm.get_messages())?;
                if tokens < 1200 {
                    match add_memories(agent) {
                        Ok(_) => {},
                        Err(_) => {
                            agent.llm.crop_to_tokens_remaining(1000)?;
                        }
                    };
                    agent.llm.crop_to_tokens_remaining(2000)?;
                }
            },
            Err(err) => {
                return Err(err);
            }
        }
    }
    
    let agent = get_agent(context);
    add_memories(agent)?;
    
    let cloned_assets = context.assets.clone();
    let assets_after: HashSet<&String> = cloned_assets.keys().collect();

    println!("{assets_before:?}, {assets_after:?}");

    let changed_assets = assets_after.difference(&assets_before)
        .map(|asset| asset.to_string())
        .collect::<Vec<_>>();

    let asset_str = if changed_assets.len() == 0 {
        format!("No assets changed.")
    } else {
        changed_assets .iter()
            .map(|el| format!("## Asset `{el}`\n{}", context.assets[el]))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let resp = format!("Assets:\n\n{}", asset_str);
    

    return Ok(resp);
}
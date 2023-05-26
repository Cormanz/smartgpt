use std::{error::Error, collections::{HashSet}};

use colored::Colorize;
use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{run::Action, try_parse_yaml, agents::employee::create_tool_list}, Weights, Command};

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
Please list all important actions you took out.
Please also list all observations of information you have collected.

Respond in this YML format:
```yml
actions:
- A
- B

observations:
- A
- B
```
    "#).trim().to_string()));

    let memories = try_parse_yaml::<Memories>(&agent.llm, 2, Some(1000), Some(0.5))?.data;
    log_yaml(&memories)?;

    for memory in memories.actions.iter().chain(memories.observations.iter()) {
        agent.observations.store_memory_sync(&agent.llm, memory)?;
    }

    Ok(())
}

pub fn run_method_agent(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str,
    data: Option<String>,
    personality: &str
) -> Result<String, Box<dyn Error>> {
    let commands: Vec<&Command> = context.plugins.iter()
        .flat_map(|plugin| &plugin.commands)
        .collect();
    
    let tools = create_tool_list(&commands);
    
    let cloned_assets = context.assets.clone();
    let assets_before: HashSet<&String> = cloned_assets.keys().collect();

    let agent = get_agent(context);

    agent.llm.clear_history();

    agent.llm.prompt.push(Message::System(format!(r#"
Personality: 
{personality}

{tools}

You have been given these tools.
"#).trim().to_string()));

    let observations = agent.observations.get_memories_sync(
        &agent.llm, task, 100, Weights {
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

    let data = data.unwrap_or(format!("No assets."));

    println!("{} {}\n", "Static Agent".yellow().bold(), "| Plan".white());

    agent.llm.prompt.push(Message::User(format!(r#"
Here is your new task:
{task}

Here is a list of assets for your task:
{data}

Here is a list of your memories:
{observations}

Create a list of steps.
Start with an idea of what you need to do next and what tool could help.
Each step will use one tool.
Then, describe each time you will use the tool.
You will explain the exact purpose of using that specific tool.

Do not specify arguments.
Do not "repeat steps".

Ensure you have as few steps as possible. Do not overcomplicate it.
Save all important output into assets.

Respond in this YML format:
```yml
steps:
- idea: idea
  action:
    tool: precise tool name
    purpose: purpose
```
"#).trim().to_string()));

    let plan = try_parse_yaml::<MethodicalPlan>(&agent.llm, 2, Some(600), Some(0.3))?;
    agent.llm.message_history.push(Message::Assistant(plan.raw));
    let plan = plan.data;
    log_yaml(&plan)?;

    drop(agent);

    for (_ind, step) in plan.steps.iter().enumerate() {
        let agent = get_agent(context);
        let tokens = agent.llm.get_tokens_remaining(&agent.llm.get_messages())?;
        if tokens < 1400 {
            add_memories(agent)?;
            agent.llm.crop_to_tokens_remaining(2600)?;
        }
        
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

Respond in this YML format:
```yml
thoughts: thoughts
action:
    tool: tool
    args: {{}}
```
"#).trim().to_string()));

        let thoughts = try_parse_yaml::<MethodicalThoughts>(&agent.llm, 2, Some(1000), Some(0.5))?;
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
            },
            Err(err) => {
                println!("{err}");

                panic!("Error.");
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
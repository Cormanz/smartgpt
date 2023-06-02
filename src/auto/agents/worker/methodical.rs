use std::{error::Error, collections::{HashSet}};

use colored::Colorize;
use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{run::Action, try_parse_json, agents::{worker::create_tool_list, prompt::{SUMMARIZE_MEMORIES, NoData, PERSONALITY, PersonalityInfo, CREATE_PLAN, CreatePlanInfo, NextStepInfo, NEXT_STEP, SAVE_ASSET, SaveAssetInfo}}, DisallowedAction, StaticUpdate, Update, NamedAsset}, Weights, Tool};

use super::{log_yaml, use_tool};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalThoughts {
    pub thoughts: String,
    pub action: Action
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MethodicalAction {
    #[serde(rename = "resource")]
    Resource {
        name: String,
        question: Option<String>
    },
    #[serde(rename = "action")]
    Action {
        name: String,
        purpose: Option<String>
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalStep {
    pub idea: String,
    pub decision: MethodicalAction
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalAsset {
    pub name: String,
    pub description: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalPlan {
    pub thoughts: String,
    pub steps: Vec<MethodicalStep>,
    pub assets: Vec<MethodicalAsset>
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

pub fn add_memories(
    agent: &mut AgentInfo,
    listen_to_update: &mut impl FnMut(&Update) -> Result<(), Box<dyn Error>>
) -> Result<(), Box<dyn Error>> {
    listen_to_update(&Update::StaticAgent(StaticUpdate::SavingMemories()))?;
    agent.llm.message_history.push(Message::User(
        SUMMARIZE_MEMORIES.fill(NoData)?  
    ));

    let memories = try_parse_json::<Memories>(&agent.llm, 2, Some(700), Some(0.5))?.data;
    listen_to_update(&Update::StaticAgent(StaticUpdate::SavedMemories(memories.clone())))?;

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
    personality: &str,
    allow_action: &mut impl FnMut(&Action) -> Result<(), DisallowedAction>,
    listen_to_update: &mut impl FnMut(&Update) -> Result<(), Box<dyn Error>>
) -> Result<String, Box<dyn Error>> {
    let tools: Vec<&Tool> = context.plugins.iter()
        .flat_map(|plugin| &plugin.tools)
        .collect();
    
    let tools = create_tool_list(&tools);
    
    let cloned_assets = context.assets.clone();
    let assets_before: HashSet<&String> = cloned_assets.keys().collect();

    get_agent(context).llm.clear_history();

    let planner = get_planner_agent(context);

    planner.llm.clear_history();

    planner.llm.prompt.push(Message::System(
        PERSONALITY.fill(PersonalityInfo { personality: personality.to_string() })?
    ));

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

    planner.llm.prompt.push(Message::User(
        CREATE_PLAN.fill(CreatePlanInfo {
            task: task.to_string(),
            observations,
            tools,
            assets: data,
            desire: desire.to_string()
        })?
    ));

    let plan = try_parse_json::<MethodicalPlan>(&planner.llm, 2, Some(600), Some(0.3))?;
    planner.llm.message_history.push(Message::Assistant(plan.raw));
    let plan = plan.data;
    listen_to_update(&Update::StaticAgent(StaticUpdate::Plan(plan.clone())))?;

    let prompt = planner.llm.prompt.clone();
    let message_history = planner.llm.message_history.clone();

    drop(planner);

    let agent = get_agent(context);
    agent.llm.prompt = prompt;
    agent.llm.message_history = message_history;

    for step in plan.steps {
        let agent = get_agent(context);
        let step_text = serde_yaml::to_string(&step)?;
        
        listen_to_update(&Update::StaticAgent(StaticUpdate::SelectedStep(step.clone())))?;

        agent.llm.message_history.push(Message::User(
            NEXT_STEP.fill(NextStepInfo {
                step: step_text
            })?
        ));

        let thoughts = try_parse_json::<MethodicalThoughts>(&agent.llm, 2, Some(1000), Some(0.5))?;
        agent.llm.message_history.push(Message::Assistant(thoughts.raw));
        let thoughts = thoughts.data;

        drop(agent);

        listen_to_update(&Update::StaticAgent(StaticUpdate::Thoughts(thoughts.clone())))?;
        allow_action(&thoughts.action)?;

        let out = use_tool(context, &|context| &mut context.agents.fast, thoughts.action.clone());
            
        match out {
            Ok(out) => {
                let agent = get_agent(context);

                listen_to_update(&Update::StaticAgent(StaticUpdate::ActionResults(out.clone())))?;
                agent.llm.message_history.push(Message::User(out));

                let tokens = agent.llm.get_tokens_remaining(&agent.llm.get_messages())?;
                if tokens < 1200 {
                    match add_memories(agent, listen_to_update) {
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

    let mut changed_assets: Vec<NamedAsset> = vec![];

    for asset in plan.assets {
        listen_to_update(&Update::StaticAgent(StaticUpdate::SelectedAsset(asset.name.clone())))?;

        let agent = get_agent(context);

        let asset_text = serde_yaml::to_string(&asset)?;

        agent.llm.message_history.push(Message::User(
            SAVE_ASSET.fill(SaveAssetInfo { asset: asset_text })?
        ));
    
        let asset_content = agent.llm.model.get_response_sync(&agent.llm.get_messages(), Some(800), Some(0.3))?;
        agent.llm.message_history.pop();

        drop(agent);

        *context.assets
            .entry(asset.name.clone())
            .or_insert(asset_content.clone()) = asset_content.clone();

        let named_asset = NamedAsset(asset.name, asset_content);
        changed_assets.push(named_asset.clone());
        listen_to_update(&Update::StaticAgent(StaticUpdate::AddedAsset(named_asset.clone())))?;
    }

    let agent = get_agent(context);

    add_memories(agent, listen_to_update)?;

    let asset_str = if changed_assets.len() == 0 {
        format!("No assets changed.")
    } else {
        changed_assets .iter()
            .map(|el| format!("## Asset `{}`\n{}", el.0, el.1))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let resp = format!("Assets:\n\n{}", asset_str);
    

    return Ok(resp);
}
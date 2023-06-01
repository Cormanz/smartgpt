use std::{error::Error, ops::Deref, fmt::Display};
use colored::Colorize;
use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{try_parse_json, agents::{worker::{log_yaml, run_method_agent}, prompt::{CONCISE_PLAN, ConcisePlanInfo, PersonalityInfo, PERSONALITY, THOUGHTS, ThoughtInfo, NewThoughtInfo, NEW_THOUGHTS}}, run::Action, DisallowedAction, DynamicUpdate}, ScriptValue};

use super::Update;

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
    personality: &str,
    allow_action: &mut impl FnMut(&Action) -> Result<(), DisallowedAction>,
    listen_to_update: &mut impl FnMut(&Update) -> Result<(), Box<dyn Error>>
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

            let out = run_method_agent(context, get_agent, get_planner_agent, &instruction, &desire, data, personality, allow_action, listen_to_update)?;
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
    personality: &str,
    allow_action: &mut impl FnMut(&Action) -> Result<(), DisallowedAction>,
    listen_to_update: &mut impl FnMut(&Update) -> Result<(), Box<dyn Error>>
) -> Result<String, Box<dyn Error>> {
    let agent = get_agent(context);
    
    agent.llm.prompt.push(Message::System(
        PERSONALITY.fill(PersonalityInfo { personality: personality.to_string() })?
    ));
    agent.llm.prompt.push(Message::User(
        CONCISE_PLAN.fill(ConcisePlanInfo { task: task.to_string() })?
    ));

    let plan = try_parse_json::<DynamicPlan>(&agent.llm, 2, Some(1000), Some(0.3))?;
    agent.llm.message_history.push(Message::Assistant(plan.raw));
    let plan = plan.data;  

    listen_to_update(&Update::DynamicAgent(DynamicUpdate::Plan(plan.plan.clone())))?;

    agent.llm.message_history.push(Message::User(
        THOUGHTS.fill(ThoughtInfo {
            plan: plan.plan,
            assets: "None found.".to_string() 
        })?
    ));

    let thoughts = try_parse_json::<BrainThoughts>(&agent.llm, 2, Some(1000), Some(0.3))?;
    agent.llm.message_history.push(Message::Assistant(thoughts.raw));
    let thoughts = thoughts.data;  

    listen_to_update(&Update::DynamicAgent(DynamicUpdate::Thoughts(thoughts.clone())))?;

    drop(agent);
    let mut response = get_response(
        context, 
        &|ctx| &mut ctx.agents.static_agent, 
        &|ctx| &mut ctx.agents.planner, 
        &thoughts, 
        &personality,
        allow_action,
        listen_to_update
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

        agent.llm.message_history.push(Message::User(
            NEW_THOUGHTS.fill(NewThoughtInfo {
                response: response.to_string(),
                assets: "None found.".to_string() 
            })?
        ));

        let thoughts = try_parse_json::<BrainThoughts>(&agent.llm, 2, Some(1000), Some(0.5))?;
        agent.llm.message_history.push(Message::Assistant(thoughts.raw));
        let thoughts = thoughts.data; 

        listen_to_update(&Update::DynamicAgent(DynamicUpdate::Thoughts(thoughts.clone())))?;

        response = get_response(
            context, 
            &|ctx| &mut ctx.agents.static_agent, 
            &|ctx| &mut ctx.agents.planner, 
            &thoughts, 
            &personality,
            allow_action,
            listen_to_update
        )?;

        if thoughts.decision.decision_type == "final_response" {
            return Ok(response);
        }
    }
}
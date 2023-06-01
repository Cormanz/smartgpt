use std::{collections::HashMap, error::Error, fmt::Display, process, sync::{Mutex, Arc}};

use colored::Colorize;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::{CommandContext, LLM, Plugin, create_browse, create_google, create_filesystem, create_wolfram, create_news, LLMProvider, create_model_chatgpt, Agents, LLMModel, create_model_llama, AgentInfo, MemoryProvider, create_memory_local, create_memory_qdrant, MemorySystem, create_memory_redis, PluginStore, create_brainstorm, SmartGPT};

mod default;
pub use default::*;

#[derive(Debug, Clone)]
pub struct NoLLMError;

impl<'a> Display for NoLLMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cannot load config without a large language model")
    }
}

impl<'a> Error for NoLLMError {}

#[derive(Debug, Clone)]
pub struct NoMemorySystemError;

impl<'a> Display for NoMemorySystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cannot load config without a large language model")
    }
}

impl<'a> Error for NoMemorySystemError {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    pub llm: HashMap<String, Value>,
    pub memory: HashMap<String, Value>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentLLMs {
    #[serde(rename = "static")] static_agent: AgentConfig,
    planner: AgentConfig,
    dynamic: AgentConfig,
    fast: AgentConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub task: String,
    pub personality: String,
    pub agents: AgentLLMs,
    pub plugins: HashMap<String, Value>,
    #[serde(rename = "disabled tools")] pub disabled_tools: Vec<String>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Llm {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "openai key")]
    pub openai_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AutoType {
    #[serde(rename = "runner")] Runner {
        task: String
    }
}

pub fn list_plugins() -> Vec<Plugin> {
    vec![
        create_browse(),
        create_google(),
        create_filesystem(),
        create_wolfram(),
        create_brainstorm(),
        create_news()
    ]
}

pub fn create_llm_providers() -> Vec<Box<dyn LLMProvider>> {
    vec![
        create_model_chatgpt(),
        create_model_llama()
    ]
}

pub fn create_memory_providers() -> Vec<Box<dyn MemoryProvider>> {
    vec![
        create_memory_local(),
        create_memory_qdrant(),
        create_memory_redis()
    ]
}

fn create_llm_model(agent: HashMap<String, Value>) -> Result<Box<dyn LLMModel>, Box<dyn Error>> {
    let (model_name, model_config) = agent.iter().next().ok_or(NoLLMError)?;
    let providers = create_llm_providers();
    let llm_provider = providers.iter()
        .filter(|el| el.is_enabled())
        .find(|el| el.get_name().to_ascii_lowercase() == model_name.to_ascii_lowercase())
        .ok_or(NoLLMError)?;

    Ok(llm_provider.create(model_config.clone())?)
}

fn create_memory_model(agent: HashMap<String, Value>) -> Result<Box<dyn MemorySystem>, Box<dyn Error>> {
    let (model_name, model_config) = agent.iter().next().ok_or(NoLLMError)?;
    let providers = create_memory_providers();
    let memory_provider = providers.iter()
        .filter(|el| el.is_enabled())
        .find(|el| el.get_name().to_ascii_lowercase() == model_name.to_ascii_lowercase())
        .ok_or(NoMemorySystemError)?;

    Ok(memory_provider.create(model_config.clone())?)
}

pub fn create_agent(agent: AgentConfig) -> Result<AgentInfo, Box<dyn Error>> {
    Ok(AgentInfo {
        llm: LLM {
            prompt: vec![],
            message_history: vec![],
            end_prompt: vec![],
            model: create_llm_model(agent.llm)?
        },
        observations: create_memory_model(agent.memory.clone())?,
        reflections: create_memory_model(agent.memory)?
    })
}

pub fn load_config(config: &str) -> Result<(String, SmartGPT), Box<dyn Error>> {
    let config: Config = serde_yaml::from_str(config)?;

    let plugins = list_plugins();
    let mut exit = false;
    for (name, _) in &config.plugins {
        let plugin = plugins.iter().find(|el| el.name.to_ascii_lowercase() == name.to_ascii_lowercase());
        if let None = plugin {
            println!("{}: No plugin named \"{}\".", "Error".red(), name);
            exit = true;
        }
    }
    if exit {
        process::exit(1);
    }

    let mut context = CommandContext {
        assets: HashMap::new(),
        plugin_data: PluginStore::new(),
        plugins: vec![],
        disabled_tools: config.disabled_tools,
        agents: Agents {
            static_agent: create_agent(config.agents.static_agent)?,
            planner: create_agent(config.agents.planner)?,
            dynamic: create_agent(config.agents.dynamic)?,
            fast: create_agent(config.agents.fast)?
        }
    };
    
    for plugin in plugins {
        if let Some(plugin_info) = config.plugins.get(&plugin.name.to_lowercase()) {
            let data = plugin.cycle.create_data(plugin_info.clone());
            if let Some(data) = data {
                context.plugin_data.0.insert(plugin.name.clone(), data);
            }

            context.plugins.push(plugin);
        }
    }

    Ok((
        config.task,
        SmartGPT {
            personality: config.personality,
            context: Arc::new(Mutex::new(context))
        }
    ))
}
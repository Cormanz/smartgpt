use std::{collections::HashMap, error::Error, fmt::Display, process, sync::{Mutex, Arc}};

use colored::Colorize;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use async_openai::Client as OpenAIClient;

use crate::{CommandContext, EndGoals, LLM, ChatGPT, Plugin, create_browse, create_google, create_filesystem, create_shutdown, create_wolfram, create_chatgpt, create_news, create_wikipedia, create_none, LLMProvider, create_model_chatgpt, Agents, LLMModel, create_model_llama, AgentInfo, MemoryProvider, create_memory_local, MemorySystem};

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
    managers: Vec<AgentConfig>,
    employee: AgentConfig,
    fast: AgentConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(rename = "type")]
    pub auto_type: AutoType,
    pub personality: String,
    pub agents: AgentLLMs,
    pub plugins: HashMap<String, Value>,
    #[serde(rename = "disabled commands")] pub disabled_commands: Vec<String>
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
    },
    #[serde(rename = "assistant")] Assistant
}

pub struct ProgramInfo {
    pub personality: String,
    pub auto_type: AutoType,
    pub plugins: Vec<Plugin>,
    pub context: Arc<Mutex<CommandContext>>,
    pub disabled_commands: Vec<String>
}

pub fn list_plugins() -> Vec<Plugin> {
    vec![
        create_browse(),
        create_google(),
        create_filesystem(),
        create_shutdown(),
        create_wolfram(),
        create_chatgpt(),
        create_news(),
        create_wikipedia(),
        create_none()
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
        create_memory_local()
    ]
}

pub fn create_llm_model(agent: HashMap<String, Value>) -> Result<Box<dyn LLMModel>, Box<dyn Error>> {
    let (model_name, model_config) = agent.iter().next().ok_or(NoLLMError)?;
    let providers = create_llm_providers();
    let llm_provider = providers.iter()
        .filter(|el| el.is_enabled())
        .find(|el| el.get_name().to_ascii_lowercase() == model_name.to_ascii_lowercase())
        .ok_or(NoLLMError)?;

    Ok(llm_provider.create(model_config.clone())?)
}

pub fn create_memory_model(agent: HashMap<String, Value>) -> Result<Box<dyn MemorySystem>, Box<dyn Error>> {
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

pub fn load_config(config: &str) -> Result<ProgramInfo, Box<dyn Error>> {
    let config: Config = serde_yaml::from_str(config)?;

    let mut context = CommandContext {
        auto_type: config.auto_type.clone(),
        command_out: vec![],
        variables: HashMap::new(),
        plugin_data: crate::PluginStore(HashMap::new()),
        agents: Agents {
            managers: config.agents.managers.iter().map(|el| create_agent(el.clone())).collect::<Result<_, _>>()?,
            employee: create_agent(config.agents.employee)?,
            fast: create_agent(config.agents.fast)?
        }
    };

    let mut used_plugins: Vec<Plugin> = vec![];
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
    
    for plugin in plugins {
        if let Some(plugin_info) = config.plugins.get(&plugin.name.to_lowercase()) {
            let data = plugin.cycle.create_data(plugin_info.clone());
            if let Some(data) = data {
                context.plugin_data.0.insert(plugin.name.clone(), data);
            }

            used_plugins.push(plugin);
        }
    }

    Ok(ProgramInfo {
        personality: config.personality,
        auto_type: config.auto_type.clone(),
        plugins: used_plugins,
        context: Arc::new(Mutex::new(context)),
        disabled_commands: config.disabled_commands
    })
}
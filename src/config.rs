use std::{collections::HashMap, error::Error, fmt::Display, ascii::AsciiExt, process};

use colored::Colorize;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use async_openai::Client as OpenAIClient;

use crate::{CommandContext, create_tokenizer, EndGoals, LLM, ChatGPT, Plugin, create_browse, create_google, create_filesystem, create_shutdown, create_memory, create_wolfram, create_chatgpt, create_news, create_wikipedia, create_none, LLMProvider, create_model_chatgpt, Agents, LLMModel};

#[derive(Debug, Clone)]
pub struct NoLLMError;

impl<'a> Display for NoLLMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cannot load config without an LLM")
    }
}

impl<'a> Error for NoLLMError {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentLLMs {
    manager: HashMap<String, Value>,
    boss: HashMap<String, Value>,
    employee: HashMap<String, Value>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub name: String,
    pub role: String,
    pub task: String,
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

pub struct ProgramInfo {
    pub name: String,
    pub role: String,
    pub task: String,
    pub plugins: Vec<Plugin>,
    pub context: CommandContext,
    pub disabled_commands: Vec<String>
}

pub fn list_plugins() -> Vec<Plugin> {
    vec![
        create_browse(),
        create_google(),
        create_filesystem(),
        create_memory(),
        create_shutdown(),
        create_wolfram(),
        create_chatgpt(),
        create_news(),
        create_wikipedia(),
        create_none()
    ]
}

pub fn create_providers() -> Vec<Box<dyn LLMProvider>> {
    vec![
        create_model_chatgpt()
    ]
}

pub async fn create_model(agent: HashMap<String, Value>) -> Result<Box<dyn LLMModel>, Box<dyn Error>> {
    let (model_name, model_config) = agent.iter().next().ok_or(NoLLMError)?;
    let providers = create_providers();
    let model = providers.iter()
        .find(|el| el.get_name().to_ascii_lowercase() == model_name.to_ascii_lowercase())
        .ok_or(NoLLMError)?;

    Ok(model.create(model_config.clone()).await?)
}

pub async fn load_config(config: &str) -> Result<ProgramInfo, Box<dyn Error>> {
    let config: Config = serde_yaml::from_str(config)?;
    let manager = create_model(config.agents.manager).await?;
    let boss = create_model(config.agents.boss).await?;
    let employee = create_model(config.agents.employee).await?;

    let mut context = CommandContext {
        task: config.task.clone(),
        tokenizer: create_tokenizer(),
        command_out: vec![],
        variables: HashMap::new(),
        plugin_data: crate::PluginStore(HashMap::new()),
        agents: Agents {
            manager: LLM {
                prompt: vec![],
                message_history: vec![],
                model: manager
            },
            boss: LLM {
                prompt: vec![],
                message_history: vec![],
                model: boss
            },
            employee: LLM {
                prompt: vec![],
                message_history: vec![],
                model: employee
            }
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
            let data = plugin.cycle.create_data(plugin_info.clone()).await;
            if let Some(data) = data {
                context.plugin_data.0.insert(plugin.name.clone(), data);
            }

            used_plugins.push(plugin);
        }
    }

    Ok(ProgramInfo {
        name: config.name,
        role: config.role,
        task: config.task.clone(),
        plugins: used_plugins,
        context,
        disabled_commands: config.disabled_commands
    })
}
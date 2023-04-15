use std::{collections::HashMap, error::Error, fmt::Display, future::Future, pin::Pin, any::Any};

use async_openai::{Client as OpenAIClient, types::ChatCompletionRequestMessage};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned, __private::de};
use serde_json::Value;
use tokenizers::Tokenizer;

#[derive(Debug, Clone)]
pub struct PluginDataNoInvoke(pub String, pub String);

impl<'a> Display for PluginDataNoInvoke {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the '{}' plugin's data does not have '{}' invocation.", self.0, self.1)
    }
}

impl<'a> Error for PluginDataNoInvoke {}

#[derive(Debug, Clone)]
pub struct CommandNoArgError<'a>(pub &'a str, pub &'a str);

impl<'a> Display for CommandNoArgError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the '{}' command did not receive the '{}' argument.", self.0, self.1)
    }
}

impl<'a> Error for CommandNoArgError<'a> {}

use crate::{LLMResponse, LLM, ScriptValue};

#[async_trait]
pub trait PluginData: Any + Send + Sync {
    async fn apply(&mut self, name: &str, info: Value) -> Result<Value, Box<dyn Error>>;
}

pub struct PluginStore(pub HashMap<String, Box<dyn PluginData>>);

pub struct EndGoals {
    pub end_goal: usize,
    pub end_goals: Vec<String>
}

impl EndGoals {
    pub fn get(&self) -> String {
        self.end_goals[self.end_goal].clone()
    }
}

pub struct CommandContext {
    pub end_goals: EndGoals,
    pub tokenizer: Tokenizer,
    pub plugin_data: PluginStore,
    pub llm: LLM,
    pub variables: HashMap<String, ScriptValue>
}


#[derive(Debug, Clone)]
pub struct NoPluginDataError(pub String);

impl Display for NoPluginDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "could not find plugin data for plugin \"{}\"", self.0)
    }
}

impl Error for NoPluginDataError {}

impl PluginStore {
    pub fn get_data(&mut self, plugin: &str) -> Result<&mut Box<dyn PluginData>, Box<dyn Error>> {
        let plugin = plugin.to_string();
        let error = NoPluginDataError(plugin.clone());
        self.0.get_mut(&plugin).ok_or(Box::new(error))
    }   
}

pub async fn invoke<T : DeserializeOwned>(
    data: &mut Box<dyn PluginData>, name: &str, info: impl Serialize
) -> Result<T, Box<dyn Error>> {
    let info = serde_json::to_value(info)?;
    let value =  data.apply(name, info).await?;
    let out = serde_json::from_value(value)?;
    Ok(out)
}

#[async_trait]
pub trait CommandImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>>;
}

#[async_trait]
pub trait PluginCycle {
    async fn create_context(&self, context: &mut CommandContext, previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>>;

    async fn apply_removed_response(&self, context: &mut CommandContext, response: &LLMResponse, cmd_output: &str, previous_response: bool) -> Result<(), Box<dyn Error>>;

    async fn create_data(&self, value: Value) -> Option<Box<dyn PluginData>>;
}

pub struct EmptyCycle;

#[async_trait]
impl PluginCycle for EmptyCycle {
    async fn create_context(&self, context: &mut CommandContext, previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
        Ok(None)
    }

    async fn apply_removed_response(&self, context: &mut CommandContext, response: &LLMResponse, cmd_output: &str, previous_response: bool) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn create_data(&self, _: Value) -> Option<Box<dyn PluginData>> {
        None
    }
}

pub struct CommandArgument {
    pub name: String,
    pub description: String,
    pub arg_type: String
}

impl CommandArgument {
    pub fn new(name: &str, description: &str, arg_type: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            arg_type: arg_type.to_string()
        }
    }
}

pub struct Command {
    pub name: String,
    pub purpose: String,
    pub return_type: String,
    pub args: Vec<CommandArgument>,
    pub run: Box<dyn CommandImpl>
}

pub struct Plugin {
    pub name: String,
    pub cycle: Box<dyn PluginCycle>,
    pub dependencies: Vec<String>,
    pub commands: Vec<Command>
}

#[derive(Debug, Clone)]
pub struct NotFoundError(pub String);

impl Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NotFoundError: {}", self.0)
    }
}

impl Error for NotFoundError {}
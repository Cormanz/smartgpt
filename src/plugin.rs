use std::{collections::HashMap, error::Error, fmt::Display, any::Any};

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned, Deserialize};
use serde_json::Value;

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
        write!(f, "the '{}' tool did not receive the '{}' argument.", self.0, self.1)
    }
}

impl<'a> Error for CommandNoArgError<'a> {}

use crate::{LLM, ScriptValue, MemorySystem, AutoType};

#[async_trait]
pub trait PluginData: Any + Send + Sync {
    async fn apply(&mut self, name: &str, info: Value) -> Result<Value, Box<dyn Error>>;
}

pub struct PluginStore(pub HashMap<String, Box<dyn PluginData>>);

impl PluginStore {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

pub struct EndGoals {
    pub end_goal: usize,
    pub end_goals: Vec<String>
}

impl EndGoals {
    pub fn get(&self) -> String {
        self.end_goals[self.end_goal].clone()
    }
}

pub struct AgentInfo {
    pub llm: LLM,
    pub observations: Box<dyn MemorySystem>,
    pub reflections: Box<dyn MemorySystem>
}

pub struct Agents {
    pub static_agent: AgentInfo,
    pub planner: AgentInfo,
    pub dynamic: AgentInfo,
    pub fast: AgentInfo
}

impl Agents {
    pub fn same(init: impl Fn() -> Result<AgentInfo, Box<dyn Error>>) -> Result<Agents, Box<dyn Error>> {
        Ok(
            Agents {
                static_agent: init()?,
                planner: init()?,
                dynamic: init()?,
                fast: init()?
            }
        )
    }
}

pub struct CommandContext {
    pub plugin_data: PluginStore,
    pub agents: Agents,
    pub plugins: Vec<Plugin>,
    pub disabled_tools: Vec<String>,
    pub assets: HashMap<String, String>
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

pub enum CommandResult {
    ScriptValue(ScriptValue),
    Text(String)
}

#[async_trait]
pub trait CommandImpl : Send + Sync {
    async fn invoke(&self, ctx: &mut CommandContext, args: ScriptValue) -> Result<CommandResult, Box<dyn Error>>;

    fn box_clone(&self) -> Box<dyn CommandImpl>;
}

#[async_trait]
pub trait PluginCycle : Send + Sync {
    async fn create_context(&self, context: &mut CommandContext, previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>>;
    fn create_data(&self, value: Value) -> Option<Box<dyn PluginData>>;
}

pub struct EmptyCycle;

#[async_trait]
impl PluginCycle for EmptyCycle {
    async fn create_context(&self, _context: &mut CommandContext, _previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
        Ok(None)
    }

    fn create_data(&self, _: Value) -> Option<Box<dyn PluginData>> {
        None
    }
}

#[derive(Clone)]
pub struct ToolArgument {
    pub name: String,
    pub example: String
}

impl ToolArgument {
    pub fn new(name: &str, example: &str) -> Self {
        Self {
            name: name.to_string(),
            example: example.to_string()
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum ToolType {
    Resource,
    Action { needs_permission: bool }
}

pub struct Tool {
    pub name: String,
    pub purpose: String,
    pub args: Vec<ToolArgument>,
    pub tool_type: ToolType,
    pub run: Box<dyn CommandImpl>
}

impl Tool {
    pub fn box_clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            purpose: self.purpose.clone(),
            args: self.args.clone(),
            tool_type: self.tool_type.clone(),
            run: self.run.box_clone()
        }
    }
}

pub struct Plugin {
    pub name: String,
    pub cycle: Box<dyn PluginCycle>,
    pub dependencies: Vec<String>,
    pub tools: Vec<Tool>
}

#[derive(Debug, Clone)]
pub struct NotFoundError(pub String);

impl Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NotFoundError: {}", self.0)
    }
}

impl Error for NotFoundError {}
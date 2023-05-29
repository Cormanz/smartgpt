use std::{sync::{Mutex, Arc}, collections::HashMap, error::Error, vec};

use serde::Serialize;
use serde_json::Value;

use crate::{CommandContext, PluginStore, Agents, AgentInfo, LLMProvider, LLMModel, LLM, ChatGPTProvider, ChatGPTConfig, memory_from_provider, LocalProvider};

pub struct SmartGPT {
    pub personality: String,
    pub context: Arc<Mutex<CommandContext>>
}

impl SmartGPT {
    fn create() -> Result<(), Box<dyn Error>> {
        let smartgpt = SmartGPT {
            personality: "A superintelligent AI".to_string(),
            context: Arc::new(Mutex::new(CommandContext {
                agents: Agents::same(|| Ok(AgentInfo {
                    llm: LLM::from_provider(ChatGPTProvider, ChatGPTConfig {
                        api_key: "X".to_string(),
                        ..Default::default()
                    })?,
                    observations: memory_from_provider(LocalProvider, Value::Null)?,
                    reflections: memory_from_provider(LocalProvider, Value::Null)?
                }))?,
                plugin_data: PluginStore::new(),
                assets: HashMap::new(),
                plugins: vec![],
                disabled_tools: vec![]
            }))
        };

        Ok(())
    }
}
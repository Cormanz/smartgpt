use std::{sync::{Mutex, Arc}, error::Error, fmt::Display};

use serde::Serialize;


use crate::{CommandContext, auto::{run_auto, Action, DisallowedAction, Update}};

#[derive(Debug, Clone)]
pub struct NoPluginError(pub String);

impl Display for NoPluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NoPluginError({})", self.0)
    }
}

impl Error for NoPluginError {}

pub struct SmartGPT {
    pub personality: String,
    pub context: Arc<Mutex<CommandContext>>
}

impl SmartGPT {
    pub fn load_plugin_data<T : Serialize>(
        &mut self,
        plugin_name: &str,
        data: T
    ) -> Result<(), Box<dyn Error>> {
        let mut context = self.context.lock().unwrap();

        let plugin_name = plugin_name.to_string();
        let no_plugin_error = Box::new(NoPluginError(plugin_name.clone()));

        let plugin = context.plugins.iter()
            .find(|plugin| plugin.name == plugin_name.clone())
            .ok_or(no_plugin_error)?;
        
        let data = plugin.cycle.create_data(serde_json::to_value(data)?);
        if let Some(data) = data {
            context.plugin_data.0.insert(plugin_name, data);
        }

        Ok(())
    }

    pub fn run_task(
        &mut self,
        task: &str,
        allow_action: &mut impl FnMut(&Action) -> Result<(), DisallowedAction>,
        listen_to_update: &mut impl FnMut(&Update) -> Result<(), Box<dyn Error>>
    ) -> Result<String, Box<dyn Error>> {
        run_auto(self, task, allow_action, listen_to_update)
    }
}
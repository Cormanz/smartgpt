use crate::{
    auto::{run::Action, DisallowedAction},
    AgentInfo, SmartGPT,
};
use serde::Serialize;
use std::error::Error;

mod actor;
mod adept;
mod methodical;
mod tools;
mod updates;

pub use actor::*;
pub use adept::*;
pub use methodical::*;
pub use tools::*;
pub use updates::*;

pub fn run_worker(
    smartgpt: &mut SmartGPT,
    task: &str,
    personality: &str,
    allow_action: &mut impl FnMut(&Action) -> Result<(), DisallowedAction>,
    listen_to_update: &mut impl FnMut(&Update) -> Result<(), Box<dyn Error>>,
) -> Result<String, Box<dyn Error>> {
    let mut context = smartgpt.context.lock().unwrap();

    let response = run_brain_agent(
        &mut context,
        &|ctx| &mut ctx.agents.dynamic,
        task,
        personality,
        allow_action,
        listen_to_update,
    )?;

    Ok(response)
}

pub fn log_yaml<T: Serialize>(data: &T) -> Result<(), Box<dyn Error>> {
    println!("{}", serde_yaml::to_string(&data)?);

    Ok(())
}

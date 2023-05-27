use std::{error::Error};
use crate::{ProgramInfo, AgentInfo};
use serde::Serialize;

mod adept;
mod actor;
mod methodical;
mod tools;

pub use adept::*;
pub use actor::*;
pub use methodical::*;
pub use tools::*;

pub fn run_employee(program: &mut ProgramInfo, task: &str, personality: &str) -> Result<String, Box<dyn Error>> {
    let mut context = program.context.lock().unwrap();
    
    /*let refine_info = refine(&mut context, &|context| &mut context.agents.planner, task)?;
    log_yaml(&refine_info)?;

    let task = &refine_info.task;*/

    let response = run_brain_agent(&mut context, &|ctx| &mut ctx.agents.dynamic, task, personality)?;

    Ok(response)
}

pub fn log_yaml<T: Serialize>(data: &T) -> Result<(), Box<dyn Error>> {
    println!("{}", serde_yaml::to_string(&data)?);

    Ok(())
}
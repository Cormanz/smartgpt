use std::{error::Error};





use crate::{ProgramInfo, AgentInfo};



mod adept;
mod actor;
mod react;
mod refine;
mod methodical;
mod tools;

pub use adept::*;
pub use actor::*;
pub use react::*;
pub use refine::*;
pub use methodical::*;
pub use tools::*;

pub fn run_employee<T>(program: &mut ProgramInfo, task: &str, personality: &str, _end: impl Fn(&mut AgentInfo) -> T) -> Result<T, Box<dyn Error>> {
    let mut context = program.context.lock().unwrap();
    
    /*let refine_info = refine(&mut context, &|context| &mut context.agents.planner, task)?;
    log_yaml(&refine_info)?;

    let task = &refine_info.task;*/

    let response = run_brain_agent(&mut context, &|ctx| &mut ctx.agents.planner, task, personality)?;
    println!("{response}");

    panic!("T");
}
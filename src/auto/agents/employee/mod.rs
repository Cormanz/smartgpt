use std::{error::Error, sync::{Arc, Mutex}, fmt::Display, ascii::AsciiExt, collections::HashMap};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{ProgramInfo, generate_commands, Message, Agents, ScriptValue, GPTRunError, Expression, Command, CommandContext, auto::{try_parse_json, ParsedResponse, run::{run_command, Action}, agents::findings::{to_points, ask_for_findings}}, LLM, AgentInfo, Weights, generate_commands_short};

use super::findings::get_observations;

mod brain;
mod actor;
mod react;
mod refine;
mod methodical;

pub use brain::*;
pub use actor::*;
pub use react::*;
pub use refine::*;
pub use methodical::*;

pub fn run_employee<T>(program: &mut ProgramInfo, task: &str, end: impl Fn(&mut AgentInfo) -> T) -> Result<T, Box<dyn Error>> {
    let mut context = program.context.lock().unwrap();
    
    /*let refine_info = refine(&mut context, &|context| &mut context.agents.planner, task)?;
    log_yaml(&refine_info)?;

    let task = &refine_info.task;*/

    let response = run_brain_agent(&mut context, &|ctx| &mut ctx.agents.react, task)?;
    println!("{response}");

    panic!("T");
}
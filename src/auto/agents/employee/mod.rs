use std::{error::Error, sync::{Arc, Mutex}, fmt::Display, ascii::AsciiExt, collections::HashMap};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{ProgramInfo, generate_commands, Message, Agents, ScriptValue, GPTRunError, Expression, Command, CommandContext, auto::{try_parse_json, ParsedResponse, run::{run_command, Action}, agents::findings::{to_points, ask_for_findings}}, LLM, AgentInfo, Weights, generate_commands_short};

use super::findings::get_observations;

mod actor;
mod react;

pub use actor::*;
pub use react::*;

pub fn run_employee<T>(program: &mut ProgramInfo, task: &str, end: impl Fn(&mut AgentInfo) -> T) -> Result<T, Box<dyn Error>> {
    let mut context = program.context.lock().unwrap();
    
    println!("{}", use_tool(
        &mut context,
        &|context| &mut context.agents.employee,
        Action {
            tool: "browse_url".to_string(),
            args: ScriptValue::Dict(HashMap::from([
                (
                    "url".to_string(),
                    "https://github.com/Cormanz/smartgpt".into()
                )
            ]))
        }
    )?);

    panic!("T");
}
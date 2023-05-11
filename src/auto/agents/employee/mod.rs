use std::{error::Error, sync::{Arc, Mutex}, fmt::Display, ascii::AsciiExt};

pub mod reflector;
pub mod brainstormer;

use reflector::*;
use brainstormer::*;

use colored::Colorize;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{ProgramInfo, generate_commands, Message, Agents, ScriptValue, GPTRunError, Expression, Command, CommandContext, auto::{try_parse_json, ParsedResponse, run::run_command, agents::findings::{to_points, ask_for_findings}}, LLM, AgentInfo, Weights, generate_commands_short};

use super::findings::get_observations;

pub fn run_employee<T>(program: &mut ProgramInfo, task: &str, end: impl Fn(&mut AgentInfo) -> T) -> Result<T, Box<dyn Error>> {
    // Ask the Reflector for initial observations.
    let InitialObservations { ideas } = initial_observations(program, task)?;

    println!("Initial Ideas: {ideas:?}");

    let ProgramInfo { context, .. } = program;
    let mut context = context.lock().unwrap();

    // Save those observations to long-term memory.
    for idea in ideas {
        let AgentInfo { llm, observations, .. } = &mut context.agents.employee;
        observations.store_memory_sync(llm, &idea)?;
    }
    
    println!("Saved...");

    drop(context);
    let brainwave = brainstorm(program, task)?;
    println!("{:?}", brainwave);

    panic!("T");
}
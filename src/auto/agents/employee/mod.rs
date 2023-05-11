use std::{error::Error, sync::{Arc, Mutex}, fmt::Display, ascii::AsciiExt};

pub mod reflector;
pub mod executor;
pub mod brainstormer;

use reflector::*;
use brainstormer::*;
use executor::*;

use colored::Colorize;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{ProgramInfo, generate_commands, Message, Agents, ScriptValue, GPTRunError, Expression, Command, CommandContext, auto::{try_parse_json, ParsedResponse, run::run_command, agents::findings::{to_points, ask_for_findings}}, LLM, AgentInfo, Weights, generate_commands_short};

use super::findings::get_observations;

pub fn run_employee<T>(program: &mut ProgramInfo, task: &str, end: impl Fn(&mut AgentInfo) -> T) -> Result<T, Box<dyn Error>> {
    println!("{}", "Loaded employee...".blue());
    println!("{}", "Reflecting...".yellow());
    
    // Ask the Reflector for initial observations.
    let InitialObservations { ideas } = initial_observations(program, task)?;

    let ProgramInfo { context, .. } = program;
    let mut context = context.lock().unwrap();

    println!();
    println!("{}", "Ideas to complete task:".blue());

    // Save those observations to long-term memory.
    for idea in ideas {
        println!("    {} {}", "-".black(), idea);
        
        let AgentInfo { llm, observations, .. } = &mut context.agents.employee;
        observations.store_memory_sync(llm, &idea)?;
    }
    
    println!();
    println!("{}", "Saved ideas to memory.".green());
    println!("{}", "Brainstorming...".yellow());

    drop(context);

    // Brainstorm thoughts and a tool to use
    let brainwave = brainstorm(program, task)?;
    
    println!();
    println!("{}", "Brainstormed Ideas:".blue());
    println!("{}", serde_yaml::to_string(&brainwave)?);

    // Run the brainstormed tool
    let out = execute(program, brainwave.action)?;

    println!();
    println!("{}", "Ran Command:".blue());
    println!("{out}");

    // Collect observations from command
    let CommandObservations { command_success, changes, notes } = collect_observations(program, &out)?;

    let ProgramInfo { context, .. } = program;
    let mut context = context.lock().unwrap();

    println!();
    println!("{} {}", "Previous Command Success:".blue(), command_success);
    println!("{}", "Mental Notes:".blue());

    let mut memories = notes.unwrap_or(vec![]);
    memories.extend(changes.unwrap_or(vec![]));

    // Save those observations to long-term memory.
    for idea in memories {
        println!("    {} {}", "-".black(), idea);
        
        let AgentInfo { llm, observations, .. } = &mut context.agents.employee;
        observations.store_memory_sync(llm, &idea)?;
    }

    panic!("T");
}
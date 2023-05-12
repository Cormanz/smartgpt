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
        
        let AgentInfo { llm, reflections, .. } = &mut context.agents.employee;
        reflections.store_memory_sync(llm, &idea)?;
    }
    
    println!();
    println!("{}", "Saved ideas to memory.".green());
    drop(context);

    loop {
        for _ in 0..2 {
            println!("{}", "Brainstorming...".yellow());
    
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
            let CommandObservations { tool_success, changes, notes } = collect_observations(program, &out)?;
    
            let ProgramInfo { context, .. } = program;
            let mut context = context.lock().unwrap();
    
            println!();
            println!("{} {}", "Previous Tool Success:".blue(), tool_success);
            println!("{}", "Mental Notes:".blue());
    
            let mut memories = notes.unwrap_or(vec![]);
            memories.extend(changes.unwrap_or(vec![]));
    
            // Save those observations to long-term memory.
            for idea in memories {
                println!("    {} {}", "-".black(), idea);
                
                let AgentInfo { llm, observations, .. } = &mut context.agents.employee;
                observations.store_memory_sync(llm, &idea)?;
            }
    
            println!();
            
            context.agents.employee.observations.decay_recency_sync(0.93)?;
            context.agents.employee.reflections.decay_recency_sync(0.97)?;

            drop(context);
        }
        
        println!("{}", "Reflecting...".yellow());
    
        let reflections = self_reflect(program, task)?;
    
        println!();
        println!("{}", "Long-Term Reflections:".blue());
        println!("{}", serde_yaml::to_string(&reflections)?);

        let ProgramInfo { context, .. } = program;

        let mut context = context.lock().unwrap();

        for reflection in reflections.reflections.unwrap_or(vec![]) {
            let AgentInfo { llm, reflections, .. } = &mut context.agents.employee;
            reflections.store_memory_sync(llm, &reflection)?;
        }

        if reflections.task_complete {
            break;
        }
    }

    panic!("T");
}
use std::{error::Error, fmt::Display, process, fs, io};

use colored::Colorize;

mod plugin;
mod plugins;
mod tools;
mod chunk;
mod llms;
mod api;
mod runner;
mod memory;
mod auto;

pub use plugin::*;
pub use plugins::*;
pub use tools::*;
pub use chunk::*;
pub use llms::*;
pub use api::*;
pub use runner::*;
pub use memory::*;

use serde::{Deserialize, Serialize};

use crate::auto::run_auto;

#[derive(Serialize, Deserialize)]
pub struct NewEndGoal {
    #[serde(rename = "new end goal")] new_end_goal: String
}

#[derive(Debug, Clone)]
pub struct NoThoughtError;

impl Display for NoThoughtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "no thought detected.")
    }
}

impl Error for NoThoughtError {}

fn main() -> Result<(), Box<dyn Error>> {
    let config = fs::read_to_string("config.yml");

    let config = match config {
        Ok(config) => config,
        Err(_) => {
            println!("{}", "Could not find 'config.yml'.".red());
            println!("Generating new config.yml...");

            fs::write("config.yml", DEFAULT_CONFIG)?;

            fs::read_to_string("config.yml")?
        }
    };

    let mut smartgpt = load_config(&config)?;

    print!("\x1B[2J\x1B[1;1H");
    println!("{}: {}", "Personality".blue(), smartgpt.personality);

    println!("{}:", "Plugins".blue());
    let mut exit_dependency_error = false;

    let context = smartgpt.context.lock().unwrap();

    for plugin in &context.plugins {
        for dependency in &plugin.dependencies {
            let dependency_exists = context.plugins.iter().any(|dep| &dep.name == dependency);
            if !dependency_exists {
                println!("{}: Cannot run {} without its needed dependency of {}.", "Error".red(), plugin.name, dependency);
                exit_dependency_error = true;
            }
        }

        let tools = if plugin.tools.len() == 0 {
            vec![ "<no tools>".white() ]
        } else {
            plugin.tools.iter()
                .map(|el| {
                    let tool_name = el.name.to_string();
                    if context.disabled_tools.contains(&tool_name) {
                        el.name.to_string().red()
                    } else {
                        el.name.to_string().green()
                    }
                }).collect::<Vec<_>>()
        };

        if !exit_dependency_error {
            print!("{} {} (tools: ", "-".black(), plugin.name);
            for (ind, tool) in tools.iter().enumerate() {
                print!("{}", tool);
                if ind < tools.len() - 1 {
                    print!(", ");
                }
            }
            println!(")");
        }
    }

    if exit_dependency_error {
        process::exit(1);
    }

    println!();

    drop(context);

    smartgpt.run_task( 
        "Write an essay on the Rust programming language.", 
        &|_| Ok(()), 
        &|_| Ok(())
    )?;

    Ok(())
}
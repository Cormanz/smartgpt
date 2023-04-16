use std::{error::Error, time::Duration, fmt::Display, mem::take, collections::HashMap, process, fs};

use colored::Colorize;
use reqwest::{self, Client, header::{USER_AGENT, HeaderMap}};
use async_openai::{
    Client as OpenAIClient, types::{CreateCompletionRequestArgs, CreateChatCompletionRequest, ChatCompletionRequestMessage, Role, CreateCompletionResponse, CreateChatCompletionResponse}, error::OpenAIError,
};

mod plugin;
mod parse;
mod prompt;
mod commands;
mod plugins;
mod chunk;
mod llm;
mod config;
mod runner;
mod agents;

pub use plugin::*;
pub use parse::*;
pub use prompt::*;
pub use commands::*;
pub use plugins::*;
pub use chunk::*;
pub use llm::*;
pub use config::*;
pub use runner::*;

use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use serde_json::Value;

use crate::agents::run_manager;

#[derive(Serialize, Deserialize)]
pub struct NewEndGoal {
    #[serde(rename = "new end goal")] new_end_goal: String
}

fn debug_yaml(results: &str) -> Result<(), Box<dyn Error>> {
    let json: Value = serde_json::from_str(&results)?;
    let mut yaml: String = serde_yaml::to_string(&json)?;
    yaml = yaml.trim().to_string();

    if yaml.len() > 1500 {
        yaml = yaml.chars().take(1500).map(|el| el.to_string()).collect::<Vec<_>>().join("") + "... (chopped off at 1,500 characters)";
    }

    println!("{yaml}");

    Ok(())
}

#[derive(Debug, Clone)]
pub struct NoThoughtError;

impl Display for NoThoughtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "no thought detected.")
    }
}

impl Error for NoThoughtError {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = fs::read_to_string("config.yml")?;
    let mut program = load_config(&config).await?;

    print!("\x1B[2J\x1B[1;1H");
    println!("{}: {}", "AI Name".blue(), program.name);
    println!("{}: {}", "Role".blue(), program.role);
    println!("{}: {}", "Task".blue(), program.task);

    println!("{}:", "Plugins".blue());
    let mut exit_dependency_error = false;
    for plugin in &program.plugins {
        for dependency in &plugin.dependencies {
            let dependency_exists = program.plugins.iter().any(|dep| &dep.name == dependency);
            if !dependency_exists {
                println!("{}: Cannot run {} without its needed dependency of {}.", "Error".red(), plugin.name, dependency);
                exit_dependency_error = true;
            }
        }

        let commands = if plugin.commands.len() == 0 {
            vec![ "<no commands>".white() ]
        } else {
            plugin.commands.iter()
                .map(|el| {
                    let command_name = el.name.to_string();
                    if program.disabled_commands.contains(&command_name) {
                        el.name.to_string().red()
                    } else {
                        el.name.to_string().green()
                    }
                }).collect::<Vec<_>>()
        };

        if !exit_dependency_error {
            print!("{} {} (commands: ", "-".black(), plugin.name);
            for (ind, command) in commands.iter().enumerate() {
                print!("{}", command);
                if ind < commands.len() - 1 {
                    print!(", ");
                }
            }
            println!(")");
        }

        // OH NO OH NO OH NO
        let data = plugin.cycle.create_data(true.into()).await;
        if let Some(data) = data {
            program.context.plugin_data.0.insert(plugin.name.clone(), data);
        }
    }

    if exit_dependency_error {
        process::exit(1);
    }

    println!();

    run_manager(&mut program).await?;

    Ok(())
}
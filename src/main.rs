use std::{error::Error, time::Duration, fmt::Display, mem::take, collections::HashMap, process, fs, io};

use colored::Colorize;
use reqwest::{self, Client, header::{USER_AGENT, HeaderMap}};
use async_openai::{
    Client as OpenAIClient, types::{CreateCompletionRequestArgs, CreateChatCompletionRequest, ChatCompletionRequestMessage, Role, CreateCompletionResponse, CreateChatCompletionResponse}, error::OpenAIError,
};

mod plugin;
mod plugins;
mod commands;
mod chunk;
mod llms;
mod config;
mod runner;
mod memory;
mod auto;

pub use plugin::*;
pub use plugins::*;
pub use commands::*;
pub use chunk::*;
pub use llms::*;
pub use config::*;
pub use runner::*;
pub use memory::*;

use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use serde_json::Value;

use crate::auto::{run_task_auto, run_assistant_auto};

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

    let mut program = load_config(&config)?;

    print!("\x1B[2J\x1B[1;1H");
    println!("{}: {}", "Personality".blue(), program.personality);
    println!("{}: {:?}", "Type".blue(), program.auto_type.clone());

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
        let data = plugin.cycle.create_data(true.into());
        if let Some(data) = data {
            let mut context = program.context.lock().unwrap();
            context.plugin_data.0.insert(plugin.name.clone(), data);
        }
    }

    if exit_dependency_error {
        process::exit(1);
    }

    println!();

    match program.auto_type.clone() {
        AutoType::Assistant => {
            let mut messages: Vec<Message> = vec![];
            let stdin = io::stdin();
            loop {
                println!("{}", "> User".yellow());
                
                let mut input = String::new();
                stdin.read_line(&mut input).unwrap();

                println!();

                let response = run_assistant_auto(&mut program, &messages, &input)?;

                messages.push(Message::User(input));
                messages.push(Message::Assistant(response.clone()));

                println!("{}", "> Assistant".yellow());
                println!("{}", response);
                println!();
            }
        },
        AutoType::Runner { task } => {
            run_task_auto(&mut program, &task)?;
        }
    }

    Ok(())
}
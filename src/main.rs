// use std::{error::Error, fmt::Display, process, fs, io};

// mod plugin;
// mod plugins;
// mod commands;
// mod chunk;
// mod llms;
// mod config;
// mod runner;
// mod memory;
// mod auto;

// pub use plugin::*;
// pub use plugins::*;
// pub use commands::*;
// pub use chunk::*;
// pub use llms::*;
// use old_config::*;
// pub use config::*;
// pub use runner::*;
// pub use memory::*;

// use serde::{Deserialize, Serialize};
// use serde_json::Value;

// use crate::auto::{run_task_auto, run_assistant_auto};

// #[derive(Serialize, Deserialize)]
// pub struct NewEndGoal {
//     #[serde(rename = "new end goal")] new_end_goal: String
// }

// fn debug_yaml(results: &str) -> Result<(), Box<dyn Error>> {
//     let json: Value = serde_json::from_str(&results)?;
//     let mut yaml: String = serde_yaml::to_string(&json)?;
//     yaml = yaml.trim().to_string();

//     if yaml.len() > 1500 {
//         yaml = yaml.chars().take(1500).map(|el| el.to_string()).collect::<Vec<_>>().join("") + "... (chopped off at 1,500 characters)";
//     }

//     println!("{yaml}");

//     Ok(())
// }

// #[derive(Debug, Clone)]
// pub struct NoThoughtError;

// impl Display for NoThoughtError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", "no thought detected.")
//     }
// }

// impl Error for NoThoughtError {}

mod config;

use anyhow::Result;
use colored::Colorize;

fn main() -> Result<()> {
    let config = match std::fs::read_to_string(config::CONFIG_LOCATION) {
        Ok(contents) => toml::from_str(&contents)?,
        _ => config::write_defaults()?
    };

    check_dependencies()?;

    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("{}: {}", "Personality".blue(), config.personality);
    println!("{}: {:?}", "Type".blue(), config.auto_type);
    println!("{}:", "Plugins".blue());
    
    Ok(())
}

fn check_dependencies() -> Result<()> {
    Ok(())
}

    // let mut exit_dependency_error = false;
    // for plugin in &config.plugins {
    //     for dependency in &plugin.dependencies {
    //         let dependency_exists = config.plugins.iter().any(|dep| &dep.name == dependency);
    //         if !dependency_exists {
    //             println!("{}: Cannot run {} without its needed dependency of {}.", "Error".red(), plugin.name, dependency);
    //             exit_dependency_error = true;
    //         }
    //     }

    //     let commands = if plugin.commands.len() == 0 {
    //         vec![ "<no commands>".white() ]
    //     } else {
    //         plugin.commands.iter()
    //             .map(|el| {
    //                 let command_name = el.name.to_string();
    //                 if config.disabled_commands.contains(&command_name) {
    //                     el.name.to_string().red()
    //                 } else {
    //                     el.name.to_string().green()
    //                 }
    //             }).collect::<Vec<_>>()
    //     };

    //     if !exit_dependency_error {
    //         print!("{} {} (commands: ", "-".black(), plugin.name);
    //         for (ind, command) in commands.iter().enumerate() {
    //             print!("{}", command);
    //             if ind < commands.len() - 1 {
    //                 print!(", ");
    //             }
    //         }
    //         println!(")");
    //     }

    //     // OH NO OH NO OH NO
    //     let data = plugin.cycle.create_data(true.into());
    //     if let Some(data) = data {
    //         let mut context = config.context.lock().unwrap();
    //         context.plugin_data.0.insert(plugin.name.clone(), data);
    //     }
    // }

    // if exit_dependency_error {
    //     process::exit(1);
    // }

    // println!();

    // match config.auto_type.clone() {
    //     AutoType::Assistant { token_limit } => {
    //         let mut messages: Vec<Message> = vec![];
    //         let stdin = io::stdin();
    //         loop {
    //             println!("{}", "> User".yellow());
                
    //             let mut input = String::new();
    //             stdin.read_line(&mut input).unwrap();

    //             println!();

    //             let response = run_assistant_auto(&mut config, &messages, &input, token_limit)?;

    //             messages.push(Message::User(input));
    //             messages.push(Message::Assistant(response.clone()));

    //             println!("{}", "> Assistant".yellow());
    //             println!("{}", response);
    //             println!();
    //         }
    //     },
    //     AutoType::Runner { task } => {
    //         run_task_auto(&mut config, &task)?;
    //     }
    // }
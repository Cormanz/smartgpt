use std::{error::Error, fmt::Display, process, fs};
use colored::Colorize;

pub use smartgpt::*;

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

    let (task, mut smartgpt) = load_config(&config)?;

    print!("\x1B[2J\x1B[1;1H");
    println!("{}: {}", "Personality".blue(), smartgpt.personality);
    println!("{}: {}", "Task".blue(), task);

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
        &task, 
        &mut |_| Ok(()), 
        &mut log_update
    )?;

    Ok(())
}
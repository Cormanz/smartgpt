use std::error::Error;

use colored::Colorize;

use crate::auto::{Update, DynamicUpdate, StaticUpdate, log_yaml, NamedAsset};

pub fn log_update(update: &Update) -> Result<(), Box<dyn Error>> {
    match update {
        Update::DynamicAgent(update) => {
            match update {
                DynamicUpdate::Plan(plan) => {
                    println!("{} | {}", "Dynamic Agent".blue().bold(), "Created Plan".white());
                    println!();
                    println!("{plan}");
                    println!();
                },
                DynamicUpdate::Thoughts(thoughts) => {
                    println!("{} | {}", "Dynamic Agent".blue().bold(), "Made Decision".white());
                    println!();
                    log_yaml(&thoughts)?;
                    println!();
                }
            }
        },
        Update::StaticAgent(update) => {
            match update {
                StaticUpdate::Plan(plan) => {
                    println!("{} | {}", "Static Agent".yellow().bold(), "Created Plan".white());
                    println!();
                    log_yaml(&plan)?;
                    println!();
                },
                StaticUpdate::SelectedStep(step) => {
                    println!("{} | {}", "Static Agent".yellow().bold(), "Selected Step".white());
                    println!();
                    log_yaml(&step)?;
                    println!();
                },
                StaticUpdate::Thoughts(thoughts) => {
                    println!("{} | {}", "Static Agent".yellow().bold(), "Running Step".white());
                    println!();
                    log_yaml(&thoughts)?;
                    println!();
                },
                StaticUpdate::ActionResults(out) => {
                    println!("{} | {}", "Static Agent".yellow().bold(), "Ran Action".white());
                    println!();
                    println!("{out}");
                    println!();
                },
                StaticUpdate::AddedAssets(assets) => {
                    println!("{} | {}", "Static Agent".yellow().bold(), "Added Assets".white());
                    println!();
                    for NamedAsset(name, content) in assets {
                        println!("{} {}", ">".white(), name.bold());
                        println!("{content}");
                        println!();
                    }
                    println!();
                },
                _ => {}
            }
        }
    }

    Ok(())
}
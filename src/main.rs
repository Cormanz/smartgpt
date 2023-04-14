use std::{error::Error, time::Duration, fmt::Display, mem::take, collections::HashMap, process, fs};

use colored::Colorize;
use faiss::{MetricType, index_factory, FlatIndex, IdMap};
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
mod commandrunner;

pub use plugin::*;
pub use parse::*;
pub use prompt::*;
pub use commands::*;
pub use plugins::*;
pub use chunk::*;
pub use llm::*;
pub use config::*;
pub use commandrunner::*;

use serde::{Deserialize, Serialize};
use serde_json::Value;

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

async fn apply_process(
    program: &mut ProgramInfo
) -> Result<(), Box<dyn Error>> {
    let ProgramInfo { 
        name, role, goals, 
        plugins, context, disabled_commands } = program;

    let previous_prompt = if context.llm.message_history.len() > 1 {
        Some(context.llm.message_history.iter()
            .filter(|message| message.is_assistant())
            .map(|message| message.content().to_string())
            .collect::<Vec<_>>()
            .join("\n"))
    } else {
        None
    };

    let end_goal = context.end_goals.get();
    let prompt = generate_prompt(
        context, name, role, &end_goal,
        &disabled_commands, &plugins, previous_prompt.as_deref()
    ).await?;

    let mut messages: Vec<Message> = context.llm.message_history.clone();

    if messages.len() > 0 {
        messages.remove(0);
    }
    messages.insert(0, Message::User(prompt.to_string()));

    if let Some(last) = messages.last_mut() {
        last.set_content(&format!("{}\n\nEnsure the response can be parsed by Python json.loads", last.content()));
    };

    let message: String = context.llm.model.get_response(&messages).await?;
    messages.push(Message::Assistant(message.clone()));
    let json = message.clone();

    let response = parse_response(&json).map_err(|err| {
        println!("ERROR DEBUG");
        println!("{json}");

        err
    })?;

    println!("{}: {}", "Findings".blue(), response.summary
        .iter()
        .flat_map(|el| {
            let mut takeaways = vec![ el.takeaway.clone() ];
            takeaways.extend(el.points.clone());
            takeaways
        })
        .collect::<Vec<_>>()
        .join(" ")
    );

    println!("{}: {}", "Current Endgoal".blue(), response.goal_information.current_endgoal);
    println!("{}:", "Planned Commands".blue());
    for task in &response.goal_information.commands {
        println!("    {} {}", "-".black(), task);
    }
    println!();

    if response.goal_information.end_goal_complete {
        println!("{}", "End Goal is Complete. Moving onto next end goal...".yellow());
        context.end_goals.end_goal += 1;

        let new_end_goal = NewEndGoal {
            new_end_goal: context.end_goals.get()
        };
        let info = serde_json::to_string(&new_end_goal)?;

        messages.push(Message::User(info));
        return Ok(());
    }

    /*println!("{}: {}", "Current Goal".blue(), response.thought.current_goal);
    println!("{}:", "Plan".blue());
    for (ind, step) in response.thought.plan.iter().enumerate() {
        println!("{}{} {}", (ind + 1).to_string().black(), ".".black(), step);
    }
    println!("{}: {}", "Idea".blue(), response.thought.idea);
    println!("{}: {}", "Reasoning".blue(), response.thought.reasoning);
    println!("{}: {}", "Criticism".blue(), response.thought.criticism);
    println!();
    println!("{}", "-".black());
    println!();*/


    let none_request = CommandRequest {
        name: "none".to_string(),
        args: HashMap::new()
    };
    let command_request = response.command.as_ref().unwrap_or(&none_request);
    let args = command_request.args.iter()
        .map(|(name, value)| format!(" [{name}: {value}]"))
        .collect::<Vec<_>>().join(" ");
    let command = format!("{}{}", command_request.name, args);
    println!("{}: {}", "Command".blue(), command);

    let results = run_command(context, &response, plugins).await?;

    println!();
    println!("{} \"{}\"", "Executed Command".blue(), command);

    match debug_yaml(&results) {
        Err(_) => {
            println!("{results}");
        },
        _ => {}
    };
    
    let mut command_result_content = "Command ".to_string();
    command_result_content.push_str(&command_request.name);
    command_result_content.push_str(" returned: ");
    command_result_content.push_str(&results);

    messages.push(Message::User(command_result_content.clone()));

    context.llm.message_history = messages;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    test_runner();
    return Ok(());

    let config = fs::read_to_string("config.yml")?;
    let mut program = load_config(&config).await?;

    println!("{}: {}", "AI Name".blue(), program.name);
    println!("{}: {}", "Role".blue(), program.role);
    println!("{}:", "Goals".blue());

    for goal in &program.goals {
        println!("{} {}", "-".black(), goal);
    }

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

    loop {
        println!("{}", "Generating...".yellow());

        let mut result: Result<(), Box<dyn Error>> = Err(Box::new(FilesNoQueryError));

        let mut all_text = program.context.llm.message_history
            .iter()
            .map(|el| el.content().clone())
            .collect::<Vec<_>>()
            .join("");   
        let mut tokens = tokenize(&program.context.tokenizer, &all_text);

        println!("{}: {}", "Chars".yellow(), all_text.len());
        println!("{}: {}", "Tokens".yellow(), tokens.len());

        let mut total_cleaned_tokens: usize = 0;
        let mut clean_count: usize = 0;
        while tokens.len() > 3200 {
            if program.context.llm.message_history[1].is_assistant() {
                let response = program.context.llm.message_history.remove(1);
                let response = response.content();
                let response = parse_response(&response)?;
                let command_response = program.context.llm.message_history.remove(1);
                let command_response = command_response.content();
                for plugin in &program.plugins {
                    plugin.cycle.apply_removed_response(&mut program.context, &response, &command_response, true).await?;
                }
            } else {
                program.context.llm.message_history.remove(1);
            }

            let new_text = program.context.llm.message_history
                .iter()
                .map(|el| el.content().clone())
                .collect::<Vec<_>>()
                .join("");   

            let prev_tokens = tokens;
            tokens = tokenize(&program.context.tokenizer, &new_text);

            total_cleaned_tokens += prev_tokens.len() - tokens.len();
            clean_count += 1;

            if total_cleaned_tokens > 2000 && clean_count >= 2 {
                break;
            }

            println!("{}: {}", "Cleaned Tokens So Far".yellow(), prev_tokens.len() - tokens.len());
        }
        
        for i in 0..5 {
            if i >= 1 {
                println!("{} Trying again... {}{}", "Error".red(), "Attempt #".blue(), (i + 1).to_string().blue());
            }

            result = apply_process(&mut program).await;

            if let Ok(_) = result {
                break;
            }

            result.as_ref().map_err(|err| println!("oh no..\n{:?}", err));
        }

        if let Err(_) = result {
            println!("{}", "Could not generate response. Resetting context. Memory is preserved.".red());

            while program.context.llm.message_history.len() > 2 {
                if program.context.llm.message_history[1].is_assistant() {
                    let response = program.context.llm.message_history.remove(1);
                    let response = response.content();
                    let response = parse_response(&response)?;
                    let command_response = program.context.llm.message_history.remove(1);
                    let command_response = command_response.content();
                    for plugin in &program.plugins {
                        plugin.cycle.apply_removed_response(&mut program.context, &response, &command_response, true).await?;
                    }
                } else {
                    program.context.llm.message_history.remove(1);
                }
            }

            program.context.llm.message_history.clear();
        }



        println!();
        println!();
    }

    Ok(())
}
use std::error::Error;

use reqwest::Client;
use tokenizers::{Tokenizer, models::bpe::BPE};

use crate::{Plugin, CommandContext, LLMResponse, NotFoundError, create_tokenizer};

pub fn parse_response(response: &str) -> Result<LLMResponse, Box<dyn Error>> {
    let response: LLMResponse = serde_json::from_str(response)?;
    Ok(response)
}

pub async fn run_command(context: &mut CommandContext, response: &LLMResponse, plugins: &[Plugin]) -> Result<String, Box<dyn Error>> {
    let mut out = String::new();

    let request = &response.command;

    let plugin = plugins.iter().find(|plugin| plugin.commands.iter().any(|command| command.name == request.name))
        .ok_or(NotFoundError(format!("Could not find plugin from command name {}", request.name)))?;

    let command = plugin.commands.iter().find(|command| command.name == request.name)
        .ok_or(NotFoundError(format!("Could not find command from command name {}", request.name)))?;

    let results = command.run.invoke(context, request.args.clone()).await?;
    out.push_str(&results);

    Ok(out.trim_end().to_string())
}
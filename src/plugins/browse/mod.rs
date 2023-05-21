use std::{error::Error, fmt::Display, collections::HashMap, fs};
use async_trait::async_trait;
use colored::Colorize;
use readability::extractor;
use reqwest::{Client, header::{USER_AGENT, HeaderMap}};
use textwrap::wrap;

mod extract;

pub use extract::*;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::{Plugin, Command, CommandContext, CommandImpl, EmptyCycle, apply_chunks, PluginData, PluginDataNoInvoke, PluginCycle, invoke, ScriptValue, CommandArgument, Message, CommandResult};

pub struct BrowseData {
    pub client: Client
}

#[derive(Serialize, Deserialize)]
pub struct BrowseRequest {
    pub url: String,
    pub params: Vec<(String, String)>
}

#[async_trait]
impl PluginData for BrowseData {
    async fn apply(&mut self, name: &str, value: Value) -> Result<Value, Box<dyn Error>> {
        match name {
            "browse" => {
                let BrowseRequest { url, params } = serde_json::from_value(value)?;
                let res_result = self.client.get(url).query(&params).send().await?;
                let text = res_result.text().await?;
                
                Ok(text.into())
            }
            _ => {
                Err(Box::new(PluginDataNoInvoke("Browse".to_string(), name.to_string())))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadTextResults {
    text: String
}

#[derive(Debug, Clone)]
pub struct BrowseNoArgError;

impl Display for BrowseNoArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "'browse-article' command did not receive one of its arguments.")
    }
}

impl Error for BrowseNoArgError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoContentError {
    error: String,
    help: String
}

fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = vec![];
    let mut current_chunk = String::new();

    for word in wrap(text, chunk_size) {
        if current_chunk.len() + word.len() > chunk_size {
            chunks.push(current_chunk.trim().to_owned());
            current_chunk = String::new();
        }
        current_chunk.push_str(&word);
    }
    if !current_chunk.is_empty() {
        chunks.push(current_chunk.trim().to_owned());
    }
    chunks
}

#[derive(Serialize, Deserialize)]
pub struct BrowseArgs {
    pub urls: Vec<String>
}

pub async fn browse_urls(ctx: &mut CommandContext, args: ScriptValue) -> Result<String, Box<dyn Error>> {
    let browse_info = ctx.plugin_data.get_data("Browse")?;

    let mut out: Vec<String> = vec![];

    let params: [(&str, &str); 0] = [];
    let args: BrowseArgs = args.parse()?;
        for url in args.urls {
        let content = extractor::scrape(&url)?.text;

        let mut summarized_content = String::new();
        let chunks = chunk_text(&content, 11000);

        let chunk_count = chunks.len();
        let summary_prompt = match chunk_count {
            0..=2 => "Create a three-sentence summary of the text below.",
            _ => "Create a one-sentence summary of the text below."
        }.to_string();

        for (ind, chunk) in chunks.iter().enumerate() {
            println!("<{url}> {} {} / {}", "Summarizing Chunk".green(), ind + 1, chunks.len());

            ctx.agents.fast.llm.message_history.clear();

            ctx.agents.fast.llm.message_history.push(Message::System(summary_prompt.clone()));

            ctx.agents.fast.llm.message_history.push(Message::User(chunk.to_string()));

            let response = ctx.agents.fast.llm.model.get_response(
                &ctx.agents.fast.llm.get_messages(),
                None,
                None
            ).await?;

            summarized_content.push_str(&response);
            summarized_content.push(' ');
        }

        summarized_content = summarized_content.trim().to_string();

        out.push(format!("# {url}\n\n{summarized_content}"));
    }

    Ok(out.join("\n\n"))
}

pub struct BrowseURLs;

#[async_trait]
impl CommandImpl for BrowseURLs {
    async fn invoke(&self, ctx: &mut CommandContext, args: ScriptValue) -> Result<CommandResult, Box<dyn Error>> {
        Ok(CommandResult::Text(browse_urls(ctx, args).await?))
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub struct BrowseCycle;

#[async_trait]
impl PluginCycle for BrowseCycle {
    async fn create_context(&self, context: &mut CommandContext, previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
        Ok(None)
    }

    fn create_data(&self, _: Value) -> Option<Box<dyn PluginData>> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, "SmartGPT v0.0.1".parse().unwrap());
    
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build().unwrap();

        Some(Box::new(BrowseData {
            client
        }))
    }
}

pub fn create_browse() -> Plugin {
    Plugin {
        name: "Browse".to_string(),
        dependencies: vec![],
        cycle: Box::new(BrowseCycle),
        commands: vec![
            Command {
                name: "browse_urls".to_string(),
                purpose: "Browse the paragraph-only content from an exact URL.".to_string(),
                args: vec![
                    CommandArgument::new("urls", "The URL to browse.", "String[]")
                ],
                return_type: "String".to_string(),
                run: Box::new(BrowseURLs)
            }
        ]
    }
}
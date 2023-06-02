use std::{error::Error, fmt::Display};
use async_trait::async_trait;
use colored::Colorize;
use readability::extractor;
use reqwest::{Client, header::{USER_AGENT, HeaderMap}};
use textwrap::wrap;

mod extract;

pub use extract::*;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::{Plugin, Tool, CommandContext, CommandImpl, PluginData, PluginDataNoInvoke, PluginCycle, ScriptValue, ToolArgument, Message, CommandResult, ToolType, LLM};

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
        write!(f, "{}", "'browse-article' tool did not receive one of its arguments.")
    }
}

impl Error for BrowseNoArgError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoContentError {
    error: String,
    help: String
}

fn chunk_text(llm: &LLM, text: &str, chunk_size: usize) -> Result<Vec<String>, Box<dyn Error>> {
    let tokens = llm.get_tokens_from_text(text)?;

    Ok(
        tokens.chunks(3500)
            .map(|token_chunk| token_chunk.join(""))
            .collect()
    )
}

#[derive(Serialize, Deserialize)]
pub struct BrowseArgs {
    pub urls: Vec<String>
}

pub async fn browse_urls(ctx: &mut CommandContext, args: ScriptValue) -> Result<String, Box<dyn Error>> {
    let _browse_info = ctx.plugin_data.get_data("Browse")?;

    let mut out: Vec<String> = vec![];

    let _params: [(&str, &str); 0] = [];
    let args: BrowseArgs = args.parse()?;
        for url in args.urls {
        match extractor::scrape(&url) {
            Ok(resp) => {
                let content = resp.content;

                let mut summarized_content = String::new();
                let chunks = chunk_text(&ctx.agents.fast.llm, &content, 3700)?;
        
                let chunk_count = chunks.len();
                let summary_prompt = match chunk_count {
                    0..=2 => "Create a three-sentence summary of the text below. Be concise.",
                    _ => "Create a one-sentence summary of the text below. Be concise."
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
            },
            Err(err) => {
                out.push(format!("# {url}\n\n[ERROR] {err}"));
            }
        };
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
    async fn create_context(&self, _context: &mut CommandContext, _previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
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
        tools: vec![
            Tool {
                name: "browse_urls".to_string(),
                purpose: "Read the text content from a URL.".to_string(),
                args: vec![
                    ToolArgument::new("urls", r#"[ "url 1", "url 2" ]"#)
                ],
                run: Box::new(BrowseURLs),
                tool_type: ToolType::Resource
            }
        ]
    }
}
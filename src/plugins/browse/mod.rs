use std::{error::Error, fmt::Display, collections::HashMap, fs};
use async_trait::async_trait;
use reqwest::{Client, header::{USER_AGENT, HeaderMap}};

mod extract;

pub use extract::*;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::{Plugin, LLMResponse, Command, CommandContext, CommandImpl, EmptyCycle, apply_chunks, PluginData, PluginDataNoInvoke, PluginCycle, invoke};

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

pub async fn download_article(ctx: &mut CommandContext, args: HashMap<String, String>) -> Result<String, Box<dyn Error>> {
    let browse_info = ctx.plugin_data.get_data("Browse")?;

    let params: [(&str, &str); 0] = [];
    let url = args.get("url").ok_or(BrowseNoArgError)?;   

    let res_result = invoke::<String>(browse_info, "browse", BrowseRequest {
        url: url.to_string(),
        params: params.iter()
            .map(|el| (el.0.to_string(), el.1.to_string()))
            .collect::<Vec<_>>()
    }).await;
    let body = match res_result {
        Ok(res) => {
            if res.len() < 5 {
                return Ok(serde_json::to_string(&NoContentError {
                    error: format!("The URL of \"{url}\" has no content."),
                    help: "Try browsing another website.".to_string()
                })?);
            }

            res
        }
        Err(_) => {
            return Ok(
                format!("Could not browse the website link of \"{url}\". Are you sure this is a valid URL?")
            )
        }
    };

    let content = extract_text_from_html(&body);
    let (content, ..) = apply_chunks(&content, 1, 5000);

    Ok(serde_json::to_string(&content)?)
}

pub struct BrowseArticle;

#[async_trait]
impl CommandImpl for BrowseArticle {
    async fn invoke(&self, ctx: &mut CommandContext, args: HashMap<String, String>) -> Result<String, Box<dyn Error>> {
        download_article(ctx, args).await
    }
}

pub struct BrowseCycle;

#[async_trait]
impl PluginCycle for BrowseCycle {
    async fn create_context(&self, context: &mut CommandContext, previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
        Ok(None)
    }

    async fn apply_removed_response(&self, context: &mut CommandContext, response: &LLMResponse, cmd_output: &str, previous_response: bool) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn create_data(&self, _: Value) -> Option<Box<dyn PluginData>> {
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
                name: "browse-article".to_string(),
                purpose: "Browse a website's paragraph-only content.".to_string(),
                args: vec![
                    ("url".to_string(), "The URL to browse.".to_string())
                ],
                run: Box::new(BrowseArticle)
            }
        ]
    }
}
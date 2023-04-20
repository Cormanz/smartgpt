mod types;

use std::{error::Error, fmt::Display, collections::HashMap};
use async_trait::async_trait;
use regex::Regex;

use select::{document::Document, predicate::Name};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::{CommandContext, CommandImpl, Plugin, EmptyCycle, Command, invoke, BrowseRequest, PluginDataNoInvoke, PluginData, PluginCycle, ScriptValue, CommandArgument};

pub use types::*;

#[derive(Debug, Clone)]
pub struct NewsNoQueryError;

impl Display for NewsNoQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "one of the 'news' commands did not receive enough info.")
    }
}

impl Error for NewsNoQueryError {}

pub async fn ask_news(ctx: &mut CommandContext, query: &str) -> Result<ScriptValue, Box<dyn Error>> {
    let wolfram_info = ctx.plugin_data.get_data("NewsAPI")?;
    let api_key = invoke::<String>(wolfram_info, "get api key", true).await?;
    let api_key: &str = &api_key;

    let params = [
        ("apiKey", api_key),
        ("pageSize", "4"),
        ("q", query)
    ];
    
    let browse_info = ctx.plugin_data.get_data("Browse")?;
    let json = invoke::<String>(browse_info, "browse", BrowseRequest {
        url: "https://newsapi.org/v2/everything".to_string(),
        params: params.iter()
            .map(|el| (el.0.to_string(), el.1.to_string()))
            .collect::<Vec<_>>()
    }).await?;

    let json: News = serde_json::from_str(&json)?;
    let json = serde_json::to_string(&json.articles)?;

    Ok(serde_json::from_str(&json)?)
}

pub async fn news(ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
    let query: String = args.get(0).ok_or(NewsNoQueryError)?.clone().try_into()?;
    let response = ask_news(ctx, &query).await?;
    
    Ok(response)
}

pub struct NewsImpl;

#[async_trait]
impl CommandImpl for NewsImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
        news(ctx, args).await
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct NewsData {
    #[serde(rename = "api key")] pub api_key: String
}

#[async_trait]
impl PluginData for NewsData {
    async fn apply(&mut self, name: &str, value: Value) -> Result<Value, Box<dyn Error>> {
        match name {
            "get api key" => {
                Ok(self.api_key.clone().into())
            }
            _ => {
                Err(Box::new(PluginDataNoInvoke("NewsAPI".to_string(), name.to_string())))
            }
        }
    }
}

pub struct NewsCycle;

#[async_trait]
impl PluginCycle for NewsCycle {
    async fn create_context(&self, context: &mut CommandContext, previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
        Ok(None)
    }

    fn create_data(&self, value: Value) -> Option<Box<dyn PluginData>> {
        let data: NewsData = serde_json::from_value(value).ok()?;
        Some(Box::new(data))
    }
}

pub fn create_news() -> Plugin {
    Plugin {
        name: "NewsAPI".to_string(),
        dependencies: vec![ "Browse".to_string() ],
        cycle: Box::new(NewsCycle),
        commands: vec![
            Command {
                name: "news_search".to_string(),
                purpose: "Search for news articles.".to_string(),
                args: vec![
                    CommandArgument::new("query", "The query to search for.", "String")
                ],
                return_type: "{ title: String, description: String, url: String }[]".to_string(),
                run: Box::new(NewsImpl)
            }
        ]
    }
}
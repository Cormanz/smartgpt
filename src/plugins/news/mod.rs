mod types;

use std::{error::Error, fmt::Display};
use async_trait::async_trait;

use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::{CommandContext, CommandImpl, Plugin, Tool, invoke, BrowseRequest, PluginDataNoInvoke, PluginData, PluginCycle, ScriptValue, ToolArgument, CommandResult, ToolType};

pub use types::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewsRequest {
    pub query: String
}

#[derive(Debug, Clone)]
pub struct NewsNoQueryError;

impl Display for NewsNoQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "one of the 'news' tools did not receive enough info.")
    }
}

impl Error for NewsNoQueryError {}

pub async fn ask_news(ctx: &mut CommandContext, query: &str) -> Result<String, Box<dyn Error>> {
    let news_info = ctx.plugin_data.get_data("NewsAPI")?;
    let api_key = invoke::<String>(news_info, "get api key", true).await?;
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

    let text = json.articles.iter()
        .flat_map(|item| vec![
            format!("# [{}]({})", item.title, item.url),
            item.description.clone()
        ])
        .collect::<Vec<_>>()
        .join("\n");
    
    println!("mhm?: {json:?}");
    println!("hehe: {text}");

    Ok(text)
}

pub async fn news(ctx: &mut CommandContext, args: ScriptValue) -> Result<CommandResult, Box<dyn Error>> {
    let NewsRequest { query } = args.parse()?;
    let response = ask_news(ctx, &query).await?;
    
    Ok(CommandResult::Text(response))
}

pub struct NewsImpl;

#[async_trait]
impl CommandImpl for NewsImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: ScriptValue) -> Result<CommandResult, Box<dyn Error>> {
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
    async fn apply(&mut self, name: &str, _value: Value) -> Result<Value, Box<dyn Error>> {
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
    async fn create_context(&self, _context: &mut CommandContext, _previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
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
        tools: vec![
            Tool {
                name: "news_search".to_string(),
                purpose: "Search for news articles.".to_string(),
                args: vec![
                    ToolArgument::new("query", "query")
                ],
                run: Box::new(NewsImpl),
                tool_type: ToolType::Resource
            }
        ]
    }
}
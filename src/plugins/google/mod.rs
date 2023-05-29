use std::{error::Error, fmt::Display};
use async_trait::async_trait;

mod types;

use serde::{Serialize, Deserialize};
use serde_json::Value;
pub use types::*;

use crate::{Plugin, Tool, CommandContext, CommandImpl, invoke, BrowseRequest, PluginData, PluginDataNoInvoke, PluginCycle, ScriptValue, ToolArgument, CommandResult, ToolType};

#[derive(Debug, Clone)]
pub struct GoogleNoQueryError;

impl Display for GoogleNoQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "'google' tool did not receive a query.")
    }
}

impl Error for GoogleNoQueryError {}

#[derive(Serialize, Deserialize)]
pub struct GoogleArgs {
    pub query: String
}

pub async fn google(ctx: &mut CommandContext, args: ScriptValue) -> Result<String, Box<dyn Error>> {
    let args: GoogleArgs = args.parse()?;

    let wolfram_info = ctx.plugin_data.get_data("Google")?;

    let api_key = invoke::<String>(wolfram_info, "get api key", true).await?;
    let api_key: &str = &api_key;
    
    let cse_id = invoke::<String>(wolfram_info, "get cse id", true).await?;
    let cse_id: &str = &cse_id;

    let params = [
        ("key", api_key),
        ("cx", cse_id),
        ("q", &args.query),
        ("num", "4")
    ];
    
    let browse_info = ctx.plugin_data.get_data("Browse")?;
    let body = invoke::<String>(browse_info, "browse", BrowseRequest {
        url: "https://www.googleapis.com/customsearch/v1".to_string(),
        params: params.iter()
            .map(|el| (el.0.to_string(), el.1.to_string()))
            .collect::<Vec<_>>()
    }).await?;

    let json: SearchResponse = serde_json::from_str(&body)?;

    let text = json.items.iter()
        .flat_map(|item| vec![
            format!("# [{}]({})", item.title, item.link),
            item.snippet.clone()
        ])
        .collect::<Vec<_>>()
        .join("\n");

    Ok(text)
}

pub struct GoogleImpl;

#[async_trait]
impl CommandImpl for GoogleImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: ScriptValue) -> Result<CommandResult, Box<dyn Error>> {
        Ok(CommandResult::Text(google(ctx, args).await?))
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct GoogleData {
    #[serde(rename = "cse id")] pub cse_id: String,
    #[serde(rename = "api key")] pub api_key: String
}

#[async_trait]
impl PluginData for GoogleData {
    async fn apply(&mut self, name: &str, _: Value) -> Result<Value, Box<dyn Error>> {
        match name {
            "get api key" => {
                Ok(self.api_key.clone().into())
            }
            "get cse id" => {
                Ok(self.cse_id.clone().into())
            }
            _ => {
                Err(Box::new(PluginDataNoInvoke("Google".to_string(), name.to_string())))
            }
        }
    }
}

pub struct GoogleCycle;

#[async_trait]
impl PluginCycle for GoogleCycle {
    async fn create_context(&self, _context: &mut CommandContext, _previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
        Ok(None)
    }
    
    fn create_data(&self, value: Value) -> Option<Box<dyn PluginData>> {
        let data: GoogleData = serde_json::from_value(value).ok()?;
        Some(Box::new(data))
    }
}

pub fn create_google() -> Plugin {
    Plugin {
        name: "Google".to_string(),
        dependencies: vec![ "Browse".to_string() ],
        cycle: Box::new(GoogleCycle),
        tools: vec![
            Tool {
                name: "google_search".to_string(),
                purpose: "Gives you a list of URLs from a query.".to_string(),
                args: vec![
                    ToolArgument::new("query", r#""query""#)
                ],
                run: Box::new(GoogleImpl),
                tool_type: ToolType::Resource
            }
        ]
    }
}
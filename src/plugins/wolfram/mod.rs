use std::{error::Error, fmt::Display};
use async_trait::async_trait;
use regex::Regex;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{CommandContext, CommandImpl, Plugin, Tool, BrowseRequest, invoke, PluginData, PluginCycle, PluginDataNoInvoke, ScriptValue, ToolArgument, CommandResult, ToolType};

#[derive(Debug, Clone)]
pub struct WolframNoQueryError;

impl Display for WolframNoQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "one of the 'wolfram' tools did not receive enough info.")
    }
}

impl Error for WolframNoQueryError {}

#[derive(Serialize, Deserialize)]
pub struct WolframArgs {
    pub query: String
}

pub fn extract_text_from_wolfram(html: &str) -> String {
    let re = Regex::new(r#"<plaintext>([^<]+)"#).unwrap();

    let mut text = String::new();

    for cap in re.captures_iter(html) {
        let match_str = cap.get(1).map_or("", |m| m.as_str());
        text.push('\n');
        text.push_str(match_str);
    }

    text.trim().to_string()
}

pub async fn ask_wolfram(ctx: &mut CommandContext, query: &str) -> Result<String, Box<dyn Error>> {
    let wolfram_info = ctx.plugin_data.get_data("Wolfram")?;
    let app_id = invoke::<String>(wolfram_info, "get app id", true).await?;
    let app_id: &str = &app_id;

    let params = [
        ("appid", app_id),
        ("input", query),
        ("podstate", "Result__Step-by-step solution"),
        ("format", "plaintext")
    ];
    
    let browse_info = ctx.plugin_data.get_data("Browse")?;
    let xml = invoke::<String>(browse_info, "browse", BrowseRequest {
        url: "http://api.wolframalpha.com/v2/query".to_string(),
        params: params.iter()
            .map(|el| (el.0.to_string(), el.1.to_string()))
            .collect::<Vec<_>>()
    }).await?; 

    Ok(extract_text_from_wolfram(&xml))
}

pub async fn wolfram(ctx: &mut CommandContext, args: ScriptValue) -> Result<ScriptValue, Box<dyn Error>> {
    let WolframArgs { query } = args.parse()?;
    let response = ask_wolfram(ctx, &query).await?;

    let response = if response.trim().len() > 0 {
        response
    } else {
        "Sorry, but Wolfram Alpha did not understand your query. Please try using another tool.".to_string()
    };
    
    Ok(response.into())
}

pub struct WolframImpl;

#[async_trait]
impl CommandImpl for WolframImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: ScriptValue) -> Result<CommandResult, Box<dyn Error>> {
        Ok(CommandResult::ScriptValue(wolfram(ctx, args).await?))
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct WolframData {
    #[serde(rename = "app id")] pub app_id: String
}

#[async_trait]
impl PluginData for WolframData {
    async fn apply(&mut self, name: &str, _value: Value) -> Result<Value, Box<dyn Error>> {
        match name {
            "get app id" => {
                Ok(self.app_id.clone().into())
            }
            _ => {
                Err(Box::new(PluginDataNoInvoke("Wolfram".to_string(), name.to_string())))
            }
        }
    }
}

pub struct WolframCycle;

#[async_trait]
impl PluginCycle for WolframCycle {
    async fn create_context(&self, _context: &mut CommandContext, _previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
        Ok(None)
    }

    fn create_data(&self, value: Value) -> Option<Box<dyn PluginData>> {
        let data: WolframData = serde_json::from_value(value).ok()?;
        Some(Box::new(data))
    }
}

pub fn create_wolfram() -> Plugin {
    Plugin {
        name: "Wolfram".to_string(),
        dependencies: vec![ "Browse".to_string() ],
        cycle: Box::new(WolframCycle),
        tools: vec![
            Tool {
                name: "wolfram".to_string(),
                purpose: "Ask WolframAlpha to answer a query.".to_string(),
                args: vec![
                    ToolArgument::new("query", r#""query""#)
                ],
                run: Box::new(WolframImpl),
                tool_type: ToolType::Resource
            }
        ]
    }
}
use std::{error::Error, fmt::Display, collections::HashMap};
use async_trait::async_trait;
use regex::Regex;

use select::{document::Document, predicate::Name};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{CommandContext, CommandImpl, Plugin, EmptyCycle, Command, BrowseRequest, invoke, PluginData, PluginCycle, PluginDataNoInvoke, ScriptValue, CommandArgument};

#[derive(Debug, Clone)]
pub struct WolframNoQueryError;

impl Display for WolframNoQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "one of the 'wolfram' commands did not receive enough info.")
    }
}

impl Error for WolframNoQueryError {}

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

pub async fn wolfram(ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
    let query: String = args.get(0).ok_or(WolframNoQueryError)?.clone().try_into()?;
    let response = ask_wolfram(ctx, &query).await?;
    
    Ok(response.into())
}

pub struct WolframImpl;

#[async_trait]
impl CommandImpl for WolframImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
        wolfram(ctx, args).await
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
    async fn apply(&mut self, name: &str, value: Value) -> Result<Value, Box<dyn Error>> {
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
    async fn create_context(&self, context: &mut CommandContext, previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
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
        commands: vec![
            Command {
                name: "wolfram".to_string(),
                purpose: "Ask WolframAlpha to answer a query.".to_string(),
                args: vec![
                    CommandArgument::new("query", "The query to ask Wolfram Alpha", "String")
                ],
                return_type: "String".to_string(),
                run: Box::new(WolframImpl)
            }
        ]
    }
}
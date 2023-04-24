use std::{error::Error, fmt::Display, collections::HashMap};
use async_trait::async_trait;
use regex::Regex;

use select::{document::Document, predicate::Name};

mod types;

use crate::{CommandContext, CommandImpl, Plugin, EmptyCycle, Command, apply_chunks, CommandNoArgError, invoke, BrowseRequest, ScriptValue, CommandArgument};

pub use types::*;

#[derive(Debug, Clone)]
pub struct WikipediaNoPageError;

impl Display for WikipediaNoPageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "one of the 'wikipedia' commands did not find a page.")
    }
}

impl Error for WikipediaNoPageError {}

pub async fn search_wikipedia(ctx: &mut CommandContext, query: &str) -> Result<String, Box<dyn Error>> {
    let params = [
        ("action", "query"),
        ("format", "json"),
        ("list", "search"),
        ("srsearch", query)
    ];
    
    let browse_info = ctx.plugin_data.get_data("Browse")?;
    let json = invoke::<String>(browse_info, "browse", BrowseRequest {
        url: "https://en.wikipedia.org/w/api.php".to_string(),
        params: params.iter()
            .map(|el| (el.0.to_string(), el.1.to_string()))
            .collect::<Vec<_>>()
    }).await?; 

    Ok(json.clone())
}

pub async fn get_wikipedia(ctx: &mut CommandContext, name: &str) -> Result<String, Box<dyn Error>> {
    let params = [
        ("action", "query"),
        ("format", "json"),
        ("prop", "extracts"),
        ("plaintext", "true"),
        ("exintro", "true"),
        ("titles", name)
    ];

    let browse_info = ctx.plugin_data.get_data("Browse")?;
    let json = invoke::<String>(browse_info, "browse", BrowseRequest {
        url: "https://en.wikipedia.org/w/api.php".to_string(),
        params: params.iter()
            .map(|el| (el.0.to_string(), el.1.to_string()))
            .collect::<Vec<_>>()
    }).await?; 
    let json: WikipediaResponse = serde_json::from_str(&json)?;
    let page = json.query.pages.iter().next().ok_or(WikipediaNoPageError)?.1;

    let content = page.extract.clone().unwrap_or("".to_string());

    let (content, length_warning) = apply_chunks(&content, 1, 5000);
    let length_warning = length_warning
        .map(|el| format!("{el}\n\n"));

    let output = WikipediaOutput {
        title: page.title.clone(),
        content
    };

    Ok(output.content.into())
}

pub async fn wikipedia_search(ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
    let query: String = args.get(0).ok_or(CommandNoArgError("wikipedia-search", "query"))?.clone().try_into()?;
    let response = search_wikipedia(ctx, &query).await?;
    let json: ScriptValue = serde_json::from_str(&response)?;
    
    Ok(json)
}

pub async fn wikipedia_get(ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
    let name: String = args.get(0).ok_or(CommandNoArgError("wikipedia-browse", "name"))?.clone().try_into()?;;
    let response = get_wikipedia(ctx, &name).await?;
    
    Ok(response.into())
}

pub struct WikipediaSearchImpl;

#[async_trait]
impl CommandImpl for WikipediaSearchImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
        wikipedia_search(ctx, args).await
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub struct WikipediaBrowseImpl;

#[async_trait]
impl CommandImpl for WikipediaBrowseImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
        wikipedia_get(ctx, args).await
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub fn create_wikipedia() -> Plugin {
    Plugin {
        name: "Wikipedia".to_string(),
        dependencies: vec![ "Browse".to_string() ],
        cycle: Box::new(EmptyCycle),
        commands: vec![
            Command {
                name: "wikipedia_search".to_string(),
                purpose: "Search for wikipedia articles.".to_string(),
                args: vec![
                    CommandArgument::new("query", "The query to search for.", "String")
                ],
                return_type: "{ query: { search: { title: String, extract: String }[] } }".to_string(),
                run: Box::new(WikipediaSearchImpl)
            },
            Command {
                name: "wikipedia_browse".to_string(),
                purpose: "Browse a wikipedia article.".to_string(),
                args: vec![
                    CommandArgument::new("article_name", "The query to search for.", "String")
                ],
                return_type: "String".to_string(),
                run: Box::new(WikipediaBrowseImpl)
            }
        ]
    }
}
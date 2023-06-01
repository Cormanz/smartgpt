use std::{fmt::Display, error::Error};

use serde::{ Serialize, de::DeserializeOwned};
use json5;

use colored::Colorize;
use serde_json::ser::PrettyFormatter;

use crate::{LLM, SmartGPT};

use self::{agents::{processing::find_text_between_braces, worker::run_worker}};

mod agents;
mod run;
mod responses;
mod classify;

pub use run::{Action};
pub use agents::worker::*;

#[derive(Debug)]
pub struct DisallowedAction(Box<dyn Error>);

impl Error for DisallowedAction {}
impl Display for DisallowedAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DisallowedAction(")?;
        write!(f, "{}", self.0)?;
        write!(f, ")")
    }
}

pub fn run_auto(
    smartgpt: &mut SmartGPT, 
    task: &str,
    allow_action: &mut impl FnMut(&Action) -> Result<(), DisallowedAction>,
    listen_to_update: &mut impl FnMut(&Update) -> Result<(), Box<dyn Error>>
) -> Result<String, Box<dyn Error>> {
    let SmartGPT { 
        context, ..
    } = smartgpt;
    let context = context.lock().unwrap();

    drop(context);

    Ok(run_worker(smartgpt, task.clone(), &smartgpt.personality.clone(), allow_action, listen_to_update)?)
}

#[derive(Debug, Clone)]
pub struct CannotParseError;

impl Display for CannotParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "could not parse.")
    }
}

impl Error for CannotParseError {}

pub struct ParsedResponse<T> {
    data: T,
    raw: String
}

pub fn try_parse_yaml<T : DeserializeOwned>(llm: &LLM, tries: usize, max_tokens: Option<u16>, temperature: Option<f32>) -> Result<ParsedResponse<T>, Box<dyn Error>> {
    try_parse_base(llm, tries, max_tokens, temperature, "yml", |str| serde_yaml::from_str(str).map_err(|el| Box::new(el) as Box<dyn Error>))
}

pub fn try_parse_json<T : DeserializeOwned + Serialize>(llm: &LLM, tries: usize, max_tokens: Option<u16>, temperature: Option<f32>) -> Result<ParsedResponse<T>, Box<dyn Error>> {
    for i in 0..tries {
        let response = llm.model.get_response_sync(&llm.get_messages(), max_tokens, temperature)?;
        let processed_response = find_text_between_braces(&response).unwrap_or("None".to_string());

        let formatter = PrettyFormatter::with_indent(b"\t");

        // We use JSON5 to allow for more lenient parsing for models like GPT3.5.
        match json5::from_str::<T>(&processed_response) {
            Ok(data) => {
                // We serialize it back to JSON itself to help GPT3.5 maintain consistency.

                let mut buf = vec![];
                let mut ser = serde_json::ser::Serializer::with_formatter(&mut buf, formatter);
                data.serialize(&mut ser).unwrap();
                let pretty_response = String::from_utf8(buf).unwrap();

                return Ok(ParsedResponse {
                    data,
                    raw: format!("```json\n{pretty_response}\n```")  
                })
            },
            Err(err) => {
                println!("{}", format!("Try {} failed.", i + 1).red());
                println!("{response}");
                println!("{err}");
            }
        }
    }
    
    Err(Box::new(CannotParseError))
}

pub fn try_parse_base<T : DeserializeOwned>(llm: &LLM, tries: usize, max_tokens: Option<u16>, temperature: Option<f32>, lang: &str, parse: impl Fn(&str) -> Result<T, Box<dyn Error>>) -> Result<ParsedResponse<T>, Box<dyn Error>> {
    for i in 0..tries {
        let response = llm.model.get_response_sync(&llm.get_messages(), max_tokens, temperature)?;
        let processed_response = response.trim();
        let processed_response = processed_response.strip_prefix("```")
            .unwrap_or(&processed_response)
            .to_string();
        let processed_response = processed_response.strip_prefix(&format!("{lang}"))
            .unwrap_or(&response)
            .to_string();
        let processed_response = processed_response.strip_suffix("```")
            .unwrap_or(&processed_response)
            .to_string();
        let processed_response = processed_response.trim().to_string();
        match parse(&processed_response) {
            Ok(data) => {
                return Ok(ParsedResponse {
                    data,
                    raw: format!("```{lang}\n{processed_response}\n```")  
                })
            },
            Err(err) => {
                println!("{}", format!("Try {} failed.", i + 1).red());
                println!("{response}");
                println!("{err}");
            }
        }
    }
    
    Err(Box::new(CannotParseError))
}
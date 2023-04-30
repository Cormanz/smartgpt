mod manager;
mod boss;
mod employee;
mod minion;

use std::{error::Error, fmt::Display};

use colored::Colorize;

pub use manager::*;
pub use boss::*;
pub use employee::*;
pub use minion::*;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_yaml::Value;

use crate::LLM;

#[derive(Debug, Clone)]
pub struct CannotParseError;

impl Display for CannotParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "could not parse.")
    }
}



impl Error for CannotParseError {}

pub const LINE_WRAP: usize = 12;

pub fn process_response(text: &str, line_wrap: usize) -> String {
    let lines: Vec<String> = text
        .lines()
        .flat_map(|line| {
            let indent = line
                .chars()
                .take_while(|&c| c.is_whitespace())
                .collect::<String>();
            line.trim()
                .split(' ')
                .collect::<Vec<_>>()
                .chunks(line_wrap)
                .map(|el| {
                    let words = el.join(" ");
                    format!("{}{}", indent, words)
                })
                .collect::<Vec<_>>()
        })
        .collect();
    lines.join("\n")
}
pub fn test() {
    let e = String::from("e");
    let yaml: Result<Value, _> = serde_yaml::from_str(&e);
}

pub fn try_parse_yaml<T : DeserializeOwned>(llm: &LLM, tries: usize, max_tokens: Option<u16>) -> Result<(String, T), Box<dyn Error>> {
    try_parse_base(llm, tries, max_tokens, "yml", |str| serde_yaml::from_str(str).map_err(|el| Box::new(el) as Box<dyn Error>))
}

pub fn try_parse_json<T : DeserializeOwned>(llm: &LLM, tries: usize, max_tokens: Option<u16>) -> Result<(String, T), Box<dyn Error>> {
    try_parse_base(llm, tries, max_tokens, "json", |str| serde_json::from_str(str).map_err(|el| Box::new(el) as Box<dyn Error>))
}

pub fn try_parse_base<T : DeserializeOwned>(llm: &LLM, tries: usize, max_tokens: Option<u16>, lang: &str, parse: impl Fn(&str) -> Result<T, Box<dyn Error>>) -> Result<(String, T), Box<dyn Error>> {
    for i in 0..tries {
        let response = llm.model.get_response_sync(&llm.get_messages(), max_tokens, None)?;
        let processed_response = response.trim();
        let processed_response = processed_response.strip_prefix(&format!("```{lang}"))
            .unwrap_or(&response)
            .to_string();
        let processed_response = processed_response.strip_prefix("```")
            .unwrap_or(&processed_response)
            .to_string();
        let processed_response = processed_response.strip_suffix("```")
            .unwrap_or(&processed_response)
            .to_string();
        match parse(&processed_response) {
            Ok(yaml) => {
                return Ok((response, yaml));
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

#[derive(Copy, Clone)]
pub enum Agent {
    Manager,
    Boss,
    Employee
}

#[derive(Serialize, Deserialize)]
pub struct Choice {
    pub choice: String
}
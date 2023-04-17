mod manager;
mod boss;
mod employee;

use std::{error::Error, fmt::Display};

pub use manager::*;
pub use boss::*;
pub use employee::*;
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
    let lines: Vec<String> = text.split("\n")
        .flat_map(|line| line.split(" ")
            .map(|el| el.to_string())
            .collect::<Vec<_>>()
            .chunks(line_wrap)
            .map(|el| el.join(" "))
            .collect::<Vec<_>>()
        )
        .map(|el| format!("    {el}"))
        .collect();
    lines.join("\n")
}

pub fn test() {
    let e = String::from("e");
    let yaml: Result<Value, _> = serde_yaml::from_str(&e);
}

pub async fn try_parse<T : DeserializeOwned>(llm: &LLM, tries: usize) -> Result<(String, T), Box<dyn Error>> {
    for _ in 0..tries {
        let response = llm.model.get_response(&llm.get_messages()).await?;
        if let Ok(yaml) = serde_yaml::from_str(&response) {
            return Ok((response, yaml));
        }
    }
    
    Err(Box::new(CannotParseError))
}

pub enum Agent {
    Manager,
    Boss,
    Employee
}

#[derive(Serialize, Deserialize)]
pub struct Choice {
    pub choice: String
}
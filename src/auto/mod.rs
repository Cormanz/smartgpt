use std::{fmt::Display, error::Error};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_yaml::Value;

use colored::Colorize;

use crate::{LLM, ProgramInfo};

use self::{employee::run_employee, manager::run_manager};

mod employee;
mod run;
mod findings;
mod manager;

pub fn run_auto(program: &mut ProgramInfo) -> Result<(), Box<dyn Error>> {
    let ProgramInfo { 
        context, task, ..
    } = program;
    let mut context = context.lock().unwrap();

    let task = task.clone();
    let has_manager = context.agents.managers.len() >= 1;

    drop(context);

    if has_manager {
        run_manager(program, 0, &task.clone(), |_| {})
    } else {
        run_employee(program, &task.clone(), |_| {})
    }
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

pub fn try_parse_yaml<T : DeserializeOwned>(llm: &LLM, tries: usize, max_tokens: Option<u16>) -> Result<ParsedResponse<T>, Box<dyn Error>> {
    try_parse_base(llm, tries, max_tokens, "yml", |str| serde_yaml::from_str(str).map_err(|el| Box::new(el) as Box<dyn Error>))
}

pub fn try_parse_json<T : DeserializeOwned>(llm: &LLM, tries: usize, max_tokens: Option<u16>) -> Result<ParsedResponse<T>, Box<dyn Error>> {
    try_parse_base(llm, tries, max_tokens, "json", |str| serde_json::from_str(str).map_err(|el| Box::new(el) as Box<dyn Error>))
}

pub fn try_parse_base<T : DeserializeOwned>(llm: &LLM, tries: usize, max_tokens: Option<u16>, lang: &str, parse: impl Fn(&str) -> Result<T, Box<dyn Error>>) -> Result<ParsedResponse<T>, Box<dyn Error>> {
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
            Ok(data) => {
                return Ok(ParsedResponse {
                    data,
                    raw: response    
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
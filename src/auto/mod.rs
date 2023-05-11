use std::{fmt::Display, error::Error};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_yaml::Value;
use json5;

use colored::Colorize;

use crate::{LLM, ProgramInfo, Message, format_prompt};

use agents::{employee::run_employee, manager::run_manager};

use self::{responses::{ask_for_responses, ask_for_assistant_response}, classify::is_task, agents::processing::find_text_between_braces};

mod agents;
mod run;
mod responses;
mod classify;

pub fn run_task_auto(program: &mut ProgramInfo, task: &str) -> Result<String, Box<dyn Error>> {
    let ProgramInfo { 
        context, ..
    } = program;
    let context = context.lock().unwrap();

    let has_manager = context.agents.managers.len() >= 1;

    drop(context);

    if has_manager {
        run_manager(program, 0, task.clone(), ask_for_responses)?
    } else {
        run_employee(program, task.clone(), ask_for_responses)?
    }
}

pub fn run_assistant_auto(program: &mut ProgramInfo, messages: &[Message], request: &str) -> Result<String, Box<dyn Error>> {
    let ProgramInfo { 
        context, ..
    } = program;
    let mut context = context.lock().unwrap();

    let mut new_messages = messages.to_vec();
    new_messages.push(Message::User(format!(
r#"Summarize the conversation."#)));

    let conversation_context = match messages.len() {
        0 => "No conversation context.".to_string(),
        _ => context.agents.fast.llm.model.get_response_sync(
            &new_messages, Some(300), None
        )?
    };

    drop(context);
    if is_task(program, request)? {
        println!("{}", "Running task...".green());

        let ProgramInfo { 
            context, ..
        } = program;
        let mut context = context.lock().unwrap();

        let has_manager: bool = context.agents.managers.len() >= 1;

        let mut task = request.trim().to_string();
        task = format!(
"Given this relevant conversation context: {conversation_context}

Generate a response to this request: {task}");
        
        drop(context);
    
        if has_manager {
            run_manager(program, 0, &task.clone(), |llm| ask_for_assistant_response(llm, &conversation_context, &request))?
        } else {
            run_employee(program, &task.clone(), |llm| ask_for_assistant_response(llm, &conversation_context, &request))?
        }
    } else {
        let ProgramInfo { 
            context, ..
        } = program;
        let mut context = context.lock().unwrap();

        context.agents.fast.llm.prompt.clear();
        context.agents.fast.llm.message_history.clear();
        
        context.agents.fast.llm.prompt.push(Message::System(format!(
r#"Respond in this conversation context:

{conversation_context}"#
        )));

        context.agents.fast.llm.message_history.push(Message::User(request.to_string()));

        context.agents.fast.llm.model.get_response_sync(
            &context.agents.fast.llm.get_messages(),
            Some(200),
            None
        )
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

pub fn try_parse_json<T : DeserializeOwned + Serialize>(llm: &LLM, tries: usize, max_tokens: Option<u16>) -> Result<ParsedResponse<T>, Box<dyn Error>> {
    for i in 0..tries {
        let response = llm.model.get_response_sync(&llm.get_messages(), max_tokens, None)?;
        let processed_response = find_text_between_braces(&response).unwrap_or("None".to_string());

        // We use JSON5 to allow for more lenient parsing for models like GPT3.5.
        match json5::from_str::<T>(&processed_response) {
            Ok(data) => {
                // We serialize it back to JSON itself to help GPT3.5 maintain consistency.
                let pretty_response = serde_json::to_string_pretty(&data)?;

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

pub fn try_parse_base<T : DeserializeOwned>(llm: &LLM, tries: usize, max_tokens: Option<u16>, lang: &str, parse: impl Fn(&str) -> Result<T, Box<dyn Error>>) -> Result<ParsedResponse<T>, Box<dyn Error>> {
    for i in 0..tries {
        let response = llm.model.get_response_sync(&llm.get_messages(), max_tokens, None)?;
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
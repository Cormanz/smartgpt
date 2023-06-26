mod chatgpt;
mod local;

pub use chatgpt::*;
pub use local::*;
use serde::Serialize;
use tokio::runtime::Runtime;

use std::{error::Error, fmt::Display};

use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ModelLoadError(pub String);

impl Display for ModelLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ModelLoadError({:?})", self.0)
    }
}

impl Error for ModelLoadError {}

#[derive(Clone, Debug)]
pub enum Message {
    User(String),
    Assistant(String),
    System(String),
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let header = match self {
            Self::Assistant(_) => "ASSISTANT",
            Self::System(_) => "SYSTEM",
            Self::User(_) => "USER",
        };

        write!(f, "-- {header} --\n")?;
        write!(f, "{}", self.content())
    }
}

impl Message {
    pub fn is_user(&self) -> bool {
        match self {
            Message::User(_) => true,
            _ => false,
        }
    }

    pub fn is_assistant(&self) -> bool {
        match self {
            Message::Assistant(_) => true,
            _ => false,
        }
    }

    pub fn is_system(&self) -> bool {
        match self {
            Message::System(_) => true,
            _ => false,
        }
    }

    pub fn content(&self) -> &str {
        match self {
            Message::User(content) => content,
            Message::Assistant(content) => content,
            Message::System(content) => content,
        }
    }

    pub fn set_content(&mut self, new_content: &str) {
        match self {
            Message::User(content) => *content = new_content.to_string(),
            Message::Assistant(content) => *content = new_content.to_string(),
            Message::System(content) => *content = new_content.to_string(),
        }
    }
}

#[async_trait]
pub trait LLMModel: Send + Sync {
    async fn get_response(
        &self,
        messages: &[Message],
        max_tokens: Option<u16>,
        temperature: Option<f32>,
    ) -> Result<String, Box<dyn Error>>;
    async fn get_base_embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn Error>>;

    fn get_token_count(&self, text: &[Message]) -> Result<usize, Box<dyn Error>>;
    fn get_token_limit(&self) -> usize;

    fn get_tokens_remaining(&self, text: &[Message]) -> Result<usize, Box<dyn Error>> {
        Ok(self.get_token_limit() - self.get_token_count(text)?)
    }

    fn get_response_sync(
        &self,
        messages: &[Message],
        max_tokens: Option<u16>,
        temperature: Option<f32>,
    ) -> Result<String, Box<dyn Error>> {
        let rt = Runtime::new()?;
        rt.block_on(async { self.get_response(messages, max_tokens, temperature).await })
    }
    fn get_base_embed_sync(&self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        let rt = Runtime::new()?;
        rt.block_on(async { self.get_base_embed(text).await })
    }

    fn get_tokens_from_text(&self, text: &str) -> Result<Vec<String>, Box<dyn Error>>;
}

#[async_trait]
pub trait LLMProvider {
    fn is_enabled(&self) -> bool;
    fn get_name(&self) -> &str;
    fn create(&self, value: Value) -> Result<Box<dyn LLMModel>, Box<dyn Error>>;
}

pub struct LLM {
    pub prompt: Vec<Message>,
    pub end_prompt: Vec<Message>,
    pub message_history: Vec<Message>,
    pub model: Box<dyn LLMModel>,
}

impl LLM {
    pub fn from_provider<T: Serialize>(
        provider: impl LLMProvider,
        config: T,
    ) -> Result<LLM, Box<dyn Error>> {
        let model = provider.create(serde_json::to_value(config)?)?;

        Ok(Self::new(model))
    }

    pub fn new(model: Box<dyn LLMModel>) -> LLM {
        LLM {
            prompt: vec![],
            end_prompt: vec![],
            message_history: vec![],
            model,
        }
    }

    pub fn get_tokens_remaining(&self, messages: &[Message]) -> Result<usize, Box<dyn Error>> {
        self.model.get_tokens_remaining(messages)
    }

    pub fn crop_to_tokens_remaining(&mut self, token_buffer: usize) -> Result<(), Box<dyn Error>> {
        while token_buffer > self.get_tokens_remaining(&self.get_messages())? {
            if !self.message_history.is_empty() {
                self.message_history.remove(0);
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn get_tokens_from_text(&self, text: &str) -> Result<Vec<String>, Box<dyn Error>> {
        self.model.get_tokens_from_text(text)
    }

    pub fn get_messages(&self) -> Vec<Message> {
        let mut messages = self.prompt.clone();
        messages.extend(self.message_history.clone());
        messages.extend(self.end_prompt.clone());
        messages
    }

    pub fn get_messages_additional(
        &self,
        additional_history: impl IntoIterator<Item = Message>,
    ) -> Vec<Message> {
        let mut messages = self.prompt.clone();
        messages.extend(self.message_history.clone());
        messages.extend(additional_history);
        messages.extend(self.end_prompt.clone());
        messages
    }

    pub fn clear_history(&mut self) {
        self.prompt.clear();
        self.end_prompt.clear();
        self.message_history.clear();
    }
}

pub fn format_prompt(messages: &[Message]) -> String {
    let mut out = String::new();

    for message in messages {
        out.push_str(&format!(
            "{}: {}",
            match message {
                Message::System(_) => "HUMAN",
                Message::User(_) => "HUMAN",
                Message::Assistant(_) => "ASSISTANT",
            },
            message.content()
        ));
        out.push_str("\n");
    }

    out.push_str("ASSISTANT: ");

    out
}

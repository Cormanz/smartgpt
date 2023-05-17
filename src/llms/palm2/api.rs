use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::Message;

const BASE_URL: &str = "https://generativelanguage.googleapis.com";

#[derive(Serialize, Deserialize, Debug)]
pub struct CitationSource {
    pub start_index: i32,
    pub end_index: i32,
    pub uri: String,
    pub license: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CitationMetadata {
    pub citation_sources: Vec<CitationSource>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PALMMessage {
    pub author: String,
    pub content: String,
    pub citation_metadata: Option<CitationMetadata>,
}

impl From<Message> for PALMMessage {
    fn from(message: Message) -> Self {
        let content = match message {
            Message::User(content) | Message::Assistant(content) | Message::System(content) => content,
        };

        PALMMessage {
            author: String::from("SmartGPT"),
            content,
            citation_metadata: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Example {
    pub input: PALMMessage,
    pub output: PALMMessage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessagePrompt {
    pub context: String,
    pub examples: Vec<Example>,
    pub messages: Vec<PALMMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GCPModel {
    pub name: String,
    pub base_model_id: String,
    pub version: String,
    pub display_name: String,
    pub description: String,
    pub input_token_limit: i32,
    pub output_token_limit: i32,
    pub supported_generation_methods: Vec<String>,
    pub temperature: f64,
    pub top_p: f64,
    pub top_k: i32,
}

pub struct ApiClient {
    client: Client,
    base_url: String
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://generativelanguage.googleapis.com".to_string()
        }
    }

    pub async fn count_message_tokens(&self, model: &str, message: &str) -> Result<u32, Box<dyn Error>> {
        let url = format!("{}/v1beta2/{}:countMessageTokens", self.base_url, model);
        let response = self.client.post(&url).body(message.to_string()).send().await?;
        let token_count: u32 = response.json().await?;
        Ok(token_count)
    }

    pub async fn embed_text(&self, model: &str, message: &str) -> Result<String, Box<dyn Error>> {
        let url = format!("{}/v1beta2/{}:embedText", self.base_url, model);
        let response = self.client.post(&url).body(message.to_string()).send().await?;
        let embedding: String = response.json().await?;
        Ok(embedding)
    }

    pub async fn generate_message(&self, model: &str, prompt: MessagePrompt) -> Result<String, Box<dyn Error>> {
        let url = format!("{}/v1beta2/{}:generateMessage", self.base_url, model);
        let response = self.client.post(&url).json(&prompt).send().await?;
        let message: String = response.json().await?;
        Ok(message)
    }

    pub async fn generate_text(&self, model: &str, message: &str) -> Result<String, Box<dyn Error>> {
        let url = format!("{}/v1beta2/{}:generateText", self.base_url, model);
        let response = self.client.post(&url).body(message.to_string()).send().await?;
        let text: String = response.json().await?;
        Ok(text)
    }

    pub async fn get_model(&self, name: &str) -> Result<GCPModel, Box<dyn Error>> {
        let url = format!("{}/v1beta2/{}", self.base_url, name);
        let response = self.client.get(&url).send().await?;
        let model: GCPModel = response.json().await?;
        Ok(model)
    }

    pub async fn list_models(&self) -> Result<Vec<GCPModel>, Box<dyn Error>> {
        let url = format!("{}/v1beta2/models", self.base_url);
        let response = self.client.get(&url).send().await?;
        let models: Vec<GCPModel> = response.json().await?;
        Ok(models)
    }
}
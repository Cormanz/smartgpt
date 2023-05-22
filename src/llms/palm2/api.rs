use reqwest::{Client};
use std::error::Error;

use crate::{CountTokensRequest, TokenCountResponse, EmbedTextRequest, EmbeddingResponse, Embedding, MessagePrompt, GenerateMessageResponse, GenerateTextRequest, GenerateTextResponse, GCPModel, ListModelResponse};

pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: String
}

impl ApiClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key
        }
    }

    pub async fn count_message_tokens(&self, model: &str, message: CountTokensRequest) -> Result<TokenCountResponse, Box<dyn Error>> {
        let url = format!("{}/v1beta2/models/{}:countMessageTokens?key={}", self.base_url, model, self.api_key);
        let response = self.client.post(&url).json(&message).send().await?;

        let token_count: TokenCountResponse = response.json().await?;
        Ok(token_count)
    }

    pub async fn embed_text(&self, model: &str, message: EmbedTextRequest) -> Result<Vec<f32>, Box<dyn Error>> {
        let url = format!("{}/v1beta2/models/{}:embedText?key={}", self.base_url, model, self.api_key);
        let response = self.client.post(&url).json(&message).send().await?;

        let embedding: EmbeddingResponse = response.json().await?;
        Ok(embedding.embedding.unwrap_or(Embedding {
            value: vec![]
        }).value)
    }

    pub async fn generate_message(&self, model: &str, prompt: MessagePrompt) -> Result<GenerateMessageResponse, Box<dyn Error>> {
        let url = format!("{}/v1beta2/models/{}:generateMessage?key={}", self.base_url, model, self.api_key);
        let response = self.client.post(&url).json(&prompt).send().await?;

        let message: GenerateMessageResponse = response.json().await?;
        Ok(message)
    }

    pub async fn generate_text(&self, model: &str, message: GenerateTextRequest) -> Result<GenerateTextResponse, Box<dyn Error>> {
        let url = format!("{}/v1beta2/models/{}:generateText?key={}", self.base_url, model, self.api_key);
        let response = self.client.post(&url).json(&message).send().await?;

        let text_response: GenerateTextResponse = response.json().await?;
        Ok(text_response)
    }

    pub async fn get_model(&self, name: &str) -> Result<GCPModel, Box<dyn Error>> {
        let url = format!("{}/v1beta2/models/{}?key={}", self.base_url, name, self.api_key);
        let response = self.client.get(&url).send().await?;

        let model: GCPModel = response.json().await?;
        Ok(model)
    }

    pub async fn list_models(&self) -> Result<ListModelResponse, Box<dyn Error>> {
        let url = format!("{}/v1beta2/models?key={}", self.base_url, self.api_key);
        let response = self.client.get(&url).send().await?;

        let models: ListModelResponse = response.json().await?;
        Ok(models)
    }
}
use std::error::Error;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tokio::runtime::Runtime;

use crate::{LLMProvider, Message, LLMModel, ApiClient, MessagePrompt, PALMMessage, GenerateTextRequest, TextPrompt, EmbedTextRequest, CountTokensRequest, GenerateTextResponse};

pub struct PALM2 {
    pub model: String,
    pub embedding_model: String,
    pub client: ApiClient
}

#[async_trait]
impl LLMModel for PALM2 {
    async fn get_response(&self, messages: &[Message], max_tokens: Option<u16>, temperature: Option<f32>) -> Result<String, Box<dyn Error>> {
        let palm_messages_string = messages
            .iter()
            .map(|el| el.content())
            .collect::<Vec<&str>>()
            .join(" ");

        let text_request = GenerateTextRequest {
            prompt: TextPrompt {
                text: palm_messages_string
            },
            safety_settings: vec![],
            stop_sequences: vec![],
            temperature: temperature.unwrap_or(1.0) as f64,
            candidate_count: 1,
            max_output_tokens: max_tokens.unwrap_or(1000) as i32,
            top_p: 0.95,
            top_k: 40,
        };

        let response_message: GenerateTextResponse = self.client.generate_text(&self.model, text_request).await?;

        let response = response_message.candidates.unwrap_or(vec![]);

        let response_text = response
            .iter()
            .map(|el| el.output.clone())
            .collect::<Vec<String>>()
            .join(" ");

        Ok(response_text)
    }
    
    async fn get_base_embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        let embedding_response = self.client.embed_text(&self.embedding_model, 
            EmbedTextRequest { 
                text: text.to_string() 
            }).await?;
    
        Ok(embedding_response)
    }

    fn get_tokens_remaining(&self, messages: &[Message]) -> Result<usize, Box<dyn Error>> {
        let all_messages: Vec<PALMMessage> = messages.iter().map(|el| PALMMessage {
            author: "".to_string(),
            content: el.content().to_string(),
            citation_metadata: None
        }
        ).collect::<Vec<PALMMessage>>();

        let count_tokens_request = CountTokensRequest {
            prompt: MessagePrompt { context: "".to_string(), examples: vec![], messages: all_messages }
        };

        let runtime = tokio::runtime::Runtime::new()?;
        
        let gcp_model = runtime.block_on(self.client.get_model(&self.model))?;
        let token_count = runtime.block_on(self.client.count_message_tokens(&self.model, count_tokens_request))?;
        let max_tokens = gcp_model.input_token_limit;

        let tokens_remaining = max_tokens.checked_sub(token_count as i32)
            .ok_or_else(|| "Token count exceeded the maximum limit.")?;

        Ok(tokens_remaining as usize)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PALM2Config {    
    #[serde(rename = "api key")] pub api_key: String,
    pub model: Option<String>,
    #[serde(rename = "api base")] pub api_base: Option<String>,
    #[serde(rename = "embedding model")] pub embedding_model: Option<String>,
}

pub struct PALM2Provider;

#[async_trait]
impl LLMProvider for PALM2Provider {
    fn is_enabled(&self) -> bool {
        true
    }

    fn get_name(&self) -> &str {
        "palm2"
    }

    fn create(&self, value: Value) -> Result<Box<dyn LLMModel>, Box<dyn Error>> {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        
        let config: PALM2Config = serde_json::from_value(value)?;

        let client = ApiClient::new(config.api_base.unwrap_or("https://generativelanguage.googleapis.com".to_owned()), config.api_key);

        // Easy way to immediately test api call on startup
        let models_response = rt.block_on(async {
            client.generate_text("text-bison-001", GenerateTextRequest { 
                prompt: TextPrompt {
                    text: "hello PALM how's life".to_string()
                },
                safety_settings: vec![],
                stop_sequences: vec![],
                temperature: 1.0 as f64,
                candidate_count: 1,
                max_output_tokens: 1000 as i32,
                top_p: 0.95,
                top_k: 40,
            }).await
        })?;
        
        eprintln!("model: {:?}", models_response);

        Ok(Box::new(PALM2 {
            model: config.model.unwrap_or("text-bison-001".to_string()),
            embedding_model: config.embedding_model.unwrap_or("embedding-gecko-001".to_string()),
            client
        }))
    }
}

pub fn create_model_palm2() -> Box<dyn LLMProvider> {
    Box::new(PALM2Provider)
}


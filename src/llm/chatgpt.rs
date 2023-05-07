// Import the required libraries and modules
use std::error::Error;

use async_openai::{Client, types::{CreateChatCompletionResponse, CreateChatCompletionRequest, ChatCompletionRequestMessage, Role, CreateEmbeddingRequest, EmbeddingInput}, error::OpenAIError, Chat};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tiktoken_rs::async_openai::get_chat_completion_max_tokens;

use crate::{LLMProvider, Message, LLMModel};

// Define the ChatGPT struct
pub struct ChatGPT {
    pub model: String,
    pub embedding_model: String,
    pub client: Client
}

// Implement LLMModel trait for ChatGPT
#[async_trait]
impl LLMModel for ChatGPT {
    // Get a response from the ChatGPT model
    async fn get_response(&self, messages: &[Message], max_tokens: Option<u16>, temperature: Option<f32>) -> Result<String, Box<dyn Error>> {
        let mut request = CreateChatCompletionRequest::default();

        request.model = self.model.clone();
        request.messages = messages
            .iter()
            .map(|el| el.clone().into())
            .collect::<Vec<_>>();

        request.temperature = temperature;
        request.max_tokens = max_tokens;
        
        let response: CreateChatCompletionResponse = self.client
            .chat()
            .create(request.clone())
            .await?;

        Ok(response.choices[0].message.content.clone())
    }
    
    // Get base embed for a given text
    async fn get_base_embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        let embeddings = self.client.embeddings().create(CreateEmbeddingRequest {
            model: self.embedding_model.clone(),
            user: None,
            input: EmbeddingInput::String(text.to_string())
        }).await?;
    
        Ok(embeddings.data[0].embedding.clone())
    }

    // Get the number of tokens remaining
    fn get_tokens_remaining(&self, messages: &[Message]) -> Result<usize, Box<dyn Error>> {
        let messages: Vec<ChatCompletionRequestMessage> = messages
            .iter()
            .map(|el| el.clone().into())
            .collect::<Vec<_>>();

        let tokens = get_chat_completion_max_tokens(&self.model, &messages)?;
        Ok(tokens)
    }
}

// Define the ChatGPTConfig struct
#[derive(Serialize, Deserialize)]
pub struct ChatGPTConfig {    
    #[serde(rename = "api key")] pub api_key: String,
    pub model: Option<String>,
    #[serde(rename = "embedding model")] pub embedding_model: Option<String>,
}

// Define the ChatGPTProvider struct
pub struct ChatGPTProvider;

// Implement LLMProvider trait for ChatGPTProvider
#[async_trait]
impl LLMProvider for ChatGPTProvider {
    fn is_enabled(&self) -> bool {
        true
    }

    fn get_name(&self) -> &str {
        "chatgpt"
    }

    fn create(&self, value: Value) -> Result<Box<dyn LLMModel>, Box<dyn Error>> {
        let config: ChatGPTConfig = serde_json::from_value(value)?;

        Ok(Box::new(ChatGPT {
            model: config.model.unwrap_or("gpt-3.5-turbo".to_string()),
            embedding_model: config.embedding_model.unwrap_or("text-embedding-ada-002".to_string()),
            client: Client::new().with_api_key(config.api_key.clone())
        }))
    }
}

// Function to create a ChatGPT model
pub fn create_model_chatgpt() -> Box<dyn LLMProvider> {
    Box::new(ChatGPTProvider)
}

// Implement the From trait to convert a Message into a ChatCompletionRequestMessage
impl From<Message> for ChatCompletionRequestMessage {
    fn from(value: Message) -> Self {
        match value {
            Message::User(text) => {
                ChatCompletionRequestMessage {
                    role: Role::User,
                    content: text,
                    name: None
                }
            },
            Message::Assistant(text) => {
                ChatCompletionRequestMessage {
                    role: Role::Assistant,
                    content: text,
                    name: None
                }
            },
            Message::System(text) => {
                ChatCompletionRequestMessage {
                    role: Role::System,
                    content: text,
                    name: None
                }
            }
        }
    }
}

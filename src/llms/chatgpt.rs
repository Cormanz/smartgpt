use std::error::Error;

use async_openai::{Client, types::{CreateChatCompletionResponse, CreateChatCompletionRequest, ChatCompletionRequestMessage, Role, CreateEmbeddingRequest, EmbeddingInput}};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tiktoken_rs::{async_openai::{get_chat_completion_max_tokens, num_tokens_from_messages}, model::get_context_size, cl100k_base, r50k_base};

use crate::{LLMProvider, Message, LLMModel};

pub struct ChatGPT {
    pub model: String,
    pub embedding_model: String,
    pub client: Client
}

#[async_trait]
impl LLMModel for ChatGPT {
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

    fn get_token_count(&self, messages: &[Message]) -> Result<usize, Box<dyn Error>> {
        let messages: Vec<ChatCompletionRequestMessage> = messages
            .iter()
            .map(|el| el.clone().into())
            .collect::<Vec<_>>();
        
        Ok(num_tokens_from_messages(&self.model, &messages)?)
    }

    fn get_token_limit(&self) -> usize {
        get_context_size(&self.model)
    }
    
    async fn get_base_embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        let embeddings = self.client.embeddings().create(CreateEmbeddingRequest {
            model: self.embedding_model.clone(),
            user: None,
            input: EmbeddingInput::String(text.to_string())
        }).await?;
    
        Ok(embeddings.data[0].embedding.clone())
    }

    fn get_tokens_remaining(&self, messages: &[Message]) -> Result<usize, Box<dyn Error>> {
        let messages: Vec<ChatCompletionRequestMessage> = messages
            .iter()
            .map(|el| el.clone().into())
            .collect::<Vec<_>>();

        let tokens = get_chat_completion_max_tokens(&self.model, &messages)?;
        Ok(tokens)
    }

    fn get_tokens_from_text(&self, text: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let bpe = r50k_base()?;
        let tokens = bpe.encode_ordinary(text).iter()
            .flat_map(|&token| bpe.decode(vec![ token ]))
            .collect::<Vec<_>>();

        Ok(tokens)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChatGPTConfig {    
    #[serde(rename = "api key")] pub api_key: String,
    pub model: Option<String>,
    #[serde(rename = "api base")] pub api_base: Option<String>,
    #[serde(rename = "embedding model")] pub embedding_model: Option<String>,
}

impl Default for ChatGPTConfig {
    fn default() -> Self {
        ChatGPTConfig {
            api_key: "Invalid API Key".to_string(),
            model: None,
            api_base: None,
            embedding_model: None
        }
    }
}

pub struct ChatGPTProvider;

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
            client: Client::new().with_api_base(config.api_base.unwrap_or("https://api.openai.com/v1".to_owned())).with_api_key(config.api_key.clone())
        }))
    }
}

pub fn create_model_chatgpt() -> Box<dyn LLMProvider> {
    Box::new(ChatGPTProvider)
}

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

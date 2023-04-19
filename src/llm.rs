use std::error::Error;

use async_openai::{Client, types::{CreateChatCompletionResponse, CreateChatCompletionRequest, ChatCompletionRequestMessage, Role, CreateEmbeddingRequest, EmbeddingInput}, error::OpenAIError, Chat};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tiktoken_rs::async_openai::get_chat_completion_max_tokens;

#[derive(Clone)]
pub enum Message {
    User(String),
    Assistant(String),
    System(String)
}

impl Message {
    pub fn is_user(&self) -> bool {
        match self {
            Message::User(_) => true,
            _ => false
        }
    }
    
    pub fn is_assistant(&self) -> bool {
        match self {
            Message::Assistant(_) => true,
            _ => false
        }
    }
    
    pub fn is_system(&self) -> bool {
        match self {
            Message::System(_) => true,
            _ => false
        }
    }

    pub fn content(&self) -> &str {
        match self {
            Message::User(content) => content,
            Message::Assistant(content) => content,
            Message::System(content) => content
        }
    }

    pub fn set_content(&mut self, new_content: &str) {
        match self {
            Message::User(content) => *content = new_content.to_string(),
            Message::Assistant(content) => *content = new_content.to_string(),
            Message::System(content) => *content = new_content.to_string()
        }
    }
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

#[async_trait]
pub trait LLMModel : Send + Sync {
    fn get_response(&self, messages: &[Message], max_tokens: Option<u16>) -> Result<String, Box<dyn Error>>;
    fn get_base_embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn Error>>;
    fn get_tokens_remaining(&self, text: &[Message]) -> Result<usize, Box<dyn Error>>;
}

#[async_trait]
pub trait LLMProvider {
    fn get_name(&self) -> String;
    fn create(&self, value: Value) -> Result<Box<dyn LLMModel>, Box<dyn Error>>;
}

pub struct LLM {
    pub prompt: Vec<Message>,
    pub message_history: Vec<Message>,
    pub model: Box<dyn LLMModel>
}

impl LLM {
    pub fn get_tokens_remaining(&self, messages: &[Message]) -> Result<usize, Box<dyn Error>> {
        self.model.get_tokens_remaining(messages)
    }

    pub fn crop_to_tokens(&mut self, token_buffer: usize) -> Result<(), Box<dyn Error>> {
        while token_buffer > self.get_tokens_remaining(&self.get_messages())? {
            self.message_history.remove(0);
        }

        Ok(())
    }

    pub fn get_messages(&self) -> Vec<Message> {
        let mut messages = self.prompt.clone();
        messages.extend(self.message_history.clone());
        messages
    }
}

pub struct ChatGPT {
    pub model: String,
    pub embedding_model: String,
    pub client: Client
}

impl LLMModel for ChatGPT {
    #[tokio::main]
    async fn get_response(&self, messages: &[Message], max_tokens: Option<u16>) -> Result<String, Box<dyn Error>> {
        let mut request = CreateChatCompletionRequest::default();

        request.model = self.model.clone();
        request.messages = messages
            .iter()
            .map(|el| el.clone().into())
            .collect::<Vec<_>>();

        request.max_tokens = max_tokens;
        
        let response: CreateChatCompletionResponse = self.client
            .chat()
            .create(request.clone())
            .await?;

        Ok(response.choices[0].message.content.clone())
    }

    #[tokio::main]
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
}

#[derive(Serialize, Deserialize)]
pub struct ChatGPTConfig {    
    #[serde(rename = "api key")] pub api_key: String,
    pub model: Option<String>,
    #[serde(rename = "embedding model")] pub embedding_model: Option<String>,
}

pub struct ChatGPTProvider;

#[async_trait]
impl LLMProvider for ChatGPTProvider {
    fn get_name(&self) -> String {
        "chatgpt".to_string()
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

pub fn create_model_chatgpt() -> Box<dyn LLMProvider> {
    Box::new(ChatGPTProvider)
}
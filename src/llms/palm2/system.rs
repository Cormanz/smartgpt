use std::error::Error;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tiktoken_rs::async_openai::get_chat_completion_max_tokens;
use tokio::runtime::Runtime;

use crate::{LLMProvider, Message, LLMModel, llms, ApiClient, MessagePrompt, Example, PALMMessage, GCPModel};

pub struct PALM2 {
    pub model: String,
    pub embedding_model: String,
    pub client: ApiClient
}

#[async_trait]
impl LLMModel for PALM2 {
    async fn get_response(&self, messages: &[Message], max_tokens: Option<u16>, temperature: Option<f32>) -> Result<String, Box<dyn Error>> {
        let examples: Vec<Example> = vec![];

        let palm_messages = messages
            .iter()
            .map(|el| el.clone().into()) // This will now convert Message to PALMMessage
            .collect::<Vec<PALMMessage>>();
        
        let mut request_body = MessagePrompt {
            context: "".to_string(),
            examples,
            messages: palm_messages
        };

        let response_message = self.client.generate_message(&self.model, request_body).await?;

        Ok(response_message)
    }
    
    async fn get_base_embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        let embedding_response = self.client.embed_text(&self.embedding_model, text).await?;
        // let embeddings = self.client.embed_text(text).await?;
        
        // .create(CreateEmbeddingRequest {
        //     model: self.embedding_model.clone(),
        //     user: None,
        //     input: EmbeddingInput::String(text.to_string())
        // }).await?;
    
        Ok(vec![embedding_response])
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

        let client = ApiClient::new(config.api_base.unwrap_or("https://generativelanguage.googleapis.com".to_owned()));

        let models = rt.block_on(async {
            client.list_models().await
        })?;
        
        eprintln!("model: {}", models[0].display_name);

        Ok(Box::new(PALM2 {
            model: config.model.unwrap_or("gpt-3.5-turbo".to_string()),
            embedding_model: config.embedding_model.unwrap_or("text-embedding-ada-002".to_string()),
            client
        }))
    }
}

pub fn create_model_palm2() -> Box<dyn LLMProvider> {
    Box::new(PALM2Provider)
}

// impl From<Message> for ChatCompletionRequestMessage {
//     fn from(value: Message) -> Self {
//         match value {
//             Message::User(text) => {
//                 ChatCompletionRequestMessage {
//                     role: Role::User,
//                     content: text,
//                     name: None
//                 }
//             },
//             Message::Assistant(text) => {
//                 ChatCompletionRequestMessage {
//                     role: Role::Assistant,
//                     content: text,
//                     name: None
//                 }
//             },
//             Message::System(text) => {
//                 ChatCompletionRequestMessage {
//                     role: Role::System,
//                     content: text,
//                     name: None
//                 }
//             }
//         }
//     }
// }

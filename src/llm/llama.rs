use std::{error::Error, collections::HashSet, sync::Mutex};

use async_trait::async_trait;
use llama_rs::{InferenceSession, InferenceSessionParameters, Model, InferenceParameters, Vocabulary, TokenBias};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use rand::thread_rng;

use crate::{LLMProvider, LLMModel, Message, ModelLoadError};

pub struct Llama {
    pub path: String
}

fn format_prompt(messages: &[Message]) -> String {
    let mut out = String::new();
    
    for message in messages {
        out.push_str(&format!("{}: {}", 
            match message {
                Message::System(_) => "SYSTEM",
                Message::User(_) => "USER",
                Message::Assistant(_) => "ASSISTANT"
            },
            message.content()
        ))
    }

    out
}

pub struct LlamaInfo {
    pub model: Model,
    pub params: InferenceParameters,
    pub session: InferenceSession
}

#[async_trait]
impl LLMModel for Llama {
    async fn get_response(&self, messages: &[Message], max_tokens: Option<u16>, temperature: Option<f32>) -> Result<String, Box<dyn Error>> {
        println!("oops!");
        let model = Model::load(&self.path, 2000, |_| {})?;
        let params = InferenceParameters::default();
        let session_params = InferenceSessionParameters::default();
        let mut session = model.start_session(session_params);
        
        let mut rng = thread_rng();

        let mut text = String::new();

        session.inference_with_prompt(
            &model, &params, &format_prompt(messages),
            None, &mut rng,
            |token| {
                text.push_str(token);

                Ok::<_, ModelLoadError>(())
            }
        )?;
    
        Ok(text)
    }

    async fn get_base_embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        Ok(vec![])
    }

    fn get_tokens_remaining(&self, text: &[Message]) -> Result<usize, Box<dyn Error>> {
        Ok(2)
    }
}

#[derive(Serialize, Deserialize)]
pub struct LlamaConfig {
    #[serde(rename = "model path")] pub model_path: String
}

pub struct LlamaProvider;

impl LLMProvider for LlamaProvider {
    fn is_enabled(&self) -> bool {
        true
    }

    fn get_name(&self) -> &str {
        "llama"
    }

    fn create(&self, value: Value) -> Result<Box<dyn LLMModel>, Box<dyn Error>> {
        let LlamaConfig { model_path } = serde_json::from_value(value)?;
        println!("haHA!");

        Ok(Box::new(Llama { path: model_path }))
    }
}

pub fn create_model_llama() -> Box<dyn LLMProvider> {
    Box::new(LlamaProvider)
}
use std::{error::Error, collections::HashSet, sync::Mutex, fmt::Display, ops::DerefMut, path::Path};

use async_trait::async_trait;
use llm::{InferenceSession, Model, InferenceParameters, Vocabulary, TokenBias, load_dynamic, ModelParameters, InferenceSessionConfig, InferenceRequest, OutputRequest};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use rand::thread_rng;

#[derive(Debug, Clone)]
pub struct NoLocalModelError(pub String);

impl Display for NoLocalModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NoLocalModelError({:?})", self.0)
    }
}

impl Error for NoLocalModelError {}

use crate::{LLMProvider, LLMModel, Message, ModelLoadError, format_prompt};

pub struct LocalLLM {
    pub model: Box<dyn Model>
}

#[async_trait]
impl LLMModel for LocalLLM {
    async fn get_response(&self, messages: &[Message], max_tokens: Option<u16>, temperature: Option<f32>) -> Result<String, Box<dyn Error>> {
        let session_config = InferenceSessionConfig::default();
        let mut session = self.model.start_session(session_config);
    
        let mut rng = thread_rng();

        let mut text = String::new();
        let prompt = format_prompt(messages);

        session.infer(
            self.model.as_ref(), &mut rng,
            &InferenceRequest {
                prompt: &prompt,
                maximum_token_count: max_tokens.map(|el| el as usize),
                parameters: Some(&InferenceParameters {
                    temperature: temperature.unwrap_or(0.7),
                    ..Default::default()
                }),
                ..Default::default()
            },
            &mut OutputRequest {
                all_logits: Default::default(),
                embeddings: None
            },
            |token| {
                text.push_str(token);

                Ok::<_, ModelLoadError>(())
            }
        )?;
    
        Ok(text.strip_prefix(&prompt).unwrap_or(&text).to_string())
    }

    async fn get_base_embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        Ok(vec![])
    }

    fn get_tokens_remaining(&self, text: &[Message]) -> Result<usize, Box<dyn Error>> {
        Ok(20000)
    }
}

#[derive(Serialize, Deserialize)]
pub struct LocalLLMConfig {
    #[serde(rename = "type")] pub model_type: String,
    #[serde(rename = "model path")] pub model_path: String,
    #[serde(rename = "context tokens")] pub n_context_tokens: usize,
    pub mmap: Option<bool>
}

pub struct LocalLLMProvider;

impl LLMProvider for LocalLLMProvider {
    fn is_enabled(&self) -> bool {
        true
    }

    fn get_name(&self) -> &str {
        "local"
    }

    fn create(&self, value: Value) -> Result<Box<dyn LLMModel>, Box<dyn Error>> {
        let LocalLLMConfig { 
            model_path, 
            model_type,
            n_context_tokens,
            mmap
        } = serde_json::from_value(value)?;
        let model = load_dynamic(
            match model_type.to_ascii_lowercase().replace("-", "").as_str() {
                "llama" => llm::ModelArchitecture::Llama,
                "bloom" => llm::ModelArchitecture::Bloom,
                "gpt2" => llm::ModelArchitecture::Gpt2,
                "gptj" => llm::ModelArchitecture::GptJ,
                "neox" => llm::ModelArchitecture::NeoX,
                _ => {
                    return Err(Box::new(NoLocalModelError(format!("unknown model: {model_type}"))))
                }
            }, 
            &Path::new(&model_path), 
            ModelParameters {
                prefer_mmap: mmap.unwrap_or(true),
                n_context_tokens: n_context_tokens,
                ..Default::default()
            },
            |_| {}
        )?;

        Ok(Box::new(LocalLLM { model }))
    }
}

pub fn create_model_llama() -> Box<dyn LLMProvider> {
    Box::new(LocalLLMProvider)
}
use std::{error::Error, collections::HashSet, sync::Mutex, fmt::Display, ops::DerefMut, path::Path};

use async_trait::async_trait;
use llm::{InferenceSession, Model, InferenceParameters, Vocabulary, TokenBias, load_dynamic, ModelParameters};
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

use crate::{LLMProvider, LLMModel, Message, ModelLoadError};

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
        "llama"
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
        );

        panic!("E");
    }
}

pub fn create_model_llama() -> Box<dyn LLMProvider> {
    Box::new(LocalLLMProvider)
}
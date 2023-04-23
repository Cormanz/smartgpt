use std::{error::Error, fmt::Display};
use async_trait::async_trait;
use serde_json::Value;

mod faiss_memory;
pub use faiss_memory::*;

use crate::{agents::Agent, LLM};

#[derive(Debug, Clone)]
pub struct MemorySystemLoadError(pub String);

impl Display for MemorySystemLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemorySystemLoadError({:?})", self.0)
    }
}

impl Error for MemorySystemLoadError {}

#[derive(Clone)]
pub struct Memory {
    pub content: String,
    pub recall: f64,
    pub embedding: Vec<f32>   
}

#[async_trait]
pub trait MemorySystem : Send + Sync {
    async fn store_memory(&mut self, llm: LLM, memory: &str) -> Result<(), Box<dyn Error>>;

    async fn search_memories(&mut self, llm: LLM, memory: &str) -> Result<Vec<Memory>, Box<dyn Error>>;
}

#[async_trait]
pub trait MemoryProvider {
    fn is_enabled(&self) -> bool;
    fn get_name(&self) -> String;
    fn create(&self, value: Value) -> Result<Box<dyn MemorySystem>, Box<dyn Error>>;
}
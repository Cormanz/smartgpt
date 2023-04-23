use std::{error::Error, fmt::Display, cmp::Reverse, cmp::Ordering::Equal};
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
    pub recall: f32,
    pub recency: f32,
    pub embedding: Vec<f32>   
}

#[derive(Clone)]
pub struct RelevantMemory {
    pub memory: Memory,
    pub relevance: f32
}

#[derive(Clone)]
pub struct ScoredMemory {
    pub memory: Memory,
    pub score: f32
}

#[derive(Clone)]
pub struct Weights {
    pub recall: f32,
    pub recency: f32,
    pub relevance: f32
}

#[async_trait]
pub trait MemorySystem : Send + Sync {
    async fn store_memory(&mut self, llm: LLM, memory: &str) -> Result<(), Box<dyn Error>>;

    async fn get_memory_pool(&mut self, llm: LLM, memory: &str, min_count: usize) -> Result<Vec<RelevantMemory>, Box<dyn Error>>;

    async fn get_memories(
        &mut self, llm: LLM, memory: &str, min_count: usize, 
        weights: Weights, count: usize
    ) -> Result<Vec<Memory>, Box<dyn Error>> {
        let memory_pool = self.get_memory_pool(llm, memory, min_count).await?;
        let mut memories = memory_pool.iter()
            .map(|RelevantMemory { memory, relevance }| {
                ScoredMemory {
                    memory: memory.clone(),
                    score: (
                        weights.recall * memory.recall +
                        weights.recency * memory.recency +
                        weights.relevance * relevance
                    )
                }
            })
            .collect::<Vec<_>>();
        memories.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Equal));
        let memories = memories.iter()
            .map(|el| el.memory.clone())
            .take(count)
            .collect::<Vec<_>>();
        Ok(memories)
    }
}

#[async_trait]
pub trait MemoryProvider {
    fn is_enabled(&self) -> bool;
    fn get_name(&self) -> String;
    fn create(&self, value: Value) -> Result<Box<dyn MemorySystem>, Box<dyn Error>>;
}
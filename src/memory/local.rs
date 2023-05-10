use std::error::Error;

use async_trait::async_trait;
use serde_json::Value;

use crate::{LLM, Memory, MemoryProvider, NoLocalModelError, RelevantMemory, compare_embeddings};

use super::MemorySystem;

pub struct LocalMemorySystem {
    pub memory: Vec<Memory>
}

#[async_trait]
impl MemorySystem for LocalMemorySystem {
    async fn store_memory(&mut self, llm: &LLM, memory: &str) -> Result<(), Box<dyn Error>> {
        let embedding = llm.model.get_base_embed(memory).await?;

        self.memory.push(Memory {
            content: memory.to_string(),
            recency: 1.,
            recall: 1.,
            embedding: embedding.clone()
        });

        Ok(())
    }

    async fn get_memory_pool(&mut self, llm: &LLM, memory: &str, min_count: usize) -> Result<Vec<RelevantMemory>, Box<dyn Error>> {
        let embedding = llm.model.get_base_embed(memory).await?;
    
        let results: Vec<RelevantMemory> = self.memory.iter()
            .map(|memory| RelevantMemory {
                memory: memory.clone(),
                relevance: compare_embeddings(&embedding, &memory.embedding)
            })
            .collect();

        Ok(results)
    }
}

pub struct LocalProvider;

impl MemoryProvider for LocalProvider {
    fn is_enabled(&self) -> bool {
        true
    }

    fn get_name(&self) -> String {
        "local".to_string()
    }

    fn create(&self, _: Value) -> Result<Box<dyn MemorySystem> ,Box<dyn Error> > {
        Ok(Box::new(LocalMemorySystem {
            memory: vec![]
        }))
    }
}

pub fn create_memory_local() -> Box<dyn MemoryProvider> {
    Box::new(LocalProvider)
}
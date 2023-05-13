use std::error::Error;

use async_trait::async_trait;
use serde_json::{Value, to_string};

use crate::{LLM, Memory, MemoryProvider, RelevantMemory, compare_embeddings};

use super::MemorySystem;

use sha2::{Sha256, Digest};

use redis::{Client, RedisResult, AsyncCommands};

pub struct LocalMemorySystem {
    pub memory: Vec<Memory>,
    client: Client
}

#[async_trait]
impl MemorySystem for LocalMemorySystem {
    async fn store_memory(&mut self, llm: &LLM, memory: &str) -> Result<(), Box<dyn Error>> {
        let embedding = llm.model.get_base_embed(memory).await?;
        let mut con = self.client.get_tokio_connection().await?;

        // Generate a unique hash for the memory
        let mut hasher = Sha256::new();
        hasher.update(memory.as_bytes());
        let id = hex::encode(hasher.finalize());

        let memory_struct = Memory {
            id: id.clone(),
            content: memory.to_string(),
            recency: 1.,
            recall: 1.,
            embedding: embedding.clone()
        };

        let memory_json = to_string(&memory_struct)?;

        con.set(id, memory_json).await?;

        self.memory.push(memory_struct);

        Ok(())
    }

    async fn get_memory_pool(&mut self, llm: &LLM, memory: &str, _min_count: usize) -> Result<Vec<RelevantMemory>, Box<dyn Error>> {
        let embedding = llm.model.get_base_embed(memory).await?;
        let mut con = self.client.get_tokio_connection().await?;

        if self.memory.is_empty() {
            let keys: Vec<String> = con.keys("*").await?;

            for key in keys {
                let memory_json: String = con.get(&key).await?;

                let memory_struct: Memory = serde_json::from_str(&memory_json)?;

                self.memory.push(memory_struct);
            }
        }
    
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
        let client = Client::open("redis://127.0.0.1/")?;

        Ok(Box::new(LocalMemorySystem {
            memory: vec![],
            client
        }))
    }
}

pub fn create_memory_local() -> Box<dyn MemoryProvider> {
    Box::new(LocalProvider)
}
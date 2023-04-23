use std::error::Error;

use async_trait::async_trait;
use serde_json::Value;

use crate::{LLM, Memory, MemoryProvider, MemorySystemLoadError, RelevantMemory};

use super::MemorySystem;

#[cfg(feature = "faiss")]
pub struct FaissMemorySystem {
    pub index: faiss::IdMap<faiss::FlatIndex>,
    pub memory: Vec<Memory>
}

#[cfg(feature = "faiss")]
#[async_trait]
impl MemorySystem for FaissMemorySystem {
    async fn store_memory(&mut self, llm: &LLM, memory: &str) -> Result<(), Box<dyn Error>> {
        use faiss::Index;

        let embedding = llm.model.get_base_embed(memory).await?;

        self.memory.push(Memory {
            content: memory.to_string(),
            recency: 1.,
            recall: 1.,
            embedding: embedding.clone()
        });

        self.index.add_with_ids(&embedding, &[faiss::Idx::new((self.memory.len() - 1) as u64)])?;

        Ok(())
    }

    async fn get_memory_pool(&mut self, llm: &LLM, memory: &str, min_count: usize) -> Result<Vec<RelevantMemory>, Box<dyn Error>> {
        use faiss::ConcurrentIndex;

        let embedding = llm.model.get_base_embed(memory).await?;
        let results = self.index.search(&embedding, min_count)?;
    
        let results = results.labels.iter().zip(results.distances)
            .filter(|(idx, _)| idx.get().is_some())
            .map(|(idx, distance)| {
                let id = idx.get().unwrap() as usize;
                RelevantMemory {
                    memory: self.memory[id].clone(),
                    relevance: distance
                }
            })
            .collect::<Vec<_>>();

        Ok(results)
    }
}

pub struct FaissProvider;

#[cfg(feature = "faiss")]
impl MemoryProvider for FaissProvider {
    fn is_enabled(&self) -> bool {
        true
    }

    fn get_name(&self) -> String {
        "faiss".to_string()
    }

    fn create(&self, _: Value) -> Result<Box<dyn MemorySystem> ,Box<dyn Error> > {
        let index = faiss::FlatIndex::new_l2(1000).unwrap();
        let index = faiss::IdMap::new(index).unwrap();

        Ok(Box::new(FaissMemorySystem {
            index,
            memory: vec![]
        }))
    }
}

#[cfg(not(feature = "faiss"))]
impl MemoryProvider for FaissProvider {
    fn is_enabled(&self) -> bool {
        false
    }

    fn get_name(&self) -> String {
        "faiss".to_string()
    }

    fn create(&self, _: Value) -> Result<Box<dyn MemorySystem> ,Box<dyn Error> > {
        Err(Box::new(MemorySystemLoadError(
            "Cannot load Faiss memory system: Please enable the `faiss` feature.".to_string()
        )))
    }
}

pub fn create_memory_faiss() -> Box<dyn MemoryProvider> {
    Box::new(FaissProvider)
}
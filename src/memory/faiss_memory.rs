use std::error::Error;

use async_trait::async_trait;
use faiss::{FlatIndex, IdMap, Idx, Index};
use serde_json::Value;

use crate::{LLM, Memory, MemoryProvider, MemorySystemLoadError};

use super::MemorySystem;

#[cfg(feature = "faiss")]
pub struct FaissMemorySystem {
    pub index: IdMap<FlatIndex>,
    pub memory: Vec<Memory>
}

#[cfg(feature = "faiss")]
#[async_trait]
impl MemorySystem for FaissMemorySystem {
    async fn store_memory(&mut self, llm: LLM, memory: &str) -> Result<(), Box<dyn Error>> {
        let embedding = llm.model.get_base_embed(memory).await?;

        self.memory.push(Memory {
            content: memory.to_string(),
            recall: 1.,
            embedding: embedding.clone()
        });

        self.index.add_with_ids(&embedding, &[Idx::new((self.memory.len() - 1) as u64)])?;

        Ok(())
    }

    async fn search_memories(&mut self, llm: LLM, memory: &str) -> Result<Vec<Memory>, Box<dyn Error>> {
        let embedding = llm.model.get_base_embed(memory).await?;
        let results = self.index.search(&embedding, 10)?;
    
        let results = results.labels.iter()
            .filter(|el| el.get().is_some())
            .map(|&el| {
                let id = el.get().unwrap() as usize;
                self.memory[id].clone()
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
        let index = FlatIndex::new_l2(1000).unwrap();
        let index = IdMap::new(index).unwrap();

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
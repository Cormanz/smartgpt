use std::error::Error;

use async_trait::async_trait;
use faiss::{FlatIndex, IdMap};

use super::MemorySystem;

pub struct Memory {
    pub content: String,
    
}

pub struct FaissMemory {
    pub index: IdMap<FlatIndex>,
    pub memory: Vec<(String, Vec<f32>)>
}

/*#[async_trait]
impl MemorySystem for FaissMemory {
    fn is_enabled(&self) -> bool {
        true
    }
    
    async fn store_memory(&self, memory: &str) -> Result<(), Box<dyn Error>> {

    }

    async fn search_memories(&self, memory_context: &str) -> Result<Vec<String>, Box<dyn Error>> {
        
    }
}*/
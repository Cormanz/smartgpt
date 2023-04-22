use std::error::Error;
use async_trait::async_trait;
use serde_json::Value;

mod faiss_memory;
pub use faiss_memory::*;

#[async_trait]
trait MemorySystem {
    fn is_enabled(&self) -> bool;

    async fn store_memory(&self, memory: &str) -> Result<(), Box<dyn Error>>;

    async fn search_memories(&self, memory_context: &str) -> Result<Vec<String>, Box<dyn Error>>;
}

#[async_trait]
pub trait MemoryProvider {
    fn get_name(&self) -> String;
    fn create(&self, value: Value) -> Result<Box<dyn MemorySystem>, Box<dyn Error>>;
}
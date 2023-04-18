use std::error::Error;

use async_trait::async_trait;

use crate::agents::Agent;

#[async_trait]
trait MemorySystem {
    fn is_enabled(&self) -> bool;

    async fn store_memory(&self, agent: Agent, memory: &str) -> Result<(), Box<dyn Error>>;

    async fn search_memories(&self, agent: Agent, memory_context: &str) -> Result<Vec<String>, Box<dyn Error>>;
}
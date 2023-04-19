mod encode;

use std::{collections::{HashMap, hash_map::DefaultHasher}, fmt::Display, error::Error, fs, hash::{Hash, Hasher}, any::Any};

use async_trait::async_trait;
use colored::Colorize;
pub use encode::*;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::EmptyCycle;

use crate::{Plugin, CommandContext, PluginCycle, PluginData, PluginDataNoInvoke, invoke};

/*
    pub memory_index: IdMap<FlatIndex>,
    pub memory: Vec<(String, Vec<f32>)>,


*/

#[cfg(feature = "faiss-rs")]
mod memory {
    pub struct MemoryData {
        pub index: IdMap<FlatIndex>,
        pub memory: Vec<(String, Vec<f32>)>
    }
    
    #[derive(Serialize, Deserialize)]
    pub struct MemoryAddInfo {
        pub memory: String,
        pub embedding: Vec<f32>
    }
    
    #[derive(Serialize, Deserialize)]
    pub struct MemorySearchInfo {
        pub embedding: Vec<f32>
    }
    
    #[async_trait]
    impl PluginData for MemoryData {
        async fn apply(&mut self, name: &str, value: Value) -> Result<Value, Box<dyn Error>> {
            match name {
                "add memory" => {
                    let MemoryAddInfo { memory, embedding } = serde_json::from_value(value)?;
                    self.memory.push((memory.to_string(), embedding.clone()));
                    self.index.add_with_ids(&embedding, &[Idx::new((self.memory.len() - 1) as u64)])?;
    
                    Ok(true.into())
                }
                "search" => {
                    let MemorySearchInfo { embedding } = serde_json::from_value(value)?;
                    let results = self.index.search(&embedding, 10)?;
    
                    let results = results.labels.iter()
                        .filter(|el| el.get().is_some())
                        .map(|&el| {
                            let id = el.get().unwrap() as usize;
                            self.memory[id].0.clone()
                        })
                        .collect::<Vec<_>>();
                    
                    Ok(results.into())
                }
                _ => {
                    Err(Box::new(PluginDataNoInvoke("Memory".to_string(), name.to_string())))
                }
            }
        }
    }
    
    pub struct MemoryCycle;
    
    fn find_closest_vector(target: Vec<f32>, vectors: &Vec<(String, Vec<f32>)>) -> Option<&(String, Vec<f32>)> {
        let mut closest_distance = f32::MAX;
        let mut closest_vector: Option<&(String, Vec<f32>)> = None;
    
        for vector in vectors {
            let distance = euclidean_distance(&target, &vector.1);
            if distance < closest_distance {
                closest_distance = distance;
                closest_vector = Some(vector);
            }
        }
    
        closest_vector
    }
    
    fn euclidean_distance(vec1: &Vec<f32>, vec2: &Vec<f32>) -> f32 {
        vec1.iter().zip(vec2.iter()).map(|(a, b)| (a - b).powi(2)).sum::<f32>().sqrt()
    }
    
    pub async fn add_memory(memory: &str, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let memory_info = context.plugin_data.get_data("Memory")?;
        
        let memory = memory.trim();
        if memory.len() < 45 || memory.contains("no previous command") {
            return Ok(());
        }
    
        let embedding = get_embed(&context.llm, memory).await?;
        invoke::<bool>(memory_info, "add memory", MemoryAddInfo {
            memory: memory.to_string(),
            embedding
        }).await?;
    
        Ok(())
    }
    
    pub async fn search_memories(memory: &str, context: &mut CommandContext) -> Result<Vec<String>, Box<dyn Error>> {
        let memory_info = context.plugin_data.get_data("Memory")?;
        
        let embedding = get_embed(&context.llm, memory).await?;
        let out: Vec<String> = invoke(memory_info, "search", MemorySearchInfo { 
            embedding 
        }).await?;
        Ok(
            out
        )
    }
    
    #[async_trait]
    impl PluginCycle for MemoryCycle {
        async fn create_context(&self, context: &mut CommandContext, previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
            match previous_prompt {
                None => {
                    Ok(None)
                }
                Some(prompt) => {
                    let memories: Vec<String> = search_memories(prompt, context).await?;
                    if memories.len() == 0 {
                        return Ok(None);
                    }
    
                    let sliced_memories = memories.clone().iter()
                        .map(|el| el.chars().take(50).map(|el| el.to_string()).collect::<Vec<_>>().join(""))
                        .collect::<Vec<_>>();
    
                    println!("{}:", "Found Memories".yellow());
                    for memory in sliced_memories {
                        println!("- {memory}...")
                    }
    
                    let mut memory_prompt = "Memories:".to_string();
                    for memory in memories {
                        memory_prompt.push_str("\n - ");
                        memory_prompt.push_str(&memory);
                    }
                    
                    Ok(Some(memory_prompt))
                }
            }
        }
    
        async fn apply_removed_response(&self, context: &mut CommandContext, response: &LLMResponse, cmd_output: &str, previous_response: bool) -> Result<(), Box<dyn Error>> {
            if !previous_response {
                return Ok(());
            }
    
            let memory = response.summary.clone()
                .iter()
                .flat_map(|el| {
                    let mut takeaways = vec![ el.takeaway.clone() ];
                    takeaways.extend(el.points.clone());
                    takeaways
                })
                .collect::<Vec<_>>()
                .join(" ");
            add_memory(&memory, context).await?;
    
            Ok(())
        }
    
        async fn create_data(&self, value: Value) -> Option<Box<dyn PluginData>> {
            let index = FlatIndex::new_l2(1000).unwrap();
            let index = IdMap::new(index).unwrap();
    
            Some(Box::new(MemoryData {
                index: index,
                memory: vec![]
            }))
        }
    }
}

#[cfg(feature = "faiss-rs")]
pub fn create_memory() -> Plugin {
    Plugin {
        name: "Memory".to_string(),
        dependencies: vec![],
        cycle: Box::new(MemoryCycle),
        commands: vec![]
    }
}

#[cfg(not(feature = "faiss-rs"))]
pub fn create_memory() -> Plugin {
    Plugin {
        name: "Memory".to_string(),
        dependencies: vec![],
        cycle: Box::new(EmptyCycle),
        commands: vec![]
    }
}
use std::cmp::Ordering;
use std::{sync::Arc};
use std::error::Error;
use redis::{Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::runtime::Runtime;

use crate::{LLM, Memory, MemoryProvider, RelevantMemory};

use tokio::{sync::Mutex};

use super::MemorySystem;

use async_trait::async_trait;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RedisPayload {
    content: String,
    recall: f32,
    recency: f32,
}

pub struct RedisMemorySystem {
    client: redis::Client,
    latest_point_id: Arc<Mutex<u64>>,
    index_name: String,
}

#[async_trait]
impl MemorySystem for RedisMemorySystem {
    async fn store_memory(&mut self, llm: &LLM, memory: &str) -> Result<(), Box<dyn Error>> {
        let embedding = llm.model.get_base_embed(memory).await?;
        let mut con = self.client.get_tokio_connection().await?;

        let memory_struct = RedisPayload {
            content: memory.to_string(),
            recency: 1.,
            recall: 1.,
        };

        let mut latest_point_id = self.latest_point_id.lock().await;
        *latest_point_id += 1;
        let point_id = latest_point_id.to_string();

        let memory_json = serde_json::to_value(&memory_struct)?;
        let embedding_json = serde_json::to_value(&embedding)?;

        redis::cmd("JSON.SET")
            .arg(&point_id)
            .arg("$")
            .arg(format!(r#"{{"memory": {}, "embedding": {}}}"#, memory_json, embedding_json))
            .query_async(&mut con)
            .await?;

        Ok(())
    }

    async fn get_memory_pool(&mut self, llm: &LLM, memory: &str, min_count: usize) -> Result<Vec<RelevantMemory>, Box<dyn Error>> {
        let embedding = llm.model.get_base_embed(memory).await?;
        let mut con = self.client.get_tokio_connection().await?;

        let query_blob: Vec<u8> = embedding
            .iter()
            .flat_map(|&value| value.to_le_bytes().to_vec())
            .collect();

        let result: redis::Value = redis::cmd("FT.SEARCH")
            .arg(&self.index_name) // Replace with the actual index name
            .arg("DIALECT")
            .arg(2)
            .arg(format!("($query)=>[KNN {} @v $B]", min_count))
            .arg("PARAMS")
            .arg("query")
            .arg(&query_blob)
            .query_async(&mut con)
            .await?;

        let result_pairs: Vec<(String, f32)> = match result {
            redis::Value::Bulk(items) => {
                items
                    .chunks_exact(2)
                    .filter_map(|chunk| match (chunk.get(0), chunk.get(1)) {
                        (Some(redis::Value::Data(key)), Some(redis::Value::Data(value))) => {
                            let score: f32 = String::from_utf8_lossy(value)
                                .parse()
                                .unwrap_or_default();
                            Some((String::from_utf8_lossy(key).into_owned(), score))
                        }
                        _ => None,
                    })
                    .collect()
            }
            _ => vec![],
        };

        let mut relevant_memories = vec![];
        for (key, similarity) in result_pairs {
            let json_data: String = redis::cmd("JSON.GET")
                .arg(&key)
                .query_async(&mut con)
                .await?;
            let data: serde_json::Value = serde_json::from_value(serde_json::Value::String(json_data))?;
            let memory_data: RedisPayload = serde_json::from_value(data["memory"].clone())?;
            let memory_embedding: Vec<f32> = serde_json::from_value(data["embedding"].clone())?;

            relevant_memories.push(RelevantMemory {
                memory: Memory {
                    content: memory_data.content,
                    recall: memory_data.recall,
                    recency: memory_data.recency,
                    embedding: memory_embedding,
                },
                relevance: similarity,
            });
        }

        // Sort the relevant memories by relevance and return the top min_count memories
        relevant_memories.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(Ordering::Equal));
        Ok(relevant_memories.into_iter().take(min_count).collect())
    }
}

pub struct RedisProvider;

impl MemoryProvider for RedisProvider {
    fn is_enabled(&self) -> bool {
        true
    }

    fn get_name(&self) -> String {
        "redis".to_string()
    }

    fn create(&self, _: Value) -> Result<Box<dyn MemorySystem> ,Box<dyn Error> > {
        let client = Client::open("redis://127.0.0.1/")?;

        let rt = Runtime::new().expect("Failed to create Tokio runtime");

        let index_name = "smartgpt_agent_memory";

        rt.block_on(async {
            let mut con = client.get_tokio_connection().await?;
            match create_index_if_not_exists(&mut con, index_name, "$.embedding", 1536).await {
                Ok(()) => {Ok(())}
                Err(err) => {
                    eprintln!("Failed to create vector index: {}", err);
                    return Err(Box::new(err));
                }
            }
        })?;

        Ok(Box::new(RedisMemorySystem {
            client,
            latest_point_id: Arc::new(Mutex::new(0)),
            index_name: index_name.to_string(), // This could be configured differently depending on your needs
        }))
    }
}

async fn create_index_if_not_exists(con: &mut redis::aio::Connection, index_name: &str, field_path: &str, dimension: usize) -> redis::RedisResult<()> {
    let collection_exists: bool = redis::cmd("FT.INFO")
        .arg(index_name)
        .query_async(con)
        .await
        .map(|_: redis::Value| true)
        .or_else(|err: redis::RedisError| {
            if err.kind() == redis::ErrorKind::TypeError {
                Ok(false)
            } else {
                Err(err)
            }
        })?;

    if !collection_exists {
        let _: () = redis::cmd("FT.CREATE")
            .arg(index_name)
            .arg("ON")
            .arg("JSON")
            .arg("SCHEMA")
            .arg(field_path)
            .arg("as")
            .arg("vector")
            .arg("VECTOR")
            .arg("FLAT")
            .arg(6)
            .arg("TYPE")
            .arg("FLOAT32")
            .arg("DIM")
            .arg(dimension)
            .arg("DISTANCE_METRIC")
            .arg("L2")
            .query_async(con)
            .await?;
    }

    Ok(())
}

async fn run_knn_search(
    client: &mut redis::aio::Connection,
    query_vector: &[f32],
    k: usize,
    index_name: &str
) -> redis::RedisResult<Vec<(String, f32)>> {
    let query_blob: Vec<u8> = query_vector
        .iter()
        .flat_map(|&value| value.to_le_bytes().to_vec())
        .collect();

    let result: redis::Value = redis::cmd("FT.SEARCH")
        .arg(index_name.to_string()) // Replace with your actual index name
        .arg("DIALECT")
        .arg(2)
        .arg("(@title:Matrix @year:[2020 2022])=>[KNN 10 @v $B]")
        .arg("PARAMS")
        .arg("@v")
        .arg(&query_blob)
        .arg("AS")
        .arg("@dist")
        .arg("LIMIT")
        .arg(0)
        .arg(k)
        .arg("SORTBY")
        .arg("@dist")
        .query_async(client)
        .await?;

    let result_pairs: Vec<(String, f32)> = match result {
        redis::Value::Bulk(items) => {
            items
                .chunks_exact(2)
                .filter_map(|chunk| match (chunk.get(0), chunk.get(1)) {
                    (Some(redis::Value::Data(key)), Some(redis::Value::Data(value))) => {
                        let score: f32 = String::from_utf8_lossy(value)
                            .parse()
                            .unwrap_or_default();
                        Some((String::from_utf8_lossy(key).into_owned(), score))
                    }
                    _ => None,
                })
                .collect()
        }
        _ => vec![],
    };

    Ok(result_pairs)
}

pub fn create_memory_redis() -> Box<dyn MemoryProvider> {
    Box::new(RedisProvider)
}
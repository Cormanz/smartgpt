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

use std::borrow::Borrow;

#[derive(Serialize, Deserialize)]
pub struct EmbeddedMemory {
    memory: RedisPayload,
    embedding: Vec<f32>
}

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

        let embedded_memory = EmbeddedMemory {
            memory: RedisPayload {
                content: memory.to_string(),
                recency: 1.,
                recall: 1.,
            },
            embedding: embedding
        };

        let mut latest_point_id = self.latest_point_id.lock().await;
        *latest_point_id += 1;
        let point_id = latest_point_id.to_string();

        set_json_record(&mut con, &point_id, &embedded_memory).await?;

        Ok(())
    }

    async fn get_memory_pool(&mut self, llm: &LLM, memory: &str, min_count: usize) -> Result<Vec<RelevantMemory>, Box<dyn Error>> {
        let embedding = llm.model.get_base_embed(memory).await?;
        let mut con = self.client.get_tokio_connection().await?;

        let query_blob: Vec<u8> = embedding
            .iter()
            .flat_map(|&value| value.to_le_bytes().to_vec())
            .collect();

        let result: redis::Value = search_vector_field(&mut con, &self.index_name, &query_blob, min_count).await?;

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

            let data: EmbeddedMemory = serde_json::from_value(serde_json::Value::String(json_data))?;

            relevant_memories.push(RelevantMemory {
                memory: Memory {
                    content: data.memory.content,
                    recall: data.memory.recall,
                    recency: data.memory.recency,
                    embedding: data.embedding,
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

#[derive(Serialize, Deserialize)]
pub struct RedisMemoryConfig {
    pub index: String
}

impl MemoryProvider for RedisProvider {
    fn is_enabled(&self) -> bool {
        true
    }

    fn get_name(&self) -> String {
        "redis".to_string()
    }

    fn create(&self, config: serde_json::Value) -> Result<Box<dyn MemorySystem> ,Box<dyn Error> > {
        let client = Client::open("redis://127.0.0.1/")?;

        let rt = Runtime::new().expect("Failed to create Tokio runtime");

        let qdrant_config: RedisMemoryConfig = serde_json::from_value(config)?;
        let index_name = qdrant_config.index;

        rt.block_on(async {
            let mut con = client.get_tokio_connection().await?;
            match create_index_if_not_exists(&mut con, &index_name, "$.embedding", 1536).await {
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

async fn execute_redis_command<T: redis::FromRedisValue, S: Borrow<str>>(
    con: &mut redis::aio::Connection,
    command: &str,
    args: &[S],
) -> redis::RedisResult<T> {
    let mut cmd = redis::cmd(command);
    for arg in args {
        cmd.arg(arg.borrow());
    }
    cmd.query_async(con).await
}

async fn create_index_if_not_exists(con: &mut redis::aio::Connection, index_name: &str, field_path: &str, dimension: usize) -> redis::RedisResult<()> {
    let index_exists: bool = redis::cmd("FT.INFO")
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

    if !index_exists {
        let _ = execute_redis_command::<redis::Value, _>(
            con,
            "FT.CREATE",
            &[
                index_name,
                "ON",
                "JSON",
                "SCHEMA",
                field_path,
                "as",
                "vector",
                "VECTOR",
                "FLAT",
                "6",
                "TYPE",
                "FLOAT32",
                "DIM",
                &dimension.to_string(),
                "DISTANCE_METRIC",
                "L2"
            ],
        ).await;
    }

    Ok(())
}

async fn search_vector_field(
    con: &mut redis::aio::Connection,
    index_name: &str,
    query_blob: &[u8],
    k: usize,
) -> redis::RedisResult<redis::Value> {
    // check if k can be formatted as a string to prevent panic
    let k_str = k.to_string();
    if k_str.is_empty() {
        return Err(redis::RedisError::from((redis::ErrorKind::TypeError, "Invalid k value")));
    }

    let query_blob_str = base64::encode(query_blob);

    execute_redis_command::<redis::Value, _>(
        con,
        "FT.SEARCH",
        &[
            index_name,
            &format!("*=>[KNN {} @vec $BLOB]", k_str),
            "PARAMS",
            "2",
            "BLOB",
            &query_blob_str,
            "DIALECT",
            "2",
        ],
    ).await
}

async fn set_json_record(
    con: &mut redis::aio::Connection,
    point_id: &str,
    embedded_memory: &EmbeddedMemory,
) -> redis::RedisResult<()> {
    execute_redis_command::<(), &str>(
        con,
        "JSON.SET",
        &[
            point_id,
            "$",
            &serde_json::to_value(&embedded_memory)?.to_string(),
        ],
    ).await
}

pub fn create_memory_redis() -> Box<dyn MemoryProvider> {
    Box::new(RedisProvider)
}
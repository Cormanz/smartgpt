use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::vec;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{LLM, Memory, MemoryProvider, RelevantMemory, compare_embeddings};

use qdrant_client::prelude::*;
use qdrant_client::qdrant::value::Kind;
use qdrant_client::qdrant::vectors::VectorsOptions;
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::with_payload_selector::SelectorOptions;
use qdrant_client::qdrant::{CreateCollection, SearchPoints, VectorParams, VectorsConfig, PointId, Vectors, Vector, WithPayloadSelector, with_payload_selector, OptimizersConfigDiff, WalConfigDiff, HnswConfigDiff, QuantizationConfig, quantization_config, ScalarQuantization, RecommendPoints};
use tokio::runtime::Runtime;

use super::MemorySystem;

use async_trait::async_trait;
use serde_json::to_string;
use sha2::{Sha256, Digest};


pub struct QdrantMemorySystem {
    client: QdrantClient,
    latest_point_id: Arc<Mutex<Option<u64>>>,
    collection_name: String
}

#[async_trait]
impl MemorySystem for QdrantMemorySystem {
    async fn store_memory(&mut self, llm: &LLM, memory: &str) -> Result<(), Box<dyn Error>> {
        let embedding = llm.model.get_base_embed(memory).await?;

        let memory_struct = Memory {
            content: memory.to_string(),
            recency: 1.0,
            recall: 1.0,
            embedding: embedding.clone(),
        };

        let mut memory_map: HashMap<String, Value> = HashMap::new();
        memory_map.insert("content".to_string(), Value {
            kind: Some(Kind::StringValue(memory_struct.content.clone())),
        });
        memory_map.insert("recency".to_string(), Value {
            kind: Some(Kind::DoubleValue(memory_struct.recency as f64)),
        });
        memory_map.insert("recall".to_string(), Value {
            kind: Some(Kind::DoubleValue(memory_struct.recall as f64)),
        });

        let mut latest_point_id = self.latest_point_id.lock().await;
        let point_id_val = match *latest_point_id {
            Some(id) => id + 1,
            None => 1,
        };
        *latest_point_id = Some(point_id_val);

        let point_id = PointId {
            point_id_options: Some(point_id::PointIdOptions::Num(point_id_val)),
        };
        
        let vectors = Vectors {
            vectors_options: Some(VectorsOptions::Vector(Vector {
                data: embedding.clone(),
            })),
        };

        self.client
        .upsert_points(
            self.collection_name.to_string(),
            vec![PointStruct {
                id: Some(point_id),
                payload: memory_map,
                vectors: Some(vectors)
            }],
            None, // Optional ordering parameter can be set to None
        )
        .await?;

        Ok(())
    }

    async fn get_memory_pool(
        &mut self,
        llm: &LLM,
        memory: &str,
        _min_count: usize,
    ) -> Result<Vec<RelevantMemory>, Box<dyn Error>> {
        let embedding = llm.model.get_base_embed(memory).await?;
        let latest_point_id_option = self.latest_point_id.lock().await.clone();
        let latest_point_id = latest_point_id_option.unwrap_or(0);

        let mut points: Vec<PointId> = vec![];
        let mut search_result = vec![];
        if latest_point_id > 0 {
            points.push(PointId {
                point_id_options: Some(point_id::PointIdOptions::Num(latest_point_id)),
            });

            let recommend_request = RecommendPoints {
                collection_name: self.collection_name.to_string(),
                limit: _min_count as u64,
                with_payload: Some(WithPayloadSelector {
                    selector_options: Some(with_payload_selector::SelectorOptions::Enable(true)),
                }),
                params: None,
                score_threshold: None,
                offset: None,
                with_vectors: None,
                read_consistency: None,
                positive: points,
                negative: vec![],
                filter: None,
                using: None,
                lookup_from: None
            };

            let recommend_response = self.client.recommend(&recommend_request).await?;
            search_result = recommend_response.result;
        } else {
            let search_request = SearchPoints {
                collection_name: self.collection_name.to_string(),
                vector: embedding.clone(),
                filter: None,
                limit: _min_count as u64,
                with_payload: Some(WithPayloadSelector {
                    selector_options: Some(with_payload_selector::SelectorOptions::Enable(true)),
                }),
                params: None,
                score_threshold: None,
                offset: None,
                vector_name: None,
                with_vectors: None,
                read_consistency: None
            };

            let search_response = self.client.search_points(&search_request).await?;
            search_result = search_response.result;
        }
        
        let relevant_memories: Vec<RelevantMemory> = search_result
            .iter()
            .map(|point| {
                let content = match point.payload.get("content") {
                    Some(value) => match &value.kind {
                        Some(Kind::StringValue(string_value)) => string_value.clone(),
                        _ => String::new(),
                    },
                    None => String::new(),
                };
        
                let recall = match point.payload.get("recall") {
                    Some(value) => match &value.kind {
                        Some(Kind::DoubleValue(double_value)) => *double_value as f32,
                        _ => 0.0,
                    },
                    None => 0.0,
                };
        
                let recency = match point.payload.get("recency") {
                    Some(value) => match &value.kind {
                        Some(Kind::DoubleValue(double_value)) => *double_value as f32,
                        _ => 0.0,
                    },
                    None => 0.0,
                };

                let point_embedding = match &point.vectors {
                    Some(vectors) => match &vectors.vectors_options {
                        Some(VectorsOptions::Vector(vector)) => vector.data.clone(),
                        _ => Vec::new(),
                    },
                    None => Vec::new(),
                };

                let memory = Memory {
                    content,
                    recall,
                    recency,
                    embedding: point_embedding.clone()
                };
                let relevance = point.score;

                RelevantMemory {
                    memory,
                    relevance,
                }
            })
            .collect();
        Ok(relevant_memories)

    }
}

pub struct QdrantProvider;

impl MemoryProvider for QdrantProvider {
    fn is_enabled(&self) -> bool {
        true
    }

    fn get_name(&self) -> String {
        "qdrant".to_string()
    }

    fn create(&self, _: serde_json::Value) -> Result<Box<dyn MemorySystem>, Box<dyn Error>> {
        let qdrant_host = std::env::var("QDRANT_HOST")
            .unwrap_or_else(|_| String::from("http://localhost:6334"));

        let config = QdrantClientConfig::from_url(&qdrant_host);

        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        let client = rt.block_on(async {
            QdrantClient::new(Some(config)).await
        })?;

        let collection_name = "qdrant_memory";

        let collection_exists = rt.block_on(async {
            client.has_collection(collection_name.to_string()).await
        })?;

        if !collection_exists {
            let collection_creation_result = rt.block_on(async {
                client.create_collection(
                    &create_initial_collection(collection_name.to_string())
                ).await
            });
            match collection_creation_result {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Failed to create collection: {}", e);
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e))));
                }
            };
    
            let collection_exists = rt.block_on(async {
                client.has_collection(collection_name.to_string()).await
            })?;
            if !collection_exists {
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to create collection")));
            }
        }

        Ok(Box::new(QdrantMemorySystem { 
            client,
            latest_point_id: Arc::new(Mutex::new(Some(0))),
            collection_name: collection_name.to_string()
        }))
    }
}

fn create_initial_collection(name: String) -> CreateCollection {
    let mut create_collection = CreateCollection::default();

    // Set the values of the fields
    create_collection.collection_name = name.to_string();
    create_collection.vectors_config = Some(VectorsConfig {
        config: Some(Config::Params(VectorParams {
            size: 1536,
            distance: 3,
            ..Default::default()
            // ... populate VectorParams fields here
        })),
        ..Default::default()
    });

    return create_collection;
}

pub fn create_memory_qdrant() -> Box<dyn MemoryProvider> {
    Box::new(QdrantProvider)
}
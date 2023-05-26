use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::vec;
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;

use crate::{LLM, Memory, MemoryProvider, RelevantMemory, MemorySystem, init_qdrant_client, create_collection_if_not_exists, convert_to_relevant_memory};

use qdrant_client::prelude::*;
use qdrant_client::qdrant::vectors::VectorsOptions;
use qdrant_client::qdrant::{SearchPoints, PointId, Vectors, Vector, WithPayloadSelector, with_payload_selector, RecommendPoints};
use tokio::runtime::Runtime;

use async_trait::async_trait;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QdrantPayload {
    pub content: String,
    pub recall: f32,
    pub recency: f32
}

impl QdrantPayload {
    pub fn new(content: String, recall: f32, recency: f32) -> Self {
        Self { content, recall, recency }
    }

    pub fn to_memory_map(&self) -> Result<HashMap<String, Value>, serde_json::Error> {
        let serialized_struct = serde_json::to_string(self)?;

        let memory_map: HashMap<String, Value> = serde_json::from_str(&serialized_struct)?;

        Ok(memory_map)
    }
}

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

        let payload = QdrantPayload::new(
            memory_struct.content.clone(),
            memory_struct.recency,
            memory_struct.recall
        );

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
                payload: payload.to_memory_map()?,
                vectors: Some(vectors)
            }],
            None,
        )
        .await?;

        Ok(())
    }

    async fn get_memory_pool(
        &mut self,
        llm: &LLM,
        memory: &str,
        min_count: usize,
    ) -> Result<Vec<RelevantMemory>, Box<dyn Error>> {
        let embedding = llm.model.get_base_embed(memory).await?;
        let latest_point_id_option = self.latest_point_id.lock().await.clone();
        let latest_point_id = latest_point_id_option.unwrap_or(0);

        let mut points: Vec<PointId> = vec![];
        let search_result;
        if latest_point_id > 0 {
            points.push(PointId {
                point_id_options: Some(point_id::PointIdOptions::Num(latest_point_id)),
            });

            let recommend_request = RecommendPoints {
                collection_name: self.collection_name.to_string(),
                limit: min_count as u64,
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
                limit: min_count as u64,
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

        let relevant_memories_result: Result<Vec<_>, _> = search_result
            .iter()
            .map(|point| convert_to_relevant_memory(point))
            .collect();

        match relevant_memories_result {
            Ok(relevant_memories) => Ok(relevant_memories),
            Err(e) => Err(e),
        }
    }

    async fn decay_recency(&mut self, _decay_factor: f32) -> Result<(), Box<dyn Error>> {
        // TODO

        Ok(())
    }
}

pub struct QdrantProvider;

#[derive(Serialize, Deserialize)]
pub struct QdrantMemoryConfig {
    pub collection: String
}

impl MemoryProvider for QdrantProvider {
    fn is_enabled(&self) -> bool {
        true
    }

    fn get_name(&self) -> String {
        "qdrant".to_string()
    }

    fn create(&self, config: serde_json::Value) -> Result<Box<dyn MemorySystem>, Box<dyn Error>> {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        let client = rt.block_on(async {
            init_qdrant_client().await
        })?;

        let qdrant_config: QdrantMemoryConfig = serde_json::from_value(config)?;
        let collection_name = qdrant_config.collection;

        rt.block_on(async {
            create_collection_if_not_exists(&client, &collection_name).await
        })?;

        Ok(Box::new(QdrantMemorySystem { 
            client,
            latest_point_id: Arc::new(Mutex::new(Some(0))),
            collection_name: collection_name.to_string()
        }))
    }
}

pub fn create_memory_qdrant() -> Box<dyn MemoryProvider> {
    Box::new(QdrantProvider)
}
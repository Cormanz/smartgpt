use std::error::Error;

use crate::{Memory, RelevantMemory, QdrantPayload};

use qdrant_client::prelude::*;
use qdrant_client::qdrant::vectors::VectorsOptions;
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::{CreateCollection, VectorParams, VectorsConfig, ScoredPoint};

pub async fn init_qdrant_client() -> Result<QdrantClient, Box<dyn Error>> {
    let qdrant_host = std::env::var("QDRANT_HOST")
        .unwrap_or_else(|_| String::from("http://localhost:6334"));

    let config = QdrantClientConfig::from_url(&qdrant_host);

    let client = QdrantClient::new(Some(config))?;

    Ok(client)
}

pub async fn create_collection_if_not_exists(client: &QdrantClient, collection_name: &str) -> Result<(), Box<dyn Error>> {
    let collection_exists = client.has_collection(collection_name.to_string()).await?;

    if !collection_exists {
        let collection_creation_result = client.create_collection(
            &create_initial_collection(collection_name.to_string())
        ).await;
        match collection_creation_result {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Failed to create collection: {}", e);
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e))));
            }
        };

        let collection_exists = client.has_collection(collection_name.to_string()).await?;
        if !collection_exists {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to create collection")));
        }
    }

    Ok(())
}

pub fn create_initial_collection(name: String) -> CreateCollection {
    let mut create_collection = CreateCollection::default();

    create_collection.collection_name = name.to_string();
    create_collection.vectors_config = Some(VectorsConfig {
        config: Some(Config::Params(VectorParams {
            size: 1536,
            distance: 3,
            ..Default::default()
        })),
        ..Default::default()
    });

    return create_collection;
}

pub fn convert_to_relevant_memory(point: &ScoredPoint) -> Result<RelevantMemory, Box<dyn Error>> {
    let json_string = serde_json::to_value(&point.payload).unwrap_or("".into());

    let payload: QdrantPayload = match serde_json::from_value(json_string) {
        Ok(p) => p,
        Err(e) => {
            return Err(Box::new(e));
        }
    };

    let point_embedding = match &point.vectors {
        Some(vectors) => match &vectors.vectors_options {
            Some(VectorsOptions::Vector(vector)) => vector.data.clone(),
            _ => Vec::new(),
        },
        None => Vec::new(),
    };

    let memory = Memory {
        content: payload.content,
        recall: payload.recall,
        recency: payload.recency,
        embedding: point_embedding.clone()
    };
    let relevance = point.score;

    Ok(RelevantMemory {
        memory,
        relevance,
    })
}


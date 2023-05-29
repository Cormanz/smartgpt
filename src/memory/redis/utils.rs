use base64::{engine::general_purpose::STANDARD, Engine};
use redis::{RedisError, RedisResult};
use crate::{EmbeddedMemory};

use std::{borrow::Borrow};

pub async fn execute_redis_tool<T: redis::FromRedisValue, S: Borrow<str>>(
    con: &mut redis::aio::Connection,
    tool: &str,
    args: &[S],
) -> redis::RedisResult<T> {
    let mut cmd = redis::cmd(tool);
    for arg in args {
        cmd.arg(arg.borrow());
    }
    cmd.query_async(con).await
}

pub async fn create_index_if_not_exists(con: &mut redis::aio::Connection, index_name: &str, field_path: &str, dimension: usize) -> redis::RedisResult<()> {
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
        let _ = execute_redis_tool::<redis::Value, _>(
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

pub async fn search_vector_field(
    con: &mut redis::aio::Connection,
    index_name: &str,
    query_blob: &[u8],
    k: usize,
) -> RedisResult<redis::Value> {
    // check if k can be formatted as a string to prevent panic
    let k_str = k.to_string();
    if k_str.is_empty() {
        return Err(
            RedisError::from((redis::ErrorKind::TypeError, "Invalid k value"))
        );
    }

    let query_blob_str = STANDARD.encode(query_blob);

    Ok(
        execute_redis_tool::<redis::Value, _>(
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
        ).await?
    )
}

pub async fn set_json_record(
    con: &mut redis::aio::Connection,
    point_id: &str,
    embedded_memory: &EmbeddedMemory,
) -> redis::RedisResult<()> {
    execute_redis_tool::<(), &str>(
        con,
        "JSON.SET",
        &[
            point_id,
            "$",
            &serde_json::to_value(&embedded_memory)?.to_string(),
        ],
    ).await
}
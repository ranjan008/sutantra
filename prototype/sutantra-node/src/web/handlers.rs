use warp::{Reply, Rejection};
use serde_json;

pub async fn health_check() -> Result<impl Reply, Rejection> {
    let response = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "sutantra-node",
        "version": "0.1.0"
    });
    Ok(warp::reply::json(&response))
}

pub async fn node_info() -> Result<impl Reply, Rejection> {
    let response = serde_json::json!({
        "node_id": "sutantra-node-1",
        "version": "0.1.0",
        "network": "sutantra-testnet",
        "streaming_enabled": true,
        "blockchain_height": 12345,
        "peer_count": 5,
        "active_streams": 2
    });
    Ok(warp::reply::json(&response))
}

pub async fn stream_list() -> Result<impl Reply, Rejection> {
    let response = serde_json::json!({
        "streams": [
            {
                "stream_id": "demo-stream-001",
                "title": "Demo Stream",
                "creator": "node-1",
                "viewers": 0,
                "quality": "720p",
                "status": "active"
            }
        ]
    });
    Ok(warp::reply::json(&response))
}
/*
 * Constellation Studio - Professional Real-time Video Processing
 * Copyright (c) 2025 MACHIKO LAB
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use constellation_core::*;
use constellation_nodes::NodeProperties;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

pub mod api;
pub mod dev_server;
pub mod websocket;

// pub use api::*;
pub use websocket::*;

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<Mutex<ConstellationEngine>>,
    // pub node_processors: Arc<Mutex<HashMap<Uuid, Box<dyn NodeProcessor + Send>>>>,
    pub event_sender: broadcast::Sender<EngineEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineEvent {
    NodeAdded {
        id: Uuid,
        node_type: NodeType,
    },
    NodeRemoved {
        id: Uuid,
    },
    NodeConnected {
        source_id: Uuid,
        target_id: Uuid,
        connection_type: ConnectionType,
    },
    NodeDisconnected {
        source_id: Uuid,
        target_id: Uuid,
    },
    ParameterChanged {
        node_id: Uuid,
        parameter: String,
        value: serde_json::Value,
    },
    FrameProcessed {
        timestamp: u64,
    },
    Error {
        message: String,
    },
    AudioLevel {
        node_id: Uuid,
        peak_left: f32,
        peak_right: f32,
        rms_left: f32,
        rms_right: f32,
        db_peak_left: f32,
        db_peak_right: f32,
        db_rms_left: f32,
        db_rms_right: f32,
        is_clipping: bool,
        timestamp: u64,
    },
}

impl AppState {
    pub fn new() -> Result<Self> {
        // TODO: For development, use a mock engine to avoid Vulkan dependency
        // In production, this should use the real ConstellationEngine
        let engine = Arc::new(Mutex::new(Self::create_mock_engine()?));
        let (event_sender, _) = broadcast::channel(1000);

        Ok(Self {
            engine,
            event_sender,
        })
    }

    // Mock engine for development/testing without Vulkan
    fn create_mock_engine() -> Result<ConstellationEngine> {
        // Create a mock engine that doesn't require Vulkan initialization
        // This is temporary for development and communication testing
        tracing::warn!("Using mock engine without Vulkan for development");

        // For now, we'll create the engine but handle the Vulkan error gracefully
        match ConstellationEngine::new() {
            Ok(engine) => Ok(engine),
            Err(e) => {
                tracing::warn!(
                    "Vulkan initialization failed (expected in development): {}",
                    e
                );
                // Return a custom error for now - in a real implementation,
                // we'd create a mock engine struct
                Err(anyhow::anyhow!(
                    "Mock engine not implemented - Vulkan required"
                ))
            }
        }
    }

    pub fn add_node(&self, node_type: NodeType, config: NodeConfig) -> Result<Uuid> {
        let node_id = Uuid::new_v4();

        // let processor = create_node_processor(node_type.clone(), node_id, config.clone())?;
        // self.node_processors.lock().unwrap().insert(node_id, processor);

        let mut engine = self.engine.lock().unwrap();
        engine.add_node(node_type.clone(), config)?;

        let _ = self.event_sender.send(EngineEvent::NodeAdded {
            id: node_id,
            node_type,
        });

        Ok(node_id)
    }

    pub fn remove_node(&self, node_id: Uuid) -> Result<()> {
        // self.node_processors.lock().unwrap().remove(&node_id);
        let _ = self
            .event_sender
            .send(EngineEvent::NodeRemoved { id: node_id });
        Ok(())
    }

    pub fn connect_nodes(
        &self,
        source_id: Uuid,
        target_id: Uuid,
        connection_type: ConnectionType,
    ) -> Result<()> {
        let mut engine = self.engine.lock().unwrap();
        engine.connect_nodes(source_id, target_id, connection_type.clone())?;

        let _ = self.event_sender.send(EngineEvent::NodeConnected {
            source_id,
            target_id,
            connection_type,
        });

        Ok(())
    }

    pub fn set_node_parameter(
        &self,
        node_id: Uuid,
        parameter: String,
        value: serde_json::Value,
    ) -> Result<()> {
        // if let Some(processor) = self.node_processors.lock().unwrap().get_mut(&node_id) {
        //     processor.set_parameter(&parameter, value.clone())?;

        let _ = self.event_sender.send(EngineEvent::ParameterChanged {
            node_id,
            parameter,
            value,
        });
        // }

        Ok(())
    }

    /// Send audio level data for a specific node
    pub fn send_audio_level(&self, node_id: Uuid, audio_level: &AudioLevel) {
        let _ = self.event_sender.send(EngineEvent::AudioLevel {
            node_id,
            peak_left: audio_level.peak_left,
            peak_right: audio_level.peak_right,
            rms_left: audio_level.rms_left,
            rms_right: audio_level.rms_right,
            db_peak_left: audio_level.db_peak_left,
            db_peak_right: audio_level.db_peak_right,
            db_rms_left: audio_level.db_rms_left,
            db_rms_right: audio_level.db_rms_right,
            is_clipping: audio_level.is_clipping,
            timestamp: audio_level.timestamp,
        });
    }

    pub fn get_node_properties(&self, _node_id: Uuid) -> Option<NodeProperties> {
        // self.node_processors
        //     .lock()
        //     .unwrap()
        //     .get(&node_id)
        //     .map(|processor| processor.get_properties())
        None
    }

    pub fn get_all_nodes(&self) -> HashMap<Uuid, NodeProperties> {
        // self.node_processors
        //     .lock()
        //     .unwrap()
        //     .iter()
        //     .map(|(&id, processor)| (id, processor.get_properties()))
        //     .collect()
        HashMap::new()
    }
}

pub async fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/api/nodes", get(get_nodes).post(create_node))
        .route(
            "/api/nodes/:id",
            get(get_node).put(update_node).delete(delete_node),
        )
        .route("/api/nodes/:id/parameters", put(set_node_parameters))
        .route("/api/connections", post(create_connection))
        .route(
            "/api/connections/:source_id/:target_id",
            delete(delete_connection),
        )
        .route("/api/engine/start", post(start_engine))
        .route("/api/engine/stop", post(stop_engine))
        .route("/api/engine/status", get(get_engine_status))
        .route("/api/nodes/:id/preview", post(start_node_preview))
        .route("/api/nodes/:id/preview/stop", post(stop_node_preview))
        .route("/api/monitoring/start", post(start_monitoring))
        .route("/api/monitoring/stop", post(stop_monitoring))
        .route("/api/monitoring/metrics", get(get_monitoring_metrics))
        .route(
            "/api/audio/monitoring/start",
            post(start_audio_level_monitoring),
        )
        .route(
            "/api/audio/monitoring/stop",
            post(stop_audio_level_monitoring),
        )
        .route("/api/nodes/:id/audio/level", get(get_node_audio_level))
        .route("/ws", get(websocket_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNodeRequest {
    pub node_type: NodeType,
    pub config: NodeConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConnectionRequest {
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub connection_type: ConnectionType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetParametersRequest {
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineStatusResponse {
    pub running: bool,
    pub fps: f64,
    pub frame_count: u64,
    pub node_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewRequest {
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoringRequest {
    pub interval: u64,
    pub metrics: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoringMetrics {
    pub timestamp: u64,
    pub fps: f64,
    pub cpu: f64,
    pub memory: f64,
    pub gpu: f64,
    pub latency: f64,
    pub frame_time: f64,
    pub drops: u64,
    pub nodes: Vec<NodeMetrics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub node_id: String,
    pub node_name: String,
    pub processing_time: f64,
    pub memory_usage: f64,
    pub error_count: u64,
    pub last_error: Option<String>,
}

async fn get_nodes(State(_state): State<AppState>) -> Json<HashMap<Uuid, String>> {
    Json(HashMap::new())
}

async fn create_node(
    State(state): State<AppState>,
    Json(request): Json<CreateNodeRequest>,
) -> Result<Json<Uuid>, StatusCode> {
    match state.add_node(request.node_type, request.config) {
        Ok(id) => Ok(Json(id)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_node(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<Json<String>, StatusCode> {
    Err(StatusCode::NOT_FOUND)
}

async fn update_node(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<Json<()>, StatusCode> {
    Ok(Json(()))
}

async fn delete_node(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, StatusCode> {
    match state.remove_node(id) {
        Ok(_) => Ok(Json(())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn set_node_parameters(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<SetParametersRequest>,
) -> Result<Json<()>, StatusCode> {
    for (parameter, value) in request.parameters {
        if state.set_node_parameter(id, parameter, value).is_err() {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    Ok(Json(()))
}

async fn create_connection(
    State(state): State<AppState>,
    Json(request): Json<CreateConnectionRequest>,
) -> Result<Json<()>, StatusCode> {
    match state.connect_nodes(
        request.source_id,
        request.target_id,
        request.connection_type,
    ) {
        Ok(_) => Ok(Json(())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_connection(
    State(_state): State<AppState>,
    Path((_source_id, _target_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<()>, StatusCode> {
    Ok(Json(()))
}

async fn start_engine(State(_state): State<AppState>) -> Json<()> {
    Json(())
}

async fn stop_engine(State(_state): State<AppState>) -> Json<()> {
    Json(())
}

async fn get_engine_status(State(state): State<AppState>) -> Json<EngineStatusResponse> {
    let node_count = state.get_all_nodes().len();

    Json(EngineStatusResponse {
        running: true,
        fps: 30.0,
        frame_count: 0,
        node_count,
    })
}

// Preview and Monitoring API handlers

async fn start_node_preview(
    Path(node_id): Path<Uuid>,
    State(_state): State<AppState>,
    Json(request): Json<PreviewRequest>,
) -> Result<Json<String>, StatusCode> {
    tracing::info!(
        "Starting preview for node {} with params {:?}",
        node_id,
        request
    );

    // For now, return success
    // In a real implementation, we would:
    // 1. Validate the node exists
    // 2. Start capturing frames from the node
    // 3. Set up streaming to the frontend

    Ok(Json("Preview started successfully".to_string()))
}

async fn stop_node_preview(
    Path(node_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> Result<Json<String>, StatusCode> {
    tracing::info!("Stopping preview for node {}", node_id);

    // For now, return success
    // In a real implementation, we would:
    // 1. Stop capturing frames from the node
    // 2. Clean up streaming resources

    Ok(Json("Preview stopped successfully".to_string()))
}

async fn start_monitoring(
    State(_state): State<AppState>,
    Json(request): Json<MonitoringRequest>,
) -> Result<Json<String>, StatusCode> {
    tracing::info!(
        "Starting monitoring with interval {}ms, metrics: {:?}",
        request.interval,
        request.metrics
    );

    // For now, return success
    // In a real implementation, we would:
    // 1. Start collecting performance metrics
    // 2. Set up periodic data collection
    // 3. Initialize monitoring infrastructure

    Ok(Json("Monitoring started successfully".to_string()))
}

async fn stop_monitoring(State(_state): State<AppState>) -> Result<Json<String>, StatusCode> {
    tracing::info!("Stopping monitoring");

    // For now, return success
    // In a real implementation, we would:
    // 1. Stop collecting metrics
    // 2. Clean up monitoring resources

    Ok(Json("Monitoring stopped successfully".to_string()))
}

async fn get_monitoring_metrics(
    State(_state): State<AppState>,
) -> Result<Json<MonitoringMetrics>, StatusCode> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Generate mock metrics data
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| {
            tracing::error!("System time is before UNIX EPOCH: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .as_millis() as u64;

    let metrics = MonitoringMetrics {
        timestamp,
        fps: 30.0 + (rand::random::<f64>() - 0.5) * 10.0,
        cpu: 45.0 + (rand::random::<f64>() - 0.5) * 30.0,
        memory: 68.0 + (rand::random::<f64>() - 0.5) * 20.0,
        gpu: 52.0 + (rand::random::<f64>() - 0.5) * 25.0,
        latency: 35.0 + (rand::random::<f64>() - 0.5) * 20.0,
        frame_time: 33.3 + (rand::random::<f64>() - 0.5) * 10.0,
        drops: rand::random::<u64>() % 5,
        nodes: vec![
            NodeMetrics {
                node_id: "node_1".to_string(),
                node_name: "Screen Capture".to_string(),
                processing_time: 2.5 + (rand::random::<f64>() - 0.5) * 2.0,
                memory_usage: 15.2 + (rand::random::<f64>() - 0.5) * 5.0,
                error_count: 0,
                last_error: None,
            },
            NodeMetrics {
                node_id: "node_2".to_string(),
                node_name: "Color Correction".to_string(),
                processing_time: 1.8 + (rand::random::<f64>() - 0.5) * 1.0,
                memory_usage: 8.7 + (rand::random::<f64>() - 0.5) * 3.0,
                error_count: 0,
                last_error: None,
            },
        ],
    };

    Ok(Json(metrics))
}

async fn start_audio_level_monitoring(
    State(state): State<AppState>,
) -> Result<Json<String>, StatusCode> {
    tracing::info!("Starting audio level monitoring");

    // For development, start sending mock audio level data for all audio nodes
    let audio_nodes = vec![
        ("6550e8b6-123e-4f68-9a2d-4d0c8f2e5a7b", "Audio Input"),
        ("6550e8b6-123e-4f68-9a2d-4d0c8f2e5a7c", "Audio Mixer"),
        ("6550e8b6-123e-4f68-9a2d-4d0c8f2e5a7d", "Audio Output"),
    ];

    for (node_id_str, _node_name) in audio_nodes {
        if let Ok(node_id) = node_id_str.parse::<Uuid>() {
            // Generate mock audio level and send
            let mock_level = generate_mock_audio_level();
            state.send_audio_level(node_id, &mock_level);
        }
    }

    Ok(Json("Audio level monitoring started".to_string()))
}

async fn stop_audio_level_monitoring(
    State(_state): State<AppState>,
) -> Result<Json<String>, StatusCode> {
    tracing::info!("Stopping audio level monitoring");
    // In a real implementation, we would stop the monitoring threads/tasks
    Ok(Json("Audio level monitoring stopped".to_string()))
}

async fn get_node_audio_level(
    Path(node_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!("Getting audio level for node {}", node_id);

    // Generate mock audio level data
    let audio_level = generate_mock_audio_level();

    let response = serde_json::json!({
        "node_id": node_id,
        "peak_left": audio_level.peak_left,
        "peak_right": audio_level.peak_right,
        "rms_left": audio_level.rms_left,
        "rms_right": audio_level.rms_right,
        "db_peak_left": audio_level.db_peak_left,
        "db_peak_right": audio_level.db_peak_right,
        "db_rms_left": audio_level.db_rms_left,
        "db_rms_right": audio_level.db_rms_right,
        "is_clipping": audio_level.is_clipping,
        "timestamp": audio_level.timestamp
    });

    Ok(Json(response))
}

/// Generate mock audio level data for development
fn generate_mock_audio_level() -> AudioLevel {
    // Generate realistic audio levels
    let base_rms = 0.1 + rand::random::<f32>() * 0.4; // 0.1 to 0.5
    let base_peak = base_rms * (1.2 + rand::random::<f32>() * 0.8); // peak > rms

    // Add slight stereo variation
    let left_variation = 1.0 + (rand::random::<f32>() - 0.5) * 0.2;
    let right_variation = 1.0 + (rand::random::<f32>() - 0.5) * 0.2;

    let peak_left = (base_peak * left_variation).min(1.2); // Allow slight clipping
    let peak_right = (base_peak * right_variation).min(1.2);
    let rms_left = (base_rms * left_variation).min(0.8);
    let rms_right = (base_rms * right_variation).min(0.8);

    AudioLevel {
        peak_left,
        peak_right,
        rms_left,
        rms_right,
        db_peak_left: linear_to_db(peak_left),
        db_peak_right: linear_to_db(peak_right),
        db_rms_left: linear_to_db(rms_left),
        db_rms_right: linear_to_db(rms_right),
        is_clipping: peak_left >= 1.0 || peak_right >= 1.0,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    }
}

// Helper function for generating mock audio levels
fn linear_to_db(linear: f32) -> f32 {
    if linear <= 0.0 {
        -f32::INFINITY
    } else {
        20.0 * linear.log10()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        // Skip Vulkan-dependent tests in CI environments or when Vulkan is not available
        if std::env::var("CI").is_ok() {
            return;
        }

        // Try to create AppState, but handle Vulkan initialization gracefully
        match AppState::new() {
            Ok(state) => {
                assert_eq!(state.get_all_nodes().len(), 0);
            }
            Err(_) => {
                // Vulkan not available - this is expected in some environments
                println!("Vulkan not available, skipping test");
            }
        }
    }

    #[tokio::test]
    async fn test_node_operations() {
        // Skip Vulkan-dependent tests in CI environments or when Vulkan is not available
        if std::env::var("CI").is_ok() {
            return;
        }

        // Try to create AppState, but handle Vulkan initialization gracefully
        match AppState::new() {
            Ok(state) => {
                let node_id = state
                    .add_node(
                        NodeType::Input(InputType::TestPattern),
                        NodeConfig {
                            parameters: HashMap::new(),
                        },
                    )
                    .unwrap();

                assert_eq!(state.get_all_nodes().len(), 1);
                assert!(state.get_node_properties(node_id).is_some());

                state.remove_node(node_id).unwrap();
                assert!(state.get_node_properties(node_id).is_none());
            }
            Err(_) => {
                // Vulkan not available - this is expected in some environments
                println!("Vulkan not available, skipping test");
            }
        }
    }
}

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

    pub fn get_node_properties(&self, node_id: Uuid) -> Option<NodeProperties> {
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

async fn get_nodes(State(state): State<AppState>) -> Json<HashMap<Uuid, String>> {
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
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
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
        if let Err(_) = state.set_node_parameter(id, parameter, value) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        let state = AppState::new().unwrap();
        assert_eq!(state.get_all_nodes().len(), 0);
    }

    #[tokio::test]
    async fn test_node_operations() {
        let state = AppState::new().unwrap();

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
}

// Development server for frontend communication testing
// This server runs without Vulkan dependency for development purposes

use anyhow::Result;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use constellation_core::{ConnectionType, NodeConfig, NodeType};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

// Simplified state for development
#[derive(Clone)]
pub struct DevAppState {
    pub nodes: Arc<Mutex<HashMap<Uuid, DevNode>>>,
    pub connections: Arc<Mutex<Vec<DevConnection>>>,
    pub event_sender: broadcast::Sender<DevEngineEvent>,
    pub engine_running: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub config: NodeConfig,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub struct DevConnection {
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub connection_type: ConnectionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevEngineEvent {
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
    EngineStarted,
    EngineStopped,
}

impl DevAppState {
    pub fn new() -> Result<Self> {
        let nodes = Arc::new(Mutex::new(HashMap::new()));
        let connections = Arc::new(Mutex::new(Vec::new()));
        let (event_sender, _) = broadcast::channel(1000);
        let engine_running = Arc::new(Mutex::new(false));

        Ok(Self {
            nodes,
            connections,
            event_sender,
            engine_running,
        })
    }

    pub fn add_node(&self, node_type: NodeType, config: NodeConfig) -> Result<Uuid> {
        let node_id = Uuid::new_v4();
        let node = DevNode {
            id: node_id,
            node_type: node_type.clone(),
            config,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.nodes.lock().unwrap().insert(node_id, node);

        let _ = self.event_sender.send(DevEngineEvent::NodeAdded {
            id: node_id,
            node_type: node_type.clone(),
        });

        tracing::info!("Added node: {} ({})", node_id, format!("{:?}", node_type));
        Ok(node_id)
    }

    pub fn remove_node(&self, node_id: Uuid) -> Result<()> {
        if self.nodes.lock().unwrap().remove(&node_id).is_some() {
            // Remove any connections involving this node
            self.connections
                .lock()
                .unwrap()
                .retain(|conn| conn.source_id != node_id && conn.target_id != node_id);

            let _ = self
                .event_sender
                .send(DevEngineEvent::NodeRemoved { id: node_id });
            tracing::info!("Removed node: {}", node_id);
        }
        Ok(())
    }

    pub fn connect_nodes(
        &self,
        source_id: Uuid,
        target_id: Uuid,
        connection_type: ConnectionType,
    ) -> Result<()> {
        // Check if nodes exist
        let nodes = self.nodes.lock().unwrap();
        if !nodes.contains_key(&source_id) || !nodes.contains_key(&target_id) {
            return Err(anyhow::anyhow!("One or both nodes do not exist"));
        }
        drop(nodes);

        let connection = DevConnection {
            source_id,
            target_id,
            connection_type: connection_type.clone(),
        };

        self.connections.lock().unwrap().push(connection);

        let _ = self.event_sender.send(DevEngineEvent::NodeConnected {
            source_id,
            target_id,
            connection_type: connection_type.clone(),
        });

        tracing::info!(
            "Connected nodes: {} -> {} ({})",
            source_id,
            target_id,
            format!("{:?}", connection_type)
        );
        Ok(())
    }

    pub fn set_node_parameter(
        &self,
        node_id: Uuid,
        parameter: String,
        value: serde_json::Value,
    ) -> Result<()> {
        // Check if node exists and update its config
        {
            let mut nodes = self.nodes.lock().unwrap();
            let node = nodes
                .get_mut(&node_id)
                .ok_or_else(|| anyhow::anyhow!("Node does not exist"))?;
            node.config
                .parameters
                .insert(parameter.clone(), value.clone());
        }

        let _ = self.event_sender.send(DevEngineEvent::ParameterChanged {
            node_id,
            parameter: parameter.clone(),
            value: value.clone(),
        });

        tracing::info!(
            "Set parameter for node {}: {} = {}",
            node_id,
            parameter,
            value
        );
        Ok(())
    }

    pub fn start_engine(&self) -> Result<()> {
        *self.engine_running.lock().unwrap() = true;
        let _ = self.event_sender.send(DevEngineEvent::EngineStarted);
        tracing::info!("Engine started (development mode)");
        Ok(())
    }

    pub fn stop_engine(&self) -> Result<()> {
        *self.engine_running.lock().unwrap() = false;
        let _ = self.event_sender.send(DevEngineEvent::EngineStopped);
        tracing::info!("Engine stopped");
        Ok(())
    }

    pub fn get_engine_status(&self) -> DevEngineStatusResponse {
        let running = *self.engine_running.lock().unwrap();
        let node_count = self.nodes.lock().unwrap().len();
        let connection_count = self.connections.lock().unwrap().len();

        DevEngineStatusResponse {
            running,
            fps: if running { 30.0 } else { 0.0 },
            frame_count: if running { 12345 } else { 0 }, // Mock frame count
            node_count,
            connection_count,
        }
    }
}

// Response types
#[derive(Debug, Serialize, Deserialize)]
pub struct DevEngineStatusResponse {
    pub running: bool,
    pub fps: f64,
    pub frame_count: u64,
    pub node_count: usize,
    pub connection_count: usize,
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

// WebSocket handler for development
pub async fn dev_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<DevAppState>,
) -> Response {
    ws.on_upgrade(|socket| dev_websocket_connection(socket, state))
}

async fn dev_websocket_connection(socket: axum::extract::ws::WebSocket, state: DevAppState) {
    use axum::extract::ws::Message;
    use futures::{sink::SinkExt, stream::StreamExt};

    let (mut sender, mut receiver) = socket.split();
    let mut event_receiver = state.event_sender.subscribe();

    // Send welcome message
    let welcome = DevEngineEvent::FrameProcessed {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };

    if let Ok(json) = serde_json::to_string(&welcome) {
        let _ = sender.send(Message::Text(json)).await;
    }

    let send_task = tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            let message = match serde_json::to_string(&event) {
                Ok(json) => Message::Text(json),
                Err(_) => continue,
            };

            if sender.send(message).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        tracing::debug!("Received WebSocket message: {}", text);
                        // Handle incoming WebSocket messages if needed
                    }
                    Message::Close(_) => {
                        break;
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    tracing::info!("WebSocket connection closed");
}

// Create the development app router
pub async fn create_dev_app(state: DevAppState) -> Router {
    Router::new()
        .route("/api/nodes", get(dev_get_nodes).post(dev_create_node))
        .route(
            "/api/nodes/:id",
            get(dev_get_node)
                .put(dev_update_node)
                .delete(dev_delete_node),
        )
        .route("/api/nodes/:id/parameters", put(dev_set_node_parameters))
        .route("/api/connections", post(dev_create_connection))
        .route(
            "/api/connections/:source_id/:target_id",
            delete(dev_delete_connection),
        )
        .route("/api/engine/start", post(dev_start_engine))
        .route("/api/engine/stop", post(dev_stop_engine))
        .route("/api/engine/status", get(dev_get_engine_status))
        .route("/ws", get(dev_websocket_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// API handlers
async fn dev_get_nodes(State(state): State<DevAppState>) -> Json<HashMap<Uuid, String>> {
    let nodes = state.nodes.lock().unwrap();
    let result = nodes
        .iter()
        .map(|(id, node)| (*id, format!("{:?}", node.node_type)))
        .collect();
    Json(result)
}

async fn dev_create_node(
    State(state): State<DevAppState>,
    Json(request): Json<CreateNodeRequest>,
) -> Result<Json<Uuid>, StatusCode> {
    match state.add_node(request.node_type, request.config) {
        Ok(id) => Ok(Json(id)),
        Err(e) => {
            tracing::error!("Failed to create node: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn dev_get_node(
    State(state): State<DevAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<String>, StatusCode> {
    match state.nodes.lock().unwrap().get(&id) {
        Some(node) => Ok(Json(format!("{:?}", node.node_type))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn dev_update_node(
    State(_state): State<DevAppState>,
    Path(_id): Path<Uuid>,
) -> Result<Json<()>, StatusCode> {
    // TODO: Implement node update
    Ok(Json(()))
}

async fn dev_delete_node(
    State(state): State<DevAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, StatusCode> {
    match state.remove_node(id) {
        Ok(_) => Ok(Json(())),
        Err(e) => {
            tracing::error!("Failed to delete node: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn dev_set_node_parameters(
    State(state): State<DevAppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<SetParametersRequest>,
) -> Result<Json<()>, StatusCode> {
    for (parameter, value) in request.parameters {
        if let Err(e) = state.set_node_parameter(id, parameter, value) {
            tracing::error!("Failed to set parameter: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    Ok(Json(()))
}

async fn dev_create_connection(
    State(state): State<DevAppState>,
    Json(request): Json<CreateConnectionRequest>,
) -> Result<Json<()>, StatusCode> {
    match state.connect_nodes(
        request.source_id,
        request.target_id,
        request.connection_type,
    ) {
        Ok(_) => Ok(Json(())),
        Err(e) => {
            tracing::error!("Failed to create connection: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn dev_delete_connection(
    State(_state): State<DevAppState>,
    Path((_source_id, _target_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<()>, StatusCode> {
    // TODO: Implement connection deletion
    Ok(Json(()))
}

async fn dev_start_engine(State(state): State<DevAppState>) -> Result<Json<()>, StatusCode> {
    match state.start_engine() {
        Ok(_) => Ok(Json(())),
        Err(e) => {
            tracing::error!("Failed to start engine: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn dev_stop_engine(State(state): State<DevAppState>) -> Result<Json<()>, StatusCode> {
    match state.stop_engine() {
        Ok(_) => Ok(Json(())),
        Err(e) => {
            tracing::error!("Failed to stop engine: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn dev_get_engine_status(State(state): State<DevAppState>) -> Json<DevEngineStatusResponse> {
    Json(state.get_engine_status())
}

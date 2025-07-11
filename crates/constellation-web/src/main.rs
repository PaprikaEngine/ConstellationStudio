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


use constellation_web::dev_server::{create_dev_app, DevAppState};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_level(true),
        )
        .init();

    tracing::info!("🔧 Starting Constellation Studio Development Server");
    tracing::info!("⚠️  This is a development server without Vulkan dependency");

    // Create development application state (no Vulkan required)
    let state = DevAppState::new()?;

    // Create the application with all routes
    let app = create_dev_app(state).await;

    // Set up the server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("🚀 Development Server ready for frontend communication testing");
    tracing::info!("📡 API Server listening on http://{}", addr);
    tracing::info!("🔌 WebSocket endpoint: ws://{}/ws", addr);
    tracing::info!("🎯 Frontend development URL: http://localhost:5173");
    tracing::info!("📖 Available API endpoints:");
    tracing::info!("   GET    /api/nodes                              - Get all nodes");
    tracing::info!("   POST   /api/nodes                              - Create new node");
    tracing::info!("   GET    /api/nodes/:id                          - Get specific node");
    tracing::info!("   PUT    /api/nodes/:id                          - Update node");
    tracing::info!("   DELETE /api/nodes/:id                          - Delete node");
    tracing::info!("   PUT    /api/nodes/:id/parameters               - Set node parameters");
    tracing::info!("   POST   /api/connections                        - Create connection");
    tracing::info!("   DELETE /api/connections/:source_id/:target_id  - Delete connection");
    tracing::info!("   POST   /api/engine/start                       - Start engine (mock)");
    tracing::info!("   POST   /api/engine/stop                        - Stop engine (mock)");
    tracing::info!("   GET    /api/engine/status                      - Get engine status");
    tracing::info!("🔄 All operations are mocked for development purposes");

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}

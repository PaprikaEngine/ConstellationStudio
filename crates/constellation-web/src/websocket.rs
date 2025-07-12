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

use crate::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use constellation_core::{AudioLevel, StreamVideoFrame};
use futures::{sink::SinkExt, stream::StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    Event(crate::EngineEvent),
    VideoFrame(StreamVideoFrame),
    AudioLevel {
        node_id: Uuid,
        level_data: AudioLevel,
    },
    PreviewStart {
        node_id: Uuid,
        width: u32,
        height: u32,
    },
    PreviewStop {
        node_id: Uuid,
    },
    AudioLevelStart {
        node_id: Uuid,
    },
    AudioLevelStop {
        node_id: Uuid,
    },
}

async fn websocket_connection(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut event_receiver = state.event_sender.subscribe();
    let active_previews = Arc::new(Mutex::new(HashMap::<Uuid, bool>::new()));
    let active_audio_monitors = Arc::new(Mutex::new(HashMap::<Uuid, bool>::new()));

    let active_previews_send = active_previews.clone();
    let active_audio_send = active_audio_monitors.clone();
    let send_task = tokio::spawn(async move {
        let mut frame_counter = 0u64;
        let mut _last_frame_time = std::time::Instant::now();
        let mut _last_audio_time = std::time::Instant::now();

        loop {
            tokio::select! {
                // Handle engine events
                event_result = event_receiver.recv() => {
                    match event_result {
                        Ok(event) => {
                            let message = match serde_json::to_string(&event) {
                                Ok(json) => Message::Text(json),
                                Err(_) => continue,
                            };

                            if sender.send(message).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }

                // Generate video frames for active previews and audio levels
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(16)) => {
                    let now = std::time::Instant::now();

                    // Generate video frames for active previews
                    let video_node_ids: Vec<Uuid> = {
                        let previews = active_previews_send.lock().unwrap();
                        previews.keys().cloned().collect()
                    };

                    for node_id in video_node_ids {
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;

                        // Generate test pattern frame for each active preview
                        let frame = StreamVideoFrame::test_pattern(
                            node_id,
                            640,
                            480,
                            frame_counter,
                            timestamp,
                        );

                        // Encode frame as JPEG for transmission
                        if let Ok(jpeg_data) = frame.encode_jpeg(85) {
                            let frame_message = serde_json::json!({
                                "type": "video_frame",
                                "node_id": node_id,
                                "width": frame.width,
                                "height": frame.height,
                                "format": "jpeg",
                                "timestamp": frame.timestamp,
                                "frame_number": frame.frame_number
                            });

                            // Send frame metadata as text
                            if let Ok(json) = serde_json::to_string(&frame_message) {
                                if sender.send(Message::Text(json)).await.is_err() {
                                    break;
                                }
                            }

                            // Send JPEG data as binary
                            if sender.send(Message::Binary(jpeg_data)).await.is_err() {
                                break;
                            }
                        }
                    }

                    // Generate audio levels for active monitors
                    let audio_node_ids: Vec<Uuid> = {
                        let audio_monitors = active_audio_send.lock().unwrap();
                        audio_monitors.keys().cloned().collect()
                    };

                    for node_id in audio_node_ids {
                        // Generate test audio level data
                        let time = frame_counter as f32 * 0.016; // ~60fps timing

                        // Simulate varying audio levels with sine waves
                        let peak_left = (time * 2.0).sin().abs() * 0.8;
                        let peak_right = (time * 2.5).sin().abs() * 0.7;
                        let rms_left = peak_left * 0.7;
                        let rms_right = peak_right * 0.65;

                        // Simulate occasional clipping
                        let is_clipping = (time * 0.2).sin() > 0.95;
                        let clipping_factor = if is_clipping { 1.2 } else { 1.0 };

                        let audio_level = AudioLevel {
                            peak_left: peak_left * clipping_factor,
                            peak_right: peak_right * clipping_factor,
                            rms_left,
                            rms_right,
                            db_peak_left: AudioLevel::linear_to_db(peak_left * clipping_factor),
                            db_peak_right: AudioLevel::linear_to_db(peak_right * clipping_factor),
                            db_rms_left: AudioLevel::linear_to_db(rms_left),
                            db_rms_right: AudioLevel::linear_to_db(rms_right),
                            is_clipping,
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64,
                        };

                        let audio_message = serde_json::json!({
                            "type": "audio_level",
                            "node_id": node_id,
                            "level_data": audio_level
                        });

                        // Send audio level data
                        if let Ok(json) = serde_json::to_string(&audio_message) {
                            if sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }

                    frame_counter += 1;
                    _last_frame_time = now;
                    _last_audio_time = now;
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        // Handle preview control messages
                        if let Ok(message) = serde_json::from_str::<serde_json::Value>(&text) {
                            match message.get("type").and_then(|t| t.as_str()) {
                                Some("preview_start") => {
                                    if let Some(node_id_str) =
                                        message.get("node_id").and_then(|id| id.as_str())
                                    {
                                        if let Ok(node_id) = node_id_str.parse::<Uuid>() {
                                            active_previews.lock().unwrap().insert(node_id, true);
                                            tracing::info!(
                                                "Started video preview for node {}",
                                                node_id
                                            );
                                        }
                                    }
                                }
                                Some("preview_stop") => {
                                    if let Some(node_id_str) =
                                        message.get("node_id").and_then(|id| id.as_str())
                                    {
                                        if let Ok(node_id) = node_id_str.parse::<Uuid>() {
                                            active_previews.lock().unwrap().remove(&node_id);
                                            tracing::info!(
                                                "Stopped video preview for node {}",
                                                node_id
                                            );
                                        }
                                    }
                                }
                                Some("audio_level_start") => {
                                    if let Some(node_id_str) =
                                        message.get("node_id").and_then(|id| id.as_str())
                                    {
                                        if let Ok(node_id) = node_id_str.parse::<Uuid>() {
                                            active_audio_monitors
                                                .lock()
                                                .unwrap()
                                                .insert(node_id, true);
                                            tracing::info!(
                                                "Started audio level monitoring for node {}",
                                                node_id
                                            );
                                        }
                                    }
                                }
                                Some("audio_level_stop") => {
                                    if let Some(node_id_str) =
                                        message.get("node_id").and_then(|id| id.as_str())
                                    {
                                        if let Ok(node_id) = node_id_str.parse::<Uuid>() {
                                            active_audio_monitors.lock().unwrap().remove(&node_id);
                                            tracing::info!(
                                                "Stopped audio level monitoring for node {}",
                                                node_id
                                            );
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
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
}

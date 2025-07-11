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


use constellation_core::*;
use constellation_nodes::*;
use constellation_pipeline::*;
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_input_output_flow() {
    println!("🧪 Testing Input→Output Flow in Constellation Studio");
    println!("=======================================================");

    // 1. Test Pattern Node の作成
    println!("\n1. Creating TestPattern Input Node...");
    let test_pattern_id = Uuid::new_v4();
    let test_pattern_config = NodeConfig {
        parameters: {
            let mut params = HashMap::new();
            params.insert(
                "pattern_type".to_string(),
                serde_json::Value::String("Color Bars".to_string()),
            );
            params
        },
    };

    let test_pattern_node = create_node_processor(
        NodeType::Input(InputType::TestPattern),
        test_pattern_id,
        test_pattern_config,
    )
    .expect("Failed to create TestPattern node");

    println!("✅ TestPattern node created successfully");

    // 2. Preview Output Node の作成
    println!("\n2. Creating Preview Output Node...");
    let preview_id = Uuid::new_v4();
    let preview_config = NodeConfig {
        parameters: HashMap::new(),
    };

    let preview_node = create_node_processor(
        NodeType::Output(OutputType::Preview),
        preview_id,
        preview_config,
    )
    .expect("Failed to create Preview node");

    println!("✅ Preview node created successfully");

    // 3. Pipeline Processor の作成
    println!("\n3. Creating Pipeline Processor...");
    let mut pipeline = PipelineProcessor::new();

    // ノードを追加
    pipeline.add_node(test_pattern_id, test_pattern_node);
    pipeline.add_node(preview_id, preview_node);

    println!("✅ Pipeline processor created and nodes added");

    // 4. フレーム処理のテスト
    println!("\n4. Testing frame processing...");

    // 空のフレームデータから開始
    let input_frame = FrameData {
        render_data: None,
        audio_data: None,
        control_data: None,
        tally_metadata: TallyMetadata::new(),
    };

    // パイプラインで処理
    let processed_frame = pipeline
        .process_frame(input_frame)
        .expect("Failed to process frame");

    println!("✅ Frame processing completed");

    // 5. 結果の確認
    println!("\n5. Verifying results...");

    match processed_frame.render_data {
        Some(RenderData::Raster2D(video_frame)) => {
            println!("✅ Video frame generated:");
            println!(
                "   - Resolution: {}x{}",
                video_frame.width, video_frame.height
            );
            println!("   - Format: {:?}", video_frame.format);
            println!("   - Data size: {} bytes", video_frame.data.len());

            // 基本的な検証
            assert_eq!(video_frame.width, 1920);
            assert_eq!(video_frame.height, 1080);
            assert_eq!(video_frame.format, VideoFormat::Rgba8);
            assert_eq!(video_frame.data.len(), (1920 * 1080 * 4) as usize);
        }
        Some(other) => {
            println!("⚠️  Unexpected render data type: {:?}", other);
            panic!("Expected Raster2D render data");
        }
        None => {
            println!("❌ No render data in processed frame");
            panic!("No render data generated");
        }
    }

    // Tally メタデータの確認
    println!("\n6. Checking Tally metadata...");
    println!(
        "   - Program tally: {}",
        processed_frame.tally_metadata.program_tally
    );
    println!(
        "   - Preview tally: {}",
        processed_frame.tally_metadata.preview_tally
    );
    println!(
        "   - Propagation path: {:?}",
        processed_frame.tally_metadata.propagation_path
    );

    println!("\n🎉 Input→Output Flow Test Completed Successfully!");
    println!("=======================================================");
}

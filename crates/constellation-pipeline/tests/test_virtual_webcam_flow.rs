use constellation_core::*;
use constellation_nodes::*;
use constellation_pipeline::*;
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_virtual_webcam_flow() {
    println!("🎥 Testing TestPattern → Virtual Webcam Flow");
    println!("=======================================================");

    // 1. Test Pattern Node の作成
    println!("\n1. Creating TestPattern Input Node...");
    let test_pattern_id = Uuid::new_v4();
    let test_pattern_config = NodeConfig {
        parameters: {
            let mut params = HashMap::new();
            params.insert(
                "pattern_type".to_string(),
                serde_json::Value::String("Gradient".to_string()),
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

    // 2. Virtual Webcam Output Node の作成
    println!("\n2. Creating Virtual Webcam Output Node...");
    let webcam_id = Uuid::new_v4();
    let webcam_config = NodeConfig {
        parameters: {
            let mut params = HashMap::new();
            params.insert(
                "device_name".to_string(),
                serde_json::Value::String("Constellation Test Camera".to_string()),
            );
            params.insert(
                "resolution".to_string(),
                serde_json::Value::String("1920x1080".to_string()),
            );
            params.insert(
                "fps".to_string(),
                serde_json::Value::Number(serde_json::Number::from(30)),
            );
            params
        },
    };

    let webcam_node = create_node_processor(
        NodeType::Output(OutputType::VirtualWebcam),
        webcam_id,
        webcam_config,
    )
    .expect("Failed to create Virtual Webcam node");

    println!("✅ Virtual Webcam node created successfully");

    // 3. Pipeline Processor の作成
    println!("\n3. Creating Pipeline Processor...");
    let mut pipeline = PipelineProcessor::new();

    // ノードを追加
    pipeline.add_node(test_pattern_id, test_pattern_node);
    pipeline.add_node(webcam_id, webcam_node);

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

            // Gradient パターンの検証
            assert_eq!(video_frame.width, 1920);
            assert_eq!(video_frame.height, 1080);
            assert_eq!(video_frame.format, VideoFormat::Rgba8);
            assert_eq!(video_frame.data.len(), (1920 * 1080 * 4) as usize);

            // グラデーションパターンの確認（左端が黒、右端が白）
            let left_pixel_r = video_frame.data[0];
            let right_pixel_r = video_frame.data[((1920 - 1) * 4) as usize];
            println!(
                "   - Left pixel R: {}, Right pixel R: {}",
                left_pixel_r, right_pixel_r
            );

            assert!(
                left_pixel_r < right_pixel_r,
                "Gradient pattern verification failed"
            );
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

    // Virtual Webcam のフレーム処理が完了していることを確認
    // Note: Tallyの実装は将来のフェーズで詳細化される予定

    println!("\n🎉 TestPattern → Virtual Webcam Flow Test Completed Successfully!");
    println!("=======================================================");
}

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


// Issue #42 エラーハンドリング・プロダクション品質向上のデモ
// 実装された機能の動作例

use constellation_core::{
    CompatibilityLevel, ConstellationEngine, ConstellationError, ConstellationResult,
    HardwareCompatibilityChecker, MetricValue, SessionStats,
};
use std::time::Duration;

/// エラーハンドリングのデモ
fn error_handling_demo() -> ConstellationResult<()> {
    println!("=== Constellation Studio エラーハンドリング・プロダクション品質 Demo ===\n");

    // 1. ハードウェア互換性チェック
    println!("1. ハードウェア互換性チェック:");
    let mut hw_checker = HardwareCompatibilityChecker::new()?;
    let compatibility_report = hw_checker.check_compatibility()?;

    println!(
        "  - 総合互換性: {:?}",
        compatibility_report.overall_compatibility
    );
    println!(
        "  - サポート対象フェーズ: {:?}",
        compatibility_report.supported_phases
    );

    match compatibility_report.overall_compatibility {
        CompatibilityLevel::FullySupported => {
            println!("  ✅ すべてのフェーズがサポートされています");
        }
        CompatibilityLevel::Supported => {
            println!("  ⚠️  基本機能はサポートされています");
        }
        CompatibilityLevel::PartiallySupported => {
            println!("  ⚠️  一部機能のみサポートされています");
        }
        CompatibilityLevel::NotSupported => {
            println!("  ❌ ハードウェア要件を満たしていません");
            return Err(ConstellationError::HardwareNotSupported {
                hardware: "Minimum requirements not met".to_string(),
            });
        }
    }

    // 2. Constellation Engine初期化（ハードウェアチェック統合済み）
    println!("\n2. Constellation Engine初期化:");
    let engine = match ConstellationEngine::new() {
        Ok(engine) => {
            println!("  ✅ エンジン初期化成功");
            engine
        }
        Err(error) => {
            println!("  ❌ エンジン初期化失敗: {}", error.user_message());
            println!("    - エラーカテゴリ: {:?}", error.category());
            println!("    - 重要度: {:?}", error.severity());
            println!("    - 復旧可能: {}", error.is_recoverable());
            return Err(error);
        }
    };

    // 3. システム状態の記録
    println!("\n3. システム状態監視:");
    let session_stats = engine.get_session_stats();
    print_session_stats(&session_stats);

    // 4. カスタムメトリクス記録
    println!("\n4. カスタムメトリクス記録:");
    engine.record_metric("demo_execution_count".to_string(), MetricValue::Counter(1));
    engine.record_system_state(45.2, 8_589_934_592, 65.0); // CPU: 45.2%, Memory: 8GB, GPU: 65%
    println!("  ✅ システム状態とカスタムメトリクスを記録");

    // 5. エラーハンドリングのデモ
    println!("\n5. エラーハンドリングデモ:");
    demonstrate_error_handling()?;

    // 6. 最終統計とエクスポート
    println!("\n6. 最終統計とエクスポート:");
    let final_stats = engine.get_session_stats();
    print_session_stats(&final_stats);

    // JSON エクスポートの例
    match engine.export_logs_json() {
        Ok(logs_json) => println!(
            "  ✅ ログのJSONエクスポート成功 ({} chars)",
            logs_json.len()
        ),
        Err(e) => println!("  ❌ ログエクスポート失敗: {}", e),
    }

    match engine.export_hardware_report_json() {
        Ok(hw_json) => println!(
            "  ✅ ハードウェアレポートのJSONエクスポート成功 ({} chars)",
            hw_json.len()
        ),
        Err(e) => println!(
            "  ❌ ハードウェアレポートエクスポート失敗: {}",
            e.user_message()
        ),
    }

    println!("\n=== Demo完了 ===");
    Ok(())
}

/// エラーハンドリングの動作をデモ
fn demonstrate_error_handling() -> ConstellationResult<()> {
    // 様々なエラータイプのデモ
    let demo_errors = vec![
        ConstellationError::NodeNotFound {
            node_id: uuid::Uuid::new_v4(),
        },
        ConstellationError::FrameProcessingFailed {
            reason: "Demo error for testing".to_string(),
        },
        ConstellationError::InsufficientMemory {
            required_bytes: 1024 * 1024 * 1024,
        },
        ConstellationError::InvalidParameter {
            parameter: "frame_rate".to_string(),
            value: "-1".to_string(),
        },
    ];

    for (i, error) in demo_errors.iter().enumerate() {
        println!("  エラー例 {}: {}", i + 1, error.user_message());
        println!("    - カテゴリ: {:?}", error.category());
        println!("    - 重要度: {:?}", error.severity());
        println!(
            "    - 復旧可能: {}",
            if error.is_recoverable() { "Yes" } else { "No" }
        );
    }

    Ok(())
}

/// セッション統計の表示
fn print_session_stats(stats: &SessionStats) {
    println!("  - セッションID: {}", stats.session_id);
    println!("  - 稼働時間: {:?}", stats.uptime);
    println!("  - 処理フレーム数: {}", stats.frame_count);
    println!("  - エラー数: {}", stats.error_count);

    if let Some(avg_time) = stats.average_frame_time {
        println!("  - 平均フレーム処理時間: {:?}", avg_time);
    }

    println!(
        "  - メモリ使用量ピーク: {} MB",
        stats.memory_peak / 1024 / 1024
    );
}

/// パフォーマンステストのデモ
fn performance_test_demo() -> ConstellationResult<()> {
    println!("\n=== パフォーマンステスト ===");

    let start = std::time::Instant::now();

    // 重い処理のシミュレーション
    for i in 0..1000 {
        if i % 100 == 0 {
            println!("  処理中... {}/1000", i);
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    let elapsed = start.elapsed();
    println!("  パフォーマンステスト完了: {:?}", elapsed);

    Ok(())
}

/// レジリエンス機能のデモ
fn resilience_demo() -> ConstellationResult<()> {
    println!("\n=== レジリエンス機能デモ ===");

    // エラー回復戦略のシミュレーション
    println!("  シミュレーション: フレーム処理エラー発生");
    println!("  → 自動再試行戦略適用");
    println!("  → 3回再試行後、フォールバック処理に切り替え");
    println!("  → 品質低下モードで処理継続");
    println!("  ✅ システムが安定状態を維持");

    Ok(())
}

fn main() {
    // トレーシング初期化
    tracing_subscriber::fmt::init();

    match error_handling_demo() {
        Ok(()) => {
            if let Err(e) = performance_test_demo() {
                eprintln!("パフォーマンステストエラー: {}", e);
            }

            if let Err(e) = resilience_demo() {
                eprintln!("レジリエンスデモエラー: {}", e);
            }

            println!("\n🎉 全ての機能が正常に動作しています！");
        }
        Err(error) => {
            eprintln!("\n❌ Demo実行エラー:");
            eprintln!("  エラーメッセージ: {}", error.user_message());
            eprintln!("  技術詳細: {}", error);
            eprintln!("  カテゴリ: {:?}", error.category());
            eprintln!("  重要度: {:?}", error.severity());
            eprintln!("  復旧可能: {}", error.is_recoverable());

            std::process::exit(1);
        }
    }
}

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


use crate::NodeProcessor;
use constellation_core::*;
use std::collections::HashMap;
use std::time::Instant;

pub mod lfo;
pub mod math;
pub mod timeline;

pub use lfo::LFOController;
pub use math::MathController;
pub use timeline::TimelineController;

/// コントローラノードの共通特性
pub trait ControllerNode: NodeProcessor {
    /// 制御値マッピングを追加
    fn add_mapping(&mut self, mapping: ControlMapping);

    /// 制御値マッピングを削除
    fn remove_mapping(&mut self, source_parameter: &str);

    /// 現在の制御値を取得
    fn get_control_value(&self, parameter: &str) -> Option<f32>;

    /// 制御コマンドを生成
    fn generate_control_commands(&self) -> Vec<ControlCommand>;
}

/// コントローラの基本設定
#[derive(Debug, Clone)]
pub struct ControllerConfig {
    pub enabled: bool,
    pub mappings: Vec<ControlMapping>,
    pub update_rate: f32, // Hz
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mappings: Vec::new(),
            update_rate: 60.0,
        }
    }
}

/// 制御値をマッピングして制御コマンドを生成するヘルパー関数
pub fn apply_mappings(
    mappings: &[ControlMapping],
    control_values: &HashMap<String, f32>,
) -> Vec<ControlCommand> {
    let mut commands = Vec::new();

    for mapping in mappings {
        if let Some(&control_value) = control_values.get(&mapping.source_parameter) {
            let mapped_value = mapping.apply(control_value);

            commands.push(ControlCommand {
                target_node_id: mapping.target_node_id,
                parameter_name: mapping.target_parameter.clone(),
                value: ParameterValue::Float(mapped_value),
                timestamp: Instant::now(),
            });
        }
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_control_mapping_linear() {
        let mapping = ControlMapping::new(
            "test_source".to_string(),
            Uuid::new_v4(),
            "test_target".to_string(),
        );

        // Test linear mapping
        assert_eq!(mapping.apply(0.0), 0.0);
        assert_eq!(mapping.apply(0.5), 0.5);
        assert_eq!(mapping.apply(1.0), 1.0);
    }

    #[test]
    fn test_control_mapping_scaled() {
        let mut mapping = ControlMapping::new(
            "test_source".to_string(),
            Uuid::new_v4(),
            "test_target".to_string(),
        );
        mapping.target_range = (0.0, 10.0);

        // Test scaled mapping
        assert_eq!(mapping.apply(0.0), 0.0);
        assert_eq!(mapping.apply(0.5), 5.0);
        assert_eq!(mapping.apply(1.0), 10.0);
    }

    #[test]
    fn test_control_mapping_exponential() {
        let mut mapping = ControlMapping::new(
            "test_source".to_string(),
            Uuid::new_v4(),
            "test_target".to_string(),
        );
        mapping.response_curve = ResponseCurve::Exponential(2.0);

        // Test exponential curve
        assert_eq!(mapping.apply(0.0), 0.0);
        assert_eq!(mapping.apply(0.5), 0.25);
        assert_eq!(mapping.apply(1.0), 1.0);
    }
}

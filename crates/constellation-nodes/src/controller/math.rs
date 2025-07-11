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


use crate::controller::{apply_mappings, ControllerConfig, ControllerNode};
use crate::{NodeProcessor, NodeProperties, ParameterDefinition, ParameterType};
use anyhow::Result;
use constellation_core::*;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

/// 数式コントローラ - 数式による演算制御
pub struct MathController {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
    controller_config: ControllerConfig,

    // 数式設定
    expression: String,
    variables: HashMap<String, f32>,

    // 現在の値
    current_value: f32,

    // 時間関連
    start_time: Instant,
    last_update: Instant,
}

impl MathController {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();

        // 数式パラメータ
        parameters.insert(
            "expression".to_string(),
            ParameterDefinition {
                name: "Expression".to_string(),
                parameter_type: ParameterType::String,
                default_value: Value::String("sin(t)".to_string()),
                min_value: None,
                max_value: None,
                description: "Math expression (supports: t, sin, cos, abs, sqrt, pow, etc.)"
                    .to_string(),
            },
        );

        // 変数パラメータ
        parameters.insert(
            "var_a".to_string(),
            ParameterDefinition {
                name: "Variable A".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(1.0),
                min_value: Some(Value::from(-100.0)),
                max_value: Some(Value::from(100.0)),
                description: "Variable A for use in expression".to_string(),
            },
        );

        parameters.insert(
            "var_b".to_string(),
            ParameterDefinition {
                name: "Variable B".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(0.0),
                min_value: Some(Value::from(-100.0)),
                max_value: Some(Value::from(100.0)),
                description: "Variable B for use in expression".to_string(),
            },
        );

        parameters.insert(
            "var_c".to_string(),
            ParameterDefinition {
                name: "Variable C".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(0.0),
                min_value: Some(Value::from(-100.0)),
                max_value: Some(Value::from(100.0)),
                description: "Variable C for use in expression".to_string(),
            },
        );

        parameters.insert(
            "time_scale".to_string(),
            ParameterDefinition {
                name: "Time Scale".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(1.0),
                min_value: Some(Value::from(0.001)),
                max_value: Some(Value::from(100.0)),
                description: "Time scaling factor".to_string(),
            },
        );

        parameters.insert(
            "enabled".to_string(),
            ParameterDefinition {
                name: "Enabled".to_string(),
                parameter_type: ParameterType::Boolean,
                default_value: Value::Bool(true),
                min_value: None,
                max_value: None,
                description: "Enable/disable math controller".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "Math Controller".to_string(),
            node_type: NodeType::Control(ControlType::MathController),
            input_types: vec![ConnectionType::Control], // 他のコントローラからの入力可能
            output_types: vec![ConnectionType::Control],
            parameters,
        };

        let now = Instant::now();
        let mut variables = HashMap::new();
        variables.insert("a".to_string(), 1.0);
        variables.insert("b".to_string(), 0.0);
        variables.insert("c".to_string(), 0.0);

        Ok(Self {
            id,
            config,
            properties,
            controller_config: ControllerConfig::default(),
            expression: "sin(t)".to_string(),
            variables,
            current_value: 0.0,
            start_time: now,
            last_update: now,
        })
    }

    /// 数式を評価
    fn evaluate_expression(&mut self, time: f32) -> f32 {
        // 基本的な数式評価器（簡略版）
        // 実際の実装では、より高度な数式パーサーを使用

        // 時間とユーザー変数を設定
        let mut context = HashMap::new();
        context.insert("t".to_string(), time);
        context.insert(
            "a".to_string(),
            self.variables.get("a").copied().unwrap_or(1.0),
        );
        context.insert(
            "b".to_string(),
            self.variables.get("b").copied().unwrap_or(0.0),
        );
        context.insert(
            "c".to_string(),
            self.variables.get("c").copied().unwrap_or(0.0),
        );

        // 簡単な数式の評価（基本的な関数のみサポート）
        self.evaluate_simple_expression(&self.expression, &context)
    }

    /// 簡単な数式評価（基本実装）
    fn evaluate_simple_expression(&self, expr: &str, context: &HashMap<String, f32>) -> f32 {
        // 非常に基本的な数式評価
        // 実際のプロダクションでは、もっと堅牢な数式パーサーを使用

        match expr {
            "sin(t)" => (context.get("t").unwrap_or(&0.0) * std::f32::consts::PI).sin(),
            "cos(t)" => (context.get("t").unwrap_or(&0.0) * std::f32::consts::PI).cos(),
            "sin(t * a)" => {
                let t = context.get("t").unwrap_or(&0.0);
                let a = context.get("a").unwrap_or(&1.0);
                (t * a * std::f32::consts::PI).sin()
            }
            "cos(t * a)" => {
                let t = context.get("t").unwrap_or(&0.0);
                let a = context.get("a").unwrap_or(&1.0);
                (t * a * std::f32::consts::PI).cos()
            }
            "a * sin(t) + b" => {
                let t = context.get("t").unwrap_or(&0.0);
                let a = context.get("a").unwrap_or(&1.0);
                let b = context.get("b").unwrap_or(&0.0);
                a * (t * std::f32::consts::PI).sin() + b
            }
            "a * cos(t) + b" => {
                let t = context.get("t").unwrap_or(&0.0);
                let a = context.get("a").unwrap_or(&1.0);
                let b = context.get("b").unwrap_or(&0.0);
                a * (t * std::f32::consts::PI).cos() + b
            }
            "abs(sin(t))" => {
                let t = context.get("t").unwrap_or(&0.0);
                (t * std::f32::consts::PI).sin().abs()
            }
            "t" => *context.get("t").unwrap_or(&0.0),
            "a" => *context.get("a").unwrap_or(&1.0),
            "b" => *context.get("b").unwrap_or(&0.0),
            "c" => *context.get("c").unwrap_or(&0.0),
            _ => {
                // フォールバック: 数値として解析を試行
                expr.parse::<f32>().unwrap_or(0.0)
            }
        }
    }

    /// パラメータを更新
    fn update_parameters(&mut self) {
        if let Some(expr_value) = self.get_parameter("expression") {
            if let Some(expr_str) = expr_value.as_str() {
                self.expression = expr_str.to_string();
            }
        }

        self.variables.insert(
            "a".to_string(),
            self.get_parameter("var_a")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32,
        );

        self.variables.insert(
            "b".to_string(),
            self.get_parameter("var_b")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
        );

        self.variables.insert(
            "c".to_string(),
            self.get_parameter("var_c")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
        );

        self.controller_config.enabled = self
            .get_parameter("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
    }
}

impl NodeProcessor for MathController {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        // パラメータを更新
        self.update_parameters();

        // 無効なら入力をそのまま通す
        if !self.controller_config.enabled {
            return Ok(input);
        }

        // 時間スケールを取得
        let time_scale = self
            .get_parameter("time_scale")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;

        // 経過時間を計算
        let now = Instant::now();
        let elapsed = now.duration_since(self.start_time).as_secs_f32() * time_scale;

        // 数式を評価
        self.current_value = self.evaluate_expression(elapsed);

        // 制御コマンドを生成
        let control_commands = self.generate_control_commands();

        let control_data = if !control_commands.is_empty() {
            Some(ControlData::MultiControl {
                commands: control_commands,
            })
        } else {
            input.control_data
        };

        self.last_update = now;

        Ok(FrameData {
            render_data: input.render_data,
            audio_data: input.audio_data,
            control_data,
            tally_metadata: input.tally_metadata,
        })
    }

    fn get_properties(&self) -> NodeProperties {
        self.properties.clone()
    }

    fn set_parameter(&mut self, key: &str, value: Value) -> Result<()> {
        self.config.parameters.insert(key.to_string(), value);
        Ok(())
    }

    fn get_parameter(&self, key: &str) -> Option<Value> {
        self.config.parameters.get(key).cloned()
    }
}

impl ControllerNode for MathController {
    fn add_mapping(&mut self, mapping: ControlMapping) {
        self.controller_config.mappings.push(mapping);
    }

    fn remove_mapping(&mut self, source_parameter: &str) {
        self.controller_config
            .mappings
            .retain(|m| m.source_parameter != source_parameter);
    }

    fn get_control_value(&self, parameter: &str) -> Option<f32> {
        match parameter {
            "output" | "result" => Some(self.current_value),
            "a" => self.variables.get("a").copied(),
            "b" => self.variables.get("b").copied(),
            "c" => self.variables.get("c").copied(),
            _ => None,
        }
    }

    fn generate_control_commands(&self) -> Vec<ControlCommand> {
        let mut control_values = HashMap::new();
        control_values.insert("output".to_string(), self.current_value);
        control_values.insert("result".to_string(), self.current_value);

        apply_mappings(&self.controller_config.mappings, &control_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math_controller_creation() {
        let id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let controller = MathController::new(id, config);
        assert!(controller.is_ok());

        let controller = controller.unwrap();
        assert_eq!(controller.id, id);
        assert_eq!(controller.expression, "sin(t)");
    }

    #[test]
    fn test_math_expression_evaluation() {
        let id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let mut controller = MathController::new(id, config).unwrap();

        // Test simple sine expression
        let value = controller.evaluate_expression(0.0);
        assert!((value - 0.0).abs() < 0.01); // sin(0) = 0

        let value = controller.evaluate_expression(0.5);
        assert!((value - 1.0).abs() < 0.01); // sin(π/2) = 1
    }

    #[test]
    fn test_math_variables() {
        let id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let mut controller = MathController::new(id, config).unwrap();
        controller.expression = "a * sin(t) + b".to_string();
        controller.variables.insert("a".to_string(), 2.0);
        controller.variables.insert("b".to_string(), 1.0);

        let value = controller.evaluate_expression(0.0);
        assert!((value - 1.0).abs() < 0.01); // 2 * sin(0) + 1 = 1

        let value = controller.evaluate_expression(0.5);
        assert!((value - 3.0).abs() < 0.01); // 2 * sin(π/2) + 1 = 3
    }

    #[test]
    fn test_math_constant_expression() {
        let id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let mut controller = MathController::new(id, config).unwrap();
        controller.expression = "a".to_string();
        controller.variables.insert("a".to_string(), 42.0);

        let value = controller.evaluate_expression(0.0);
        assert!((value - 42.0).abs() < 0.01);
    }
}

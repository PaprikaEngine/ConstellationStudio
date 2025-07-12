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
use constellation_core::*;
use std::collections::HashMap;
use uuid::Uuid;

pub struct AudioProcessor {
    sample_rate: u32,
    channels: u16,
}

impl AudioProcessor {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate,
            channels,
        }
    }

    pub fn process_audio(&mut self, input: &AudioFrame) -> Result<AudioFrame> {
        Ok(AudioFrame {
            sample_rate: input.sample_rate,
            channels: input.channels,
            samples: input.samples.clone(),
        })
    }

    pub fn mix_audio(&self, inputs: &[AudioFrame]) -> Result<AudioFrame> {
        if inputs.is_empty() {
            return Ok(AudioFrame {
                sample_rate: self.sample_rate,
                channels: self.channels,
                samples: vec![],
            });
        }

        let first_frame = &inputs[0];
        let mut mixed_samples = first_frame.samples.clone();

        for input in inputs.iter().skip(1) {
            for (i, &sample) in input.samples.iter().enumerate() {
                if i < mixed_samples.len() {
                    mixed_samples[i] += sample;
                }
            }
        }

        let num_inputs = inputs.len() as f32;
        for sample in &mut mixed_samples {
            *sample /= num_inputs;
        }

        Ok(AudioFrame {
            sample_rate: first_frame.sample_rate,
            channels: first_frame.channels,
            samples: mixed_samples,
        })
    }
}

/// Real-time audio level analyzer for live monitoring
pub struct AudioLevelAnalyzer {
    /// Track levels per node
    node_levels: HashMap<Uuid, AudioLevel>,
    /// Update interval in milliseconds
    update_interval_ms: u64,
    /// Last update timestamp per node
    last_update: HashMap<Uuid, u64>,
}

impl Default for AudioLevelAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioLevelAnalyzer {
    pub fn new() -> Self {
        Self {
            node_levels: HashMap::new(),
            update_interval_ms: 16, // ~60fps update rate
            last_update: HashMap::new(),
        }
    }

    /// Set update interval for level analysis
    pub fn set_update_interval(&mut self, interval_ms: u64) {
        self.update_interval_ms = interval_ms;
    }

    /// Analyze audio frame and return current levels
    pub fn analyze_frame(
        &mut self,
        node_id: Uuid,
        audio_data: &UnifiedAudioData,
    ) -> Option<AudioLevel> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Check if we should update (throttle updates)
        if let Some(&last_time) = self.last_update.get(&node_id) {
            if now - last_time < self.update_interval_ms {
                // Return cached level if update too frequent
                return self.node_levels.get(&node_id).cloned();
            }
        }

        // Calculate new level
        let level = AudioLevel::from_audio_data(audio_data);

        // Store level and update timestamp
        self.node_levels.insert(node_id, level.clone());
        self.last_update.insert(node_id, now);

        Some(level)
    }

    /// Get current audio level for a node (cached)
    pub fn get_current_level(&self, node_id: &Uuid) -> Option<&AudioLevel> {
        self.node_levels.get(node_id)
    }

    /// Get all current levels
    pub fn get_all_levels(&self) -> &HashMap<Uuid, AudioLevel> {
        &self.node_levels
    }

    /// Clear level data for a node (when node is removed)
    pub fn clear_node(&mut self, node_id: &Uuid) {
        self.node_levels.remove(node_id);
        self.last_update.remove(node_id);
    }

    /// Clear all level data
    pub fn clear_all(&mut self) {
        self.node_levels.clear();
        self.last_update.clear();
    }

    /// Check if any node is currently clipping
    pub fn has_clipping(&self) -> bool {
        self.node_levels.values().any(|level| level.is_clipping)
    }

    /// Get nodes that are currently clipping
    pub fn get_clipping_nodes(&self) -> Vec<Uuid> {
        self.node_levels
            .iter()
            .filter(|(_, level)| level.is_clipping)
            .map(|(node_id, _)| *node_id)
            .collect()
    }

    /// Calculate overall peak level across all nodes
    pub fn get_overall_peak(&self) -> f32 {
        self.node_levels
            .values()
            .map(|level| level.mono_peak())
            .fold(0.0, f32::max)
    }

    /// Calculate overall RMS level across all nodes  
    pub fn get_overall_rms(&self) -> f32 {
        if self.node_levels.is_empty() {
            return 0.0;
        }

        let sum_rms_squared: f32 = self
            .node_levels
            .values()
            .map(|level| {
                let rms = level.mono_rms();
                rms * rms
            })
            .sum();

        (sum_rms_squared / self.node_levels.len() as f32).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_processor() {
        let mut processor = AudioProcessor::new(48000, 2);

        let input_frame = AudioFrame {
            sample_rate: 48000,
            channels: 2,
            samples: vec![0.5, -0.3, 0.8, -0.1],
        };

        let result = processor.process_audio(&input_frame);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.sample_rate, 48000);
        assert_eq!(output.channels, 2);
        assert_eq!(output.samples.len(), 4);
    }

    #[test]
    fn test_audio_mixing() {
        let processor = AudioProcessor::new(48000, 2);

        let frame1 = AudioFrame {
            sample_rate: 48000,
            channels: 2,
            samples: vec![0.5, 0.5],
        };

        let frame2 = AudioFrame {
            sample_rate: 48000,
            channels: 2,
            samples: vec![0.3, 0.3],
        };

        let inputs = vec![frame1, frame2];
        let result = processor.mix_audio(&inputs);
        assert!(result.is_ok());

        let mixed = result.unwrap();
        assert_eq!(mixed.samples[0], 0.4); // (0.5 + 0.3) / 2
        assert_eq!(mixed.samples[1], 0.4); // (0.5 + 0.3) / 2
    }

    #[test]
    fn test_audio_level_analyzer() {
        let mut analyzer = AudioLevelAnalyzer::new();
        let node_id = Uuid::new_v4();

        // Test with normal audio data
        let audio_data = UnifiedAudioData::Stereo {
            sample_rate: 48000,
            channels: 2,
            samples: vec![0.5, -0.3, 0.8, -0.1], // Left: 0.5, 0.8; Right: -0.3, -0.1
        };

        let level = analyzer.analyze_frame(node_id, &audio_data);
        assert!(level.is_some());

        let level = level.unwrap();
        assert!(!level.is_clipping);
        assert!(level.peak_left > 0.0);
        assert!(level.peak_right > 0.0);
        assert!(level.rms_left > 0.0);
        assert!(level.rms_right > 0.0);

        // Test caching behavior
        let cached_level = analyzer.get_current_level(&node_id);
        assert!(cached_level.is_some());
        assert_eq!(cached_level.unwrap(), &level);
    }

    #[test]
    fn test_audio_level_clipping_detection() {
        let mut analyzer = AudioLevelAnalyzer::new();
        let node_id = Uuid::new_v4();

        // Test with clipping audio data
        let clipping_data = UnifiedAudioData::Stereo {
            sample_rate: 48000,
            channels: 2,
            samples: vec![1.2, -1.1, 0.8, -0.9], // Values > 1.0 should trigger clipping
        };

        let level = analyzer.analyze_frame(node_id, &clipping_data);
        assert!(level.is_some());

        let level = level.unwrap();
        assert!(level.is_clipping);
        assert!(analyzer.has_clipping());

        let clipping_nodes = analyzer.get_clipping_nodes();
        assert!(clipping_nodes.contains(&node_id));
    }

    #[test]
    fn test_audio_level_analyzer_multiple_nodes() {
        let mut analyzer = AudioLevelAnalyzer::new();
        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();

        let audio1 = UnifiedAudioData::Stereo {
            sample_rate: 48000,
            channels: 2,
            samples: vec![0.5, -0.5],
        };

        let audio2 = UnifiedAudioData::Stereo {
            sample_rate: 48000,
            channels: 2,
            samples: vec![0.3, -0.3],
        };

        analyzer.analyze_frame(node1, &audio1);
        analyzer.analyze_frame(node2, &audio2);

        let all_levels = analyzer.get_all_levels();
        assert_eq!(all_levels.len(), 2);
        assert!(all_levels.contains_key(&node1));
        assert!(all_levels.contains_key(&node2));

        let overall_peak = analyzer.get_overall_peak();
        assert!(overall_peak > 0.0);

        let overall_rms = analyzer.get_overall_rms();
        assert!(overall_rms > 0.0);
    }
}

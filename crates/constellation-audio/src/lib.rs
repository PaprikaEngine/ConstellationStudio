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
}

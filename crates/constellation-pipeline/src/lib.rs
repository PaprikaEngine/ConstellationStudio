use anyhow::Result;
use constellation_core::*;
use constellation_nodes::*;
use std::collections::HashMap;
use uuid::Uuid;

pub struct PipelineProcessor {
    nodes: HashMap<Uuid, Box<dyn NodeProcessor + Send>>,
    execution_order: Vec<Uuid>,
}

impl Default for PipelineProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineProcessor {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            execution_order: Vec::new(),
        }
    }

    pub fn add_node(&mut self, id: Uuid, processor: Box<dyn NodeProcessor + Send>) {
        self.nodes.insert(id, processor);
        self.rebuild_execution_order();
    }

    pub fn remove_node(&mut self, id: &Uuid) {
        self.nodes.remove(id);
        self.execution_order.retain(|&node_id| node_id != *id);
    }

    pub fn process_frame(&mut self, input: FrameData) -> Result<FrameData> {
        let mut current_frame = input;

        for &node_id in &self.execution_order {
            if let Some(processor) = self.nodes.get_mut(&node_id) {
                // Tally伝播処理
                if processor.should_propagate_tally(&current_frame.tally_metadata) {
                    let processed_tally = processor.process_tally_metadata(&current_frame.tally_metadata);
                    current_frame.tally_metadata.merge_with(&processed_tally);
                }
                
                // メインフレーム処理
                current_frame = processor.process(current_frame)?;
                
                // ノード固有のTally状態を生成・追加
                let node_tally = processor.generate_tally_state();
                current_frame.tally_metadata.merge_with(&node_tally);
            }
        }

        Ok(current_frame)
    }

    fn rebuild_execution_order(&mut self) {
        self.execution_order = self.nodes.keys().copied().collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_pipeline_processor() {
        let mut pipeline = PipelineProcessor::new();

        let node_id = Uuid::new_v4();
        let processor = create_node_processor(
            NodeType::Input(InputType::TestPattern),
            node_id,
            NodeConfig {
                parameters: HashMap::new(),
            },
        )
        .unwrap();

        pipeline.add_node(node_id, processor);

        let input_frame = FrameData {
            render_data: None,
            audio_data: None,
            control_data: None,
            tally_metadata: TallyMetadata::new(),
        };

        let result = pipeline.process_frame(input_frame);
        assert!(result.is_ok());
    }
}

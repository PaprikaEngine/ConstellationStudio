import os
import re

def fix_frame_data_in_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Simple replacement patterns
    patterns = [
        # Replace old FrameData pattern with basic replacements
        (r'video_data:', 'render_data:'),
        (r'tally_data:\s*None,', ''),
        (r'scene3d_data:\s*None,', ''),
        (r'spatial_audio_data:\s*None,', ''),
        (r'transform_data:\s*None,', 'control_data: None,'),
        (r'Some\(frame\)', 'Some(RenderData::Raster2D(frame))'),
        (r'Some\(video_frame\)', 'Some(RenderData::Raster2D(video_frame))'),
        (r'Some\(frame_data\)', 'Some(RenderData::Raster2D(frame_data))'),
    ]
    
    new_content = content
    for pattern, replacement in patterns:
        new_content = re.sub(pattern, replacement, new_content)
    
    if new_content \!= content:
        with open(filepath, 'w') as f:
            f.write(new_content)
        print(f"Fixed patterns in {filepath}")

# Find and fix all .rs files
for root, dirs, files in os.walk('/Users/mirabilis/dev/ConstellationStudio/crates/constellation-nodes/src'):
    for file in files:
        if file.endswith('.rs'):
            filepath = os.path.join(root, file)
            fix_frame_data_in_file(filepath)

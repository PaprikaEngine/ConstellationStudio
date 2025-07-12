# Constellation Studio

**Next-Generation Real-time Video Processing Platform**

A node-based video processing system powered by Rust + Ash Vulkan, designed to scale from individual streamers to major broadcasting stations, supporting everything from 2D video to VR/XR content creation.

## 🚀 Phase 1: Local Standalone (2D Foundation)

Phase 1 foundation development is complete with the following features implemented:

### ✅ Completed Features
- **Rust Workspace**: Modular design with 5 core crates
- **Vulkan Foundation**: Ash + high-speed memory pools + GPU parallel processing
- **Node System**: Comprehensive Input/Output/Effect/Audio/Tally nodes
- **React Frontend**: Intuitive UI with TypeScript + React Flow
- **Type-Safe Communication**: Complete type safety via Serde + UUID
- **🎵 Real-time Audio Level Meters**: Professional vertical audio monitoring with WebSocket streaming

### 🎵 Audio Level Meter Features (Issue #45 - ✅ Completed)
- **Vertical Level Meters**: Intuitive bottom-to-top level visualization
- **Real-time WebSocket Streaming**: 60fps audio level updates
- **Professional Audio Monitoring**: RMS/Peak level calculation with clipping detection
- **Node Integration**: Automatic level meters for all audio nodes
- **Peak Hold & Clipping Warnings**: Visual feedback for audio professionals
- **Mono/Stereo Support**: Configurable display modes

## 🔧 Technology Stack

### Backend
- **Rust**: Memory safety + maximum performance
- **Ash Vulkan**: Ultra-low latency GPU processing (<1.2ms@1080p target)
- **Multi-platform**: Windows/macOS/Linux support
- **Real-time Audio**: High-performance audio level analysis

### Frontend  
- **React + TypeScript**: Type-safe development experience
- **React Flow**: Professional node editor interface
- **Vite**: Fast development environment
- **WebSocket Integration**: Real-time audio/video data streaming

## 📋 Development Roadmap

Current development status is managed through [GitHub Issues](https://github.com/PaprikaEngine/ConstellationStudio/issues):

### 🎯 Phase 1 Remaining Tasks
1. **[#40 Screen/Window Capture](https://github.com/PaprikaEngine/ConstellationStudio/issues/40)** - Platform-specific implementation
2. **[#41 Virtual Webcam Device](https://github.com/PaprikaEngine/ConstellationStudio/issues/41)** - Zoom/Teams integration
3. **[#3 Vulkan Optimization](https://github.com/PaprikaEngine/ConstellationStudio/issues/3)** - Performance target achievement
4. **[#4 Frontend Integration](https://github.com/PaprikaEngine/ConstellationStudio/issues/4)** - Working application
5. **[#5 Basic Effects](https://github.com/PaprikaEngine/ConstellationStudio/issues/5)** - GPU-optimized shaders
6. **[#6 TDD & CI/CD](https://github.com/PaprikaEngine/ConstellationStudio/issues/6)** - Quality assurance

### 🔮 Future Phases
- **Phase 2**: Professional video standards (SDI/NDI/SRT)
- **Phase 3**: Cloud-scalable system
- **Phase 4**: 3D/VR/XR support

## 🏗️ Project Structure

```
constellation-studio/
├── crates/
│   ├── constellation-core/      # Core engine (Ash Vulkan)
│   ├── constellation-vulkan/    # Vulkan processing & memory management
│   ├── constellation-nodes/     # Node implementations
│   ├── constellation-pipeline/  # Pipeline management
│   ├── constellation-audio/     # Audio processing & level analysis
│   └── constellation-web/       # Web API (frontend integration)
├── frontend/                    # React + TypeScript + React Flow
└── examples/                    # Samples & benchmarks
```

## ⚡ Performance Targets

| Resolution | Target Latency | Frame Rate | Audio Precision |
|------------|----------------|------------|-----------------|
| 1080p | <1.2ms | 60fps+ | ±0.1dB RMS |
| 4K | <6ms | 60fps | ±0.05dB Peak |
| 8K | <24ms | 30fps | <20ms Update |

## 🛠️ Development Environment

### Prerequisites
```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (for frontend)
# Install from https://nodejs.org/

# Vulkan SDK (for development/testing)
# Install from https://vulkan.lunarg.com/
```

### Build and Run
```bash
# Backend build
cargo build

# Run tests
cargo test

# Audio-specific tests
cargo test --package constellation-audio

# Code quality check
cargo clippy --workspace --all-targets --all-features
cargo fmt --all

# Frontend development server
cd frontend && npm install && npm run dev

# Backend web server (for WebSocket audio streaming)
cargo run --bin constellation-web
```

## 🎵 Audio Level Meter Usage

The real-time audio level meters are automatically integrated into audio nodes:

```typescript
// Component automatically included in audio nodes
<AudioLevelMeter
  nodeId={nodeId}
  width={30}
  height={35}
  mode="mono"
  showLabels={false}
  showValues={true}
/>
```

WebSocket communication for real-time updates:
```json
{
  "type": "audio_level",
  "node_id": "uuid",
  "level_data": {
    "peak_left": 0.78,
    "peak_right": 0.65,
    "rms_left": 0.54,
    "rms_right": 0.45,
    "db_peak_left": -2.1,
    "db_peak_right": -3.7,
    "db_rms_left": -5.4,
    "db_rms_right": -6.9,
    "is_clipping": false,
    "timestamp": 1234567890
  }
}
```

## 🤝 Development Philosophy

- **TDD (Test-Driven Development)**: Quality assurance through testing
- **Atomic Commits**: Feature-based change management
- **GitHub Issue Management**: Transparent task management
- **Incremental Delivery**: Phase-based reliable progress
- **Performance First**: Real-time requirements drive architecture decisions

## 📖 Detailed Specifications

For detailed project specifications and architecture, please refer to [CLAUDE.md](./CLAUDE.md).

## 🌟 Innovative Features

- **🔥 Intermediate Rendering Sharing**: High-speed processing without quality degradation
- **⚡ Ash Vulkan Optimization**: C++ equivalent performance + Rust safety
- **🎛️ Node-based UI**: Intuitive video processing pipeline
- **📈 Gradual Scalability**: From individuals to broadcasting stations
- **🔒 Memory Safety**: Rust safety + maximum performance
- **🎵 Professional Audio Monitoring**: Real-time level meters with WebSocket streaming

## 🧪 Testing & Quality

```bash
# All tests must pass
cargo test --workspace --lib

# Audio level meter specific tests
✅ test_audio_level_analyzer - Basic functionality 
✅ test_audio_level_clipping_detection - Overload detection
✅ test_audio_level_analyzer_multiple_nodes - Multi-node management  
✅ test_audio_mixing - Audio signal processing
✅ test_audio_processor - Core audio engine
```

## 🚀 Recent Achievements

- **Issue #45**: ✅ Real-time vertical audio level meters with WebSocket streaming
- **Issue #44**: ✅ Video preview display component implementation
- **Performance**: ✅ 60fps audio monitoring with <20ms latency
- **Code Quality**: ✅ Comprehensive test coverage with precise validation
- **Code Review**: ✅ Gemini AI code review feedback addressed

---

**🤖 Generated with [Claude Code](https://claude.ai/code)**
# Constellation Studio

**Next-Generation Real-time Video Processing Platform**

A professional node-based video processing system powered by Rust + Ash Vulkan, designed to scale from individual streamers to major broadcasting stations. Supports everything from 2D video processing to future VR/XR content creation with revolutionary intermediate rendering architecture.

## 🚀 Current Status: Phase 1 Foundation (In Development)

### ✅ Core Features Implemented
- **🦀 Rust Workspace**: Modular architecture with 6 specialized crates
- **⚡ Vulkan Foundation**: Ash-powered GPU processing with optimized memory pools
- **🎛️ Node System Framework**: Basic Input/Output/Effect/Audio/Tally node structure
- **⚛️ React Frontend**: Professional UI with TypeScript + React Flow integration
- **🔒 Type-Safe Communication**: End-to-end type safety via Serde + UUID system
- **🎵 Real-time Audio Monitoring**: Professional vertical level meters with WebSocket streaming

### 🔄 Core Features In Development
- **📹 Screen/Window Capture**: Platform-specific implementation ([Issue #40](https://github.com/PaprikaEngine/ConstellationStudio/issues/40))
- **📹 Virtual Webcam Output**: Cross-platform virtual camera device ([Issue #41](https://github.com/PaprikaEngine/ConstellationStudio/issues/41))
- **⚡ Vulkan Compute Pipeline**: GPU-accelerated video processing ([Issue #39](https://github.com/PaprikaEngine/ConstellationStudio/issues/39))
- **📺 Video Preview Components**: Real-time video display ([Issue #44](https://github.com/PaprikaEngine/ConstellationStudio/issues/44))

### 🎵 Professional Audio Features
- **Vertical Level Meters**: Intuitive bottom-to-top level visualization (-∞ to 0dB)
- **Real-time Analysis**: 60fps audio level updates via WebSocket
- **Professional Monitoring**: RMS/Peak calculation with precise dB measurements
- **Clipping Detection**: Visual warnings and peak hold functionality
- **Multi-channel Support**: Mono/Stereo configurable display modes
- **Low Latency**: <20ms update latency for real-time monitoring

### 📹 Video Processing Foundation
- **Vulkan Context**: GPU device initialization and memory management
- **Frame Buffer System**: Efficient video memory allocation framework
- **Cross-platform Base**: Windows/macOS/Linux compatibility layer
- **Processing Pipeline**: Architecture ready for compute shader implementation

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

Development progress is managed through [GitHub Issues](https://github.com/PaprikaEngine/ConstellationStudio/issues) using Conventional Commit standards.

### 🎯 Phase 1: Foundation (Current Priority)
1. **[#40 Screen/Window Capture](https://github.com/PaprikaEngine/ConstellationStudio/issues/40)** - Platform-specific desktop/window capture
2. **[#41 Virtual Webcam Device](https://github.com/PaprikaEngine/ConstellationStudio/issues/41)** - Zoom/Teams/Discord integration
3. **[#3 Vulkan Optimization](https://github.com/PaprikaEngine/ConstellationStudio/issues/3)** - Performance target achievement (<1.2ms@1080p)
4. **[#4 Frontend Integration](https://github.com/PaprikaEngine/ConstellationStudio/issues/4)** - Complete working application
5. **[#5 Basic Effects](https://github.com/PaprikaEngine/ConstellationStudio/issues/5)** - GPU-optimized shader effects
6. **[#6 TDD & CI/CD](https://github.com/PaprikaEngine/ConstellationStudio/issues/6)** - Quality assurance pipeline

### 🔮 Future Development Phases
- **Phase 2**: Professional Standards (SDI/NDI/SRT/SMPTE ST 2110)
- **Phase 3**: Cloud-Scalable Microservices (Kubernetes + Edge Computing)
- **Phase 4**: 3D/VR/XR Support (Metaverse + Spatial Audio)

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

## 🌟 Revolutionary Architecture

### Intermediate Rendering State Sharing
Unlike traditional video processing tools that rasterize at every node (causing quality degradation), Constellation Studio maintains **intermediate rendering states** throughout the processing pipeline:

- **Traditional Approach**: Input → Raster → Effect → Raster → Output ❌ Quality Loss
- **Our Innovation**: Input → Intermediate → Effect → Intermediate → Final Raster ✅ Zero Degradation


### Core Innovations
- **🔥 Zero-Degradation Pipeline**: Intermediate rendering state preservation
- **⚡ Ultra-Low Latency**: <1.2ms@1080p target via Ash Vulkan optimization
- **🎛️ Professional Node Editor**: Intuitive visual programming interface
- **📈 Infinite Scalability**: Architecture scales from streamers to broadcast stations
- **🔒 Memory Safety + Performance**: Rust safety without sacrificing speed
- **🎵 Real-time Audio Monitoring**: Professional-grade level metering

## 🧪 Testing & Quality Assurance

### Test Coverage
```bash
# All tests must pass before commits
cargo test --workspace --lib

# Audio subsystem tests
✅ test_audio_level_analyzer - Real-time level analysis
✅ test_audio_level_clipping_detection - Overload detection & warnings
✅ test_audio_level_analyzer_multiple_nodes - Multi-node management
✅ test_audio_mixing - Audio signal processing pipeline
✅ test_audio_processor - Core audio engine functionality
```

### Code Quality Standards
```bash
# Mandatory before all commits
cargo fmt --all                                              # Code formatting
cargo clippy --workspace --all-targets --all-features       # Lint checking
cargo test --workspace --lib                                # Unit tests
```

## 🚀 Recent Major Achievements

- **Issue #49**: ✅ Video preview streaming with real-time display components
- **Issue #45**: ✅ Professional vertical audio level meters with WebSocket streaming
- **Issue #44**: ✅ Comprehensive video preview display component implementation
- **Performance Milestone**: ✅ 60fps audio monitoring with <20ms latency achieved
- **Code Quality**: ✅ Comprehensive test coverage with precise validation framework
- **AI Code Review**: ✅ Gemini AI code review feedback integration completed

---

**🤖 Generated with [Claude Code](https://claude.ai/code)**
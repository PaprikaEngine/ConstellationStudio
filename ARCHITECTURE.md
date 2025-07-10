# Constellation Studio Architecture

## Crates Structure

### Core Crates

#### `constellation-core`
- **Purpose**: Core engine and data structures
- **Key Components**:
  - `ConstellationEngine`: Main engine coordinator
  - `FrameData`: Unified frame data structure
  - `NodeGraph`: Node system management
  - Data structures for Video, Audio, Control, and Tally
- **Dependencies**: `constellation-vulkan`, `constellation-3d` (optional)
- **Features**: 
  - `phase-4`: Enables 3D/VR/XR functionality

#### `constellation-vulkan`
- **Purpose**: Vulkan GPU processing and memory management
- **Key Components**:
  - `VulkanContext`: Vulkan initialization and device management
  - `MemoryManager`: GPU memory pool management
  - `FrameBuffer`: GPU frame buffer management
- **Dependencies**: `ash`, platform-specific libraries
- **Platform Support**: Windows, macOS, Linux

#### `constellation-nodes`
- **Purpose**: Node implementations (Input, Output, Effect, etc.)
- **Key Components**:
  - Camera input nodes
  - Screen/Window capture nodes  
  - Virtual webcam output nodes
  - Effect processing nodes
- **Dependencies**: `constellation-core`, `constellation-vulkan`
- **Features**: 
  - `test-capture-backends`: Test mode for capture backends

#### `constellation-pipeline`
- **Purpose**: Pipeline execution and processing coordination
- **Key Components**:
  - Pipeline execution engine
  - Frame processing orchestration
  - Real-time processing coordination
- **Dependencies**: `constellation-core`, `constellation-vulkan`, `constellation-nodes`

#### `constellation-audio`
- **Purpose**: Audio processing and spatial audio
- **Key Components**:
  - Audio input/output management
  - Audio effect processing
  - Spatial audio processing (Phase 4)
- **Dependencies**: `constellation-core`

#### `constellation-web`
- **Purpose**: Web API and frontend communication
- **Key Components**:
  - REST API server
  - WebSocket real-time communication
  - Frontend-backend bridge
- **Dependencies**: `constellation-core`, `constellation-nodes`
- **Binary**: `constellation-server`

#### `constellation-3d` (Phase 4)
- **Purpose**: 3D/VR/XR processing
- **Key Components**:
  - 3D scene processing
  - VR device integration
  - XR rendering pipeline
- **Dependencies**: `constellation-core`, `constellation-vulkan`
- **Features**: 
  - `phase-4`: Main 3D functionality

## Module Interfaces

### Core Engine Interface
```rust
pub struct ConstellationEngine {
    // Core processing coordination
    pub fn new() -> Result<Self>;
    pub fn process_frame(&mut self, input: &FrameData) -> Result<FrameData>;
    pub fn add_node(&mut self, node_type: NodeType, config: NodeConfig) -> Result<Uuid>;
    pub fn connect_nodes(&mut self, source: Uuid, target: Uuid, conn_type: ConnectionType) -> Result<()>;
}
```

### Vulkan Context Interface
```rust
pub struct VulkanContext {
    // GPU resource management
    pub fn new() -> Result<Self>;
    // Device, queue, and command pool access
}

pub struct MemoryManager {
    // GPU memory pool management
    pub fn new(context: &VulkanContext) -> Result<Self>;
    pub fn allocate_frame_buffer(&mut self, size: u64, memory_type: u32) -> Result<FrameBuffer>;
    pub fn free_frame_buffer(&mut self, buffer: FrameBuffer);
}
```

### Node System Interface
```rust
pub trait Node {
    fn process(&mut self, input: FrameData) -> Result<FrameData>;
    fn get_parameters(&self) -> &NodeConfig;
    fn set_parameter(&mut self, name: &str, value: serde_json::Value) -> Result<()>;
}
```

### Pipeline Interface
```rust
pub struct PipelineExecutor {
    // Pipeline execution coordination
    pub fn new(context: VulkanContext) -> Result<Self>;
    pub fn execute_graph(&mut self, graph: &NodeGraph) -> Result<()>;
}
```

## Data Flow

### Frame Processing Flow
1. **Input Nodes**: Capture/generate source data
2. **Processing Nodes**: Apply effects and transformations
3. **Output Nodes**: Send to virtual webcam or other outputs

### Memory Management Flow
1. **Allocation**: GPU memory pools managed by `MemoryManager`
2. **Processing**: Frame data processed in GPU memory
3. **Deallocation**: Memory returned to pools for reuse

### Node Communication
- **RenderData**: Main video/3D processing chain
- **Audio**: Audio processing chain
- **Control**: Parameter control and automation
- **Tally**: Status and monitoring signals

## Build System

### Feature Flags
- `phase-4`: Enables 3D/VR/XR functionality
- `test-capture-backends`: Test mode for capture systems

### Platform Support
- **Windows**: DirectShow, Graphics Capture API
- **macOS**: Screen Capture Kit, Core Media I/O
- **Linux**: V4L2, X11/Wayland capture

### Dependencies
- **Vulkan**: `ash` for GPU processing
- **Async**: `tokio` for async operations
- **Serialization**: `serde` for data exchange
- **Web**: `axum` for REST API
- **3D Math**: `nalgebra`, `cgmath` for 3D processing

## Development Phases

### Phase 1: Core Implementation
- Basic Vulkan setup
- Node system foundation
- Screen/window capture
- Virtual webcam output

### Phase 2: Advanced Features
- Professional video I/O (SDI, NDI, SRT)
- Advanced frame control
- Performance optimization

### Phase 3: Cloud Scaling
- Microservices architecture
- Kubernetes deployment
- Global CDN integration

### Phase 4: 3D/VR/XR
- 3D scene processing
- VR device integration
- XR rendering pipeline
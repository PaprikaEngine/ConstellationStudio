use thiserror::Error;
use uuid::Uuid;

/// Constellation Studio の統一エラー型
#[derive(Error, Debug)]
pub enum ConstellationError {
    // === コアシステムエラー ===
    #[error("Engine initialization failed: {reason}")]
    EngineInitializationFailed { reason: String },

    #[error("Engine is not running")]
    EngineNotRunning,

    #[error("Engine is already running")]
    EngineAlreadyRunning,

    // === ノードシステムエラー ===
    #[error("Node not found: {node_id}")]
    NodeNotFound { node_id: Uuid },

    #[error("Invalid node type: {node_type}")]
    InvalidNodeType { node_type: String },

    #[error("Node creation failed: {reason}")]
    NodeCreationFailed { reason: String },

    #[error("Node processing failed: {node_id} - {reason}")]
    NodeProcessingFailed { node_id: Uuid, reason: String },

    #[error("Invalid connection: {source_id} -> {target_id} ({connection_type})")]
    InvalidConnection {
        source_id: Uuid,
        target_id: Uuid,
        connection_type: String,
    },

    #[error("Connection cycle detected: {path:?}")]
    ConnectionCycleDetected { path: Vec<Uuid> },

    // === フレーム処理エラー ===
    #[error("Frame processing failed: {reason}")]
    FrameProcessingFailed { reason: String },

    #[error("Invalid frame format: expected {expected}, got {actual}")]
    InvalidFrameFormat { expected: String, actual: String },

    #[error("Frame data corrupted: {details}")]
    FrameDataCorrupted { details: String },

    #[error("Frame processing timeout: {timeout_ms}ms")]
    FrameProcessingTimeout { timeout_ms: u64 },

    // === リソースエラー ===
    #[error("Insufficient memory: required {required_bytes} bytes")]
    InsufficientMemory { required_bytes: u64 },

    #[error("Resource allocation failed: {resource_type}")]
    ResourceAllocationFailed { resource_type: String },

    #[error("Resource limit exceeded: {resource} ({current}/{limit})")]
    ResourceLimitExceeded {
        resource: String,
        current: u64,
        limit: u64,
    },

    // === ハードウェア・ドライバーエラー ===
    #[error("Hardware not supported: {hardware}")]
    HardwareNotSupported { hardware: String },

    #[error("Driver incompatible: {driver} version {version}")]
    DriverIncompatible { driver: String, version: String },

    #[error("Device access failed: {device} - {reason}")]
    DeviceAccessFailed { device: String, reason: String },

    #[error("GPU processing failed: {reason}")]
    GpuProcessingFailed { reason: String },

    // === ネットワーク・通信エラー ===
    #[error("Network connection failed: {endpoint}")]
    NetworkConnectionFailed { endpoint: String },

    #[error("Data transmission failed: {reason}")]
    DataTransmissionFailed { reason: String },

    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    ProtocolVersionMismatch { expected: String, actual: String },

    // === ファイル・I/Oエラー ===
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("File format not supported: {format}")]
    FileFormatNotSupported { format: String },

    #[error("File read/write failed: {path} - {reason}")]
    FileIoFailed { path: String, reason: String },

    #[error("Insufficient disk space: required {required_bytes} bytes")]
    InsufficientDiskSpace { required_bytes: u64 },

    // === 設定・パラメータエラー ===
    #[error("Invalid parameter: {parameter} = {value}")]
    InvalidParameter { parameter: String, value: String },

    #[error("Parameter out of range: {parameter} = {value} (range: {min}-{max})")]
    ParameterOutOfRange {
        parameter: String,
        value: String,
        min: String,
        max: String,
    },

    #[error("Configuration error: {reason}")]
    ConfigurationError { reason: String },

    // === プラットフォーム固有エラー ===
    #[error("Platform not supported: {platform}")]
    PlatformNotSupported { platform: String },

    #[error("Platform-specific error ({platform}): {reason}")]
    PlatformSpecificError { platform: String, reason: String },

    // === ライセンス・権限エラー ===
    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },

    #[error("License validation failed: {reason}")]
    LicenseValidationFailed { reason: String },

    // === 包括的エラー ===
    #[error("Internal error: {reason}")]
    InternalError { reason: String },

    #[error("External library error: {library} - {reason}")]
    ExternalLibraryError { library: String, reason: String },

    #[error("Unknown error: {reason}")]
    Unknown { reason: String },
}

/// エラーの重要度レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ErrorSeverity {
    /// 情報レベル（動作に影響なし）
    Info,
    /// 警告レベル（軽微な問題）
    Warning,
    /// エラーレベル（機能に影響）
    Error,
    /// 致命的レベル（システム停止）
    Critical,
}

/// エラーカテゴリ
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ErrorCategory {
    System,
    Node,
    Frame,
    Resource,
    Hardware,
    Network,
    FileIo,
    Configuration,
    Platform,
    Security,
    Unknown,
}

impl ConstellationError {
    /// エラーの重要度を取得
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // 致命的エラー
            ConstellationError::EngineInitializationFailed { .. }
            | ConstellationError::InsufficientMemory { .. }
            | ConstellationError::HardwareNotSupported { .. }
            | ConstellationError::DriverIncompatible { .. } => ErrorSeverity::Critical,

            // 通常のエラー
            ConstellationError::NodeNotFound { .. }
            | ConstellationError::InvalidConnection { .. }
            | ConstellationError::FrameProcessingFailed { .. }
            | ConstellationError::DeviceAccessFailed { .. }
            | ConstellationError::FileNotFound { .. } => ErrorSeverity::Error,

            // 警告レベル
            ConstellationError::NodeProcessingFailed { .. }
            | ConstellationError::FrameProcessingTimeout { .. }
            | ConstellationError::ResourceLimitExceeded { .. } => ErrorSeverity::Warning,

            // その他は情報レベル
            _ => ErrorSeverity::Info,
        }
    }

    /// エラーのカテゴリを取得
    pub fn category(&self) -> ErrorCategory {
        match self {
            ConstellationError::EngineInitializationFailed { .. }
            | ConstellationError::EngineNotRunning
            | ConstellationError::EngineAlreadyRunning => ErrorCategory::System,

            ConstellationError::NodeNotFound { .. }
            | ConstellationError::InvalidNodeType { .. }
            | ConstellationError::NodeCreationFailed { .. }
            | ConstellationError::NodeProcessingFailed { .. }
            | ConstellationError::InvalidConnection { .. }
            | ConstellationError::ConnectionCycleDetected { .. } => ErrorCategory::Node,

            ConstellationError::FrameProcessingFailed { .. }
            | ConstellationError::InvalidFrameFormat { .. }
            | ConstellationError::FrameDataCorrupted { .. }
            | ConstellationError::FrameProcessingTimeout { .. } => ErrorCategory::Frame,

            ConstellationError::InsufficientMemory { .. }
            | ConstellationError::ResourceAllocationFailed { .. }
            | ConstellationError::ResourceLimitExceeded { .. } => ErrorCategory::Resource,

            ConstellationError::HardwareNotSupported { .. }
            | ConstellationError::DriverIncompatible { .. }
            | ConstellationError::DeviceAccessFailed { .. }
            | ConstellationError::GpuProcessingFailed { .. } => ErrorCategory::Hardware,

            ConstellationError::NetworkConnectionFailed { .. }
            | ConstellationError::DataTransmissionFailed { .. }
            | ConstellationError::ProtocolVersionMismatch { .. } => ErrorCategory::Network,

            ConstellationError::FileNotFound { .. }
            | ConstellationError::FileFormatNotSupported { .. }
            | ConstellationError::FileIoFailed { .. }
            | ConstellationError::InsufficientDiskSpace { .. } => ErrorCategory::FileIo,

            ConstellationError::InvalidParameter { .. }
            | ConstellationError::ParameterOutOfRange { .. }
            | ConstellationError::ConfigurationError { .. } => ErrorCategory::Configuration,

            ConstellationError::PlatformNotSupported { .. }
            | ConstellationError::PlatformSpecificError { .. } => ErrorCategory::Platform,

            ConstellationError::PermissionDenied { .. }
            | ConstellationError::LicenseValidationFailed { .. } => ErrorCategory::Security,

            ConstellationError::InternalError { .. }
            | ConstellationError::ExternalLibraryError { .. }
            | ConstellationError::Unknown { .. } => ErrorCategory::Unknown,
        }
    }

    /// ユーザーフレンドリなエラーメッセージを生成
    pub fn user_message(&self) -> String {
        match self {
            ConstellationError::EngineInitializationFailed { .. } => {
                "システムの初期化に失敗しました。ドライバーを確認してください。".to_string()
            }
            ConstellationError::NodeNotFound { .. } => {
                "指定されたノードが見つかりません。".to_string()
            }
            ConstellationError::InvalidConnection { .. } => {
                "ノードの接続が無効です。接続タイプを確認してください。".to_string()
            }
            ConstellationError::FrameProcessingFailed { .. } => {
                "映像処理中にエラーが発生しました。".to_string()
            }
            ConstellationError::InsufficientMemory { .. } => {
                "メモリが不足しています。解像度を下げるか他のアプリケーションを終了してください。"
                    .to_string()
            }
            ConstellationError::HardwareNotSupported { .. } => {
                "お使いのハードウェアはサポートされていません。".to_string()
            }
            ConstellationError::FileNotFound { path } => {
                format!("ファイルが見つかりません: {}", path)
            }
            ConstellationError::PermissionDenied { operation } => {
                format!(
                    "{}の権限がありません。アプリケーションに必要な権限を付与してください。",
                    operation
                )
            }
            _ => self.to_string(),
        }
    }

    /// 復旧可能かどうかを判定
    pub fn is_recoverable(&self) -> bool {
        match self {
            // 復旧不可能
            ConstellationError::EngineInitializationFailed { .. }
            | ConstellationError::HardwareNotSupported { .. }
            | ConstellationError::DriverIncompatible { .. }
            | ConstellationError::PlatformNotSupported { .. } => false,

            // 復旧可能
            ConstellationError::NodeProcessingFailed { .. }
            | ConstellationError::FrameProcessingTimeout { .. }
            | ConstellationError::NetworkConnectionFailed { .. }
            | ConstellationError::DeviceAccessFailed { .. } => true,

            // その他は条件次第
            _ => true,
        }
    }
}

/// Result型の型エイリアス
pub type ConstellationResult<T> = std::result::Result<T, ConstellationError>;

/// anyhow::Error からの変換
impl From<anyhow::Error> for ConstellationError {
    fn from(err: anyhow::Error) -> Self {
        ConstellationError::InternalError {
            reason: err.to_string(),
        }
    }
}

/// std::io::Error からの変換
impl From<std::io::Error> for ConstellationError {
    fn from(err: std::io::Error) -> Self {
        ConstellationError::FileIoFailed {
            path: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

/// constellation_vulkan::VulkanError からの変換
#[cfg(feature = "vulkan")]
impl From<constellation_vulkan::VulkanError> for ConstellationError {
    fn from(err: constellation_vulkan::VulkanError) -> Self {
        match err {
            constellation_vulkan::VulkanError::InitializationFailed { reason } => {
                ConstellationError::EngineInitializationFailed { reason }
            }
            constellation_vulkan::VulkanError::DeviceCreationFailed { reason } => {
                ConstellationError::EngineInitializationFailed { reason }
            }
            constellation_vulkan::VulkanError::HardwareNotSupported { hardware } => {
                ConstellationError::HardwareNotSupported { hardware }
            }
            constellation_vulkan::VulkanError::InsufficientMemory { required_bytes } => {
                ConstellationError::InsufficientMemory { required_bytes }
            }
            constellation_vulkan::VulkanError::GpuProcessingFailed { reason } => {
                ConstellationError::GpuProcessingFailed { reason }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        let error = ConstellationError::EngineInitializationFailed {
            reason: "test".to_string(),
        };
        assert_eq!(error.severity(), ErrorSeverity::Critical);
    }

    #[test]
    fn test_error_category() {
        let error = ConstellationError::NodeNotFound {
            node_id: Uuid::new_v4(),
        };
        assert_eq!(error.category(), ErrorCategory::Node);
    }

    #[test]
    fn test_user_message() {
        let error = ConstellationError::FileNotFound {
            path: "/test/path".to_string(),
        };
        assert!(error.user_message().contains("ファイルが見つかりません"));
    }

    #[test]
    fn test_is_recoverable() {
        let critical_error = ConstellationError::HardwareNotSupported {
            hardware: "GPU".to_string(),
        };
        assert!(!critical_error.is_recoverable());

        let recoverable_error = ConstellationError::NodeProcessingFailed {
            node_id: Uuid::new_v4(),
            reason: "test".to_string(),
        };
        assert!(recoverable_error.is_recoverable());
    }
}

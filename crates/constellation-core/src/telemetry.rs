use crate::error::{ConstellationError, ErrorCategory, ErrorSeverity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// 構造化ログとテレメトリシステム
pub struct TelemetryManager {
    pub metrics_collector: MetricsCollector,
    event_logger: EventLogger,
    performance_tracer: PerformanceTracer,
    error_tracker: ErrorTracker,
    session_id: Uuid,
    start_time: Instant,
}

/// メトリクス収集
#[derive(Debug)]
pub struct MetricsCollector {
    pub frame_count: AtomicU64,
    pub error_count: AtomicU64,
    pub total_processing_time: AtomicU64, // microseconds
    pub memory_usage_peak: AtomicU64,     // bytes
    pub gpu_utilization_samples: std::sync::Mutex<Vec<f32>>,
    pub custom_metrics: std::sync::Mutex<HashMap<String, MetricValue>>,
}

/// カスタムメトリック値
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Summary { count: u64, sum: f64, avg: f64 },
}

/// 構造化イベントログ
#[derive(Debug)]
pub struct EventLogger {
    buffer: std::sync::Mutex<Vec<LogEvent>>,
    max_buffer_size: usize,
}

/// ログイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub timestamp: u64, // Unix timestamp in milliseconds
    pub level: LogLevel,
    pub category: LogCategory,
    pub message: String,
    pub context: HashMap<String, serde_json::Value>,
    pub session_id: Uuid,
    pub node_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogCategory {
    System,
    Engine,
    Node,
    Frame,
    Resource,
    Network,
    Hardware,
    Security,
    Performance,
    User,
}

/// パフォーマンストレーサー
#[derive(Debug)]
pub struct PerformanceTracer {
    spans: std::sync::Mutex<HashMap<Uuid, PerformanceSpan>>,
    completed_spans: std::sync::Mutex<Vec<CompletedSpan>>,
}

#[derive(Debug, Clone)]
pub struct PerformanceSpan {
    pub id: Uuid,
    pub name: String,
    pub start_time: Instant,
    pub parent_id: Option<Uuid>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedSpan {
    pub id: Uuid,
    pub name: String,
    pub duration_us: u64,
    pub start_timestamp: u64,
    pub parent_id: Option<Uuid>,
    pub tags: HashMap<String, String>,
    pub events: Vec<SpanEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub timestamp: u64,
    pub name: String,
    pub attributes: HashMap<String, serde_json::Value>,
}

/// エラートラッカー
#[derive(Debug)]
pub struct ErrorTracker {
    error_counts: std::sync::Mutex<HashMap<String, u64>>,
    recent_errors: std::sync::Mutex<Vec<TrackedError>>,
    #[allow(dead_code)]
    error_patterns: std::sync::Mutex<Vec<ErrorPattern>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedError {
    pub timestamp: u64,
    pub error_type: String,
    pub message: String,
    pub severity: ErrorSeverity,
    pub category: ErrorCategory,
    pub context: HashMap<String, serde_json::Value>,
    pub node_id: Option<Uuid>,
    pub stack_trace: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ErrorPattern {
    pub pattern: String,
    pub count: u64,
    pub first_seen: Instant,
    pub last_seen: Instant,
}

impl TelemetryManager {
    pub fn new() -> Self {
        let session_id = Uuid::new_v4();

        info!(
            session_id = %session_id,
            "Constellation Studio telemetry initialized"
        );

        Self {
            metrics_collector: MetricsCollector::new(),
            event_logger: EventLogger::new(1000), // 1000 events buffer
            performance_tracer: PerformanceTracer::new(),
            error_tracker: ErrorTracker::new(),
            session_id,
            start_time: Instant::now(),
        }
    }

    /// フレーム処理開始のトレース
    pub fn start_frame_processing(&self, frame_id: Uuid) -> PerformanceSpanGuard {
        let span_id = self.performance_tracer.start_span(
            "frame_processing".to_string(),
            None,
            [("frame_id".to_string(), frame_id.to_string())]
                .iter()
                .cloned()
                .collect(),
        );

        debug!(
            frame_id = %frame_id,
            span_id = %span_id,
            "Started frame processing"
        );

        PerformanceSpanGuard {
            span_id,
            tracer: &self.performance_tracer,
        }
    }

    /// ノード処理開始のトレース
    pub fn start_node_processing(
        &self,
        node_id: Uuid,
        node_type: &str,
        parent_span: Option<Uuid>,
    ) -> PerformanceSpanGuard {
        let span_id = self.performance_tracer.start_span(
            format!("node_processing:{}", node_type),
            parent_span,
            [
                ("node_id".to_string(), node_id.to_string()),
                ("node_type".to_string(), node_type.to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        );

        debug!(
            node_id = %node_id,
            node_type = node_type,
            span_id = %span_id,
            "Started node processing"
        );

        PerformanceSpanGuard {
            span_id,
            tracer: &self.performance_tracer,
        }
    }

    /// エラーの記録
    pub fn record_error(&self, error: &ConstellationError, node_id: Option<Uuid>) {
        let tracked_error = TrackedError {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            error_type: std::any::type_name::<ConstellationError>().to_string(),
            message: error.to_string(),
            severity: error.severity(),
            category: error.category(),
            context: HashMap::new(),
            node_id,
            stack_trace: None, // TODO: キャプチャを実装
        };

        self.error_tracker.record_error(tracked_error.clone());

        // 構造化ログに記録
        let level = match error.severity() {
            ErrorSeverity::Critical => tracing::Level::ERROR,
            ErrorSeverity::Error => tracing::Level::ERROR,
            ErrorSeverity::Warning => tracing::Level::WARN,
            ErrorSeverity::Info => tracing::Level::INFO,
        };

        let log_event = LogEvent {
            timestamp: tracked_error.timestamp,
            level: match error.severity() {
                ErrorSeverity::Critical => LogLevel::Critical,
                ErrorSeverity::Error => LogLevel::Error,
                ErrorSeverity::Warning => LogLevel::Warn,
                ErrorSeverity::Info => LogLevel::Info,
            },
            category: LogCategory::Engine,
            message: error.user_message(),
            context: [
                (
                    "error_type".to_string(),
                    serde_json::Value::String(tracked_error.error_type),
                ),
                (
                    "severity".to_string(),
                    serde_json::Value::String(format!("{:?}", error.severity())),
                ),
                (
                    "category".to_string(),
                    serde_json::Value::String(format!("{:?}", error.category())),
                ),
                (
                    "recoverable".to_string(),
                    serde_json::Value::Bool(error.is_recoverable()),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
            session_id: self.session_id,
            node_id,
            correlation_id: None,
        };

        self.event_logger.record_event(log_event);

        // トレーシングマクロでの出力
        match level {
            tracing::Level::ERROR => {
                error!(
                    error = %error,
                    node_id = ?node_id,
                    severity = ?error.severity(),
                    category = ?error.category(),
                    recoverable = error.is_recoverable(),
                    "Constellation error occurred"
                );
            }
            tracing::Level::WARN => {
                warn!(
                    error = %error,
                    node_id = ?node_id,
                    severity = ?error.severity(),
                    category = ?error.category(),
                    "Constellation warning"
                );
            }
            tracing::Level::INFO => {
                info!(
                    message = %error,
                    node_id = ?node_id,
                    "Constellation info"
                );
            }
            _ => {}
        }

        // メトリクス更新
        self.metrics_collector
            .error_count
            .fetch_add(1, Ordering::Relaxed);
    }

    /// カスタムメトリクス記録
    pub fn record_metric(&self, name: String, value: MetricValue) {
        if let Ok(mut metrics) = self.metrics_collector.custom_metrics.lock() {
            metrics.insert(name.clone(), value.clone());
        }

        debug!(
            metric_name = name,
            metric_value = ?value,
            "Recorded custom metric"
        );
    }

    /// システム状態の記録
    pub fn record_system_state(&self, cpu_usage: f32, memory_usage: u64, gpu_usage: f32) {
        if let Ok(mut samples) = self.metrics_collector.gpu_utilization_samples.lock() {
            samples.push(gpu_usage);
            // 最新100サンプルのみ保持
            if samples.len() > 100 {
                samples.remove(0);
            }
        }

        // メモリ使用量のピーク更新
        let current_peak = self
            .metrics_collector
            .memory_usage_peak
            .load(Ordering::Relaxed);
        if memory_usage > current_peak {
            self.metrics_collector
                .memory_usage_peak
                .store(memory_usage, Ordering::Relaxed);
        }

        debug!(
            cpu_usage = cpu_usage,
            memory_usage = memory_usage,
            gpu_usage = gpu_usage,
            "System state recorded"
        );
    }

    /// セッション統計の取得
    pub fn get_session_stats(&self) -> SessionStats {
        let uptime = self.start_time.elapsed();
        let frame_count = self.metrics_collector.frame_count.load(Ordering::Relaxed);
        let error_count = self.metrics_collector.error_count.load(Ordering::Relaxed);
        let total_processing_time = Duration::from_micros(
            self.metrics_collector
                .total_processing_time
                .load(Ordering::Relaxed),
        );

        SessionStats {
            session_id: self.session_id,
            uptime,
            frame_count,
            error_count,
            total_processing_time,
            average_frame_time: if frame_count > 0 {
                Some(total_processing_time / frame_count as u32)
            } else {
                None
            },
            memory_peak: self
                .metrics_collector
                .memory_usage_peak
                .load(Ordering::Relaxed),
        }
    }

    /// ログの書き出し（JSON形式）
    pub fn export_logs_json(&self) -> serde_json::Result<String> {
        let events = self.event_logger.get_events();
        serde_json::to_string_pretty(&events)
    }

    /// パフォーマンストレースの書き出し
    pub fn export_traces_json(&self) -> serde_json::Result<String> {
        let traces = self.performance_tracer.get_completed_spans();
        serde_json::to_string_pretty(&traces)
    }
}

impl Default for TelemetryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub session_id: Uuid,
    pub uptime: Duration,
    pub frame_count: u64,
    pub error_count: u64,
    pub total_processing_time: Duration,
    pub average_frame_time: Option<Duration>,
    pub memory_peak: u64,
}

/// RAII パフォーマンススパンガード
pub struct PerformanceSpanGuard<'a> {
    span_id: Uuid,
    tracer: &'a PerformanceTracer,
}

impl Drop for PerformanceSpanGuard<'_> {
    fn drop(&mut self) {
        self.tracer.end_span(self.span_id);
    }
}

impl PerformanceSpanGuard<'_> {
    pub fn add_event(&self, name: String, attributes: HashMap<String, serde_json::Value>) {
        self.tracer.add_span_event(self.span_id, name, attributes);
    }

    pub fn span_id(&self) -> Uuid {
        self.span_id
    }
}

impl MetricsCollector {
    fn new() -> Self {
        Self {
            frame_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            total_processing_time: AtomicU64::new(0),
            memory_usage_peak: AtomicU64::new(0),
            gpu_utilization_samples: std::sync::Mutex::new(Vec::new()),
            custom_metrics: std::sync::Mutex::new(HashMap::new()),
        }
    }
}

impl EventLogger {
    fn new(max_buffer_size: usize) -> Self {
        Self {
            buffer: std::sync::Mutex::new(Vec::new()),
            max_buffer_size,
        }
    }

    fn record_event(&self, event: LogEvent) {
        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.push(event);

            // バッファサイズ制限
            while buffer.len() > self.max_buffer_size {
                buffer.remove(0);
            }
        }
    }

    fn get_events(&self) -> Vec<LogEvent> {
        self.buffer
            .lock()
            .unwrap_or_else(|_| panic!("Mutex poisoned"))
            .clone()
    }
}

impl PerformanceTracer {
    fn new() -> Self {
        Self {
            spans: std::sync::Mutex::new(HashMap::new()),
            completed_spans: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn start_span(
        &self,
        name: String,
        parent_id: Option<Uuid>,
        tags: HashMap<String, String>,
    ) -> Uuid {
        let span_id = Uuid::new_v4();
        let span = PerformanceSpan {
            id: span_id,
            name,
            start_time: Instant::now(),
            parent_id,
            tags,
        };

        if let Ok(mut spans) = self.spans.lock() {
            spans.insert(span_id, span);
        }

        span_id
    }

    fn end_span(&self, span_id: Uuid) {
        if let Ok(mut spans) = self.spans.lock() {
            if let Some(span) = spans.remove(&span_id) {
                let duration = span.start_time.elapsed();
                let completed_span = CompletedSpan {
                    id: span.id,
                    name: span.name,
                    duration_us: duration.as_micros() as u64,
                    start_timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                    parent_id: span.parent_id,
                    tags: span.tags,
                    events: Vec::new(), // 簡素化のため空
                };

                if let Ok(mut completed) = self.completed_spans.lock() {
                    completed.push(completed_span);

                    // 最新1000スパンのみ保持
                    if completed.len() > 1000 {
                        completed.remove(0);
                    }
                }
            }
        }
    }

    fn add_span_event(
        &self,
        span_id: Uuid,
        name: String,
        _attributes: HashMap<String, serde_json::Value>,
    ) {
        // 簡素化のため、イベントの追加は省略
        debug!(
            span_id = %span_id,
            event_name = name,
            "Span event added"
        );
    }

    fn get_completed_spans(&self) -> Vec<CompletedSpan> {
        self.completed_spans
            .lock()
            .unwrap_or_else(|_| panic!("Mutex poisoned"))
            .clone()
    }
}

impl ErrorTracker {
    fn new() -> Self {
        Self {
            error_counts: std::sync::Mutex::new(HashMap::new()),
            recent_errors: std::sync::Mutex::new(Vec::new()),
            error_patterns: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn record_error(&self, error: TrackedError) {
        // エラータイプ別カウント
        if let Ok(mut counts) = self.error_counts.lock() {
            *counts.entry(error.error_type.clone()).or_insert(0) += 1;
        }

        // 最近のエラー履歴
        if let Ok(mut recent) = self.recent_errors.lock() {
            recent.push(error);

            // 最新100エラーのみ保持
            if recent.len() > 100 {
                recent.remove(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_manager_creation() {
        let manager = TelemetryManager::new();
        let stats = manager.get_session_stats();
        assert_eq!(stats.frame_count, 0);
        assert_eq!(stats.error_count, 0);
    }

    #[test]
    fn test_performance_span() {
        let tracer = PerformanceTracer::new();
        let span_id = tracer.start_span("test_span".to_string(), None, HashMap::new());

        std::thread::sleep(Duration::from_millis(1));
        tracer.end_span(span_id);

        let completed = tracer.get_completed_spans();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].name, "test_span");
        assert!(completed[0].duration_us > 0);
    }

    #[test]
    fn test_metrics_collection() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.frame_count.load(Ordering::Relaxed), 0);

        collector.frame_count.fetch_add(1, Ordering::Relaxed);
        assert_eq!(collector.frame_count.load(Ordering::Relaxed), 1);
    }
}

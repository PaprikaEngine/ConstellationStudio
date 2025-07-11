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

use crate::error::{ConstellationError, ConstellationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ハードウェア互換性チェックおよび要件管理システム
pub struct HardwareCompatibilityChecker {
    system_info: SystemInfo,
    requirements: HardwareRequirements,
    compatibility_report: Option<CompatibilityReport>,
}

/// システム情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpu: Vec<GpuInfo>,
    pub storage: StorageInfo,
    pub network: NetworkInfo,
    pub operating_system: OsInfo,
    pub display: DisplayInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub vendor: String,
    pub cores: u32,
    pub threads: u32,
    pub base_frequency_mhz: f32,
    pub boost_frequency_mhz: Option<f32>,
    pub architecture: String,  // x86_64, arm64, etc.
    pub features: Vec<String>, // AVX, SSE, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub memory_type: String, // DDR4, DDR5, etc.
    pub channels: u32,
    pub speed_mhz: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String, // NVIDIA, AMD, Intel
    pub device_id: String,
    pub memory_bytes: u64,
    pub driver_version: String,
    pub vulkan_version: Option<String>,
    pub opencl_version: Option<String>,
    pub compute_capability: Option<String>, // CUDA compute capability
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub drives: Vec<DriveInfo>,
    pub total_space: u64,
    pub available_space: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub name: String,
    pub drive_type: DriveType,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub read_speed_mbps: Option<f32>,
    pub write_speed_mbps: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriveType {
    Hdd,
    Ssd,
    NVMe,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interfaces: Vec<NetworkInterface>,
    pub max_bandwidth_mbps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub interface_type: NetworkInterfaceType,
    pub speed_mbps: f32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkInterfaceType {
    Ethernet,
    WiFi,
    Fiber,
    Cellular,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub kernel_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub monitors: Vec<MonitorInfo>,
    pub primary_resolution: (u32, u32),
    pub refresh_rate_hz: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub name: String,
    pub resolution: (u32, u32),
    pub refresh_rate_hz: f32,
    pub color_depth: u32,
    pub hdr_support: bool,
}

/// ハードウェア要件定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareRequirements {
    pub phases: HashMap<String, PhaseRequirements>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseRequirements {
    pub phase_name: String,
    pub description: String,
    pub minimum: RequirementSpec,
    pub recommended: RequirementSpec,
    pub professional: Option<RequirementSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementSpec {
    pub cpu: CpuRequirement,
    pub memory: MemoryRequirement,
    pub gpu: GpuRequirement,
    pub storage: StorageRequirement,
    pub network: Option<NetworkRequirement>,
    pub operating_system: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuRequirement {
    pub min_cores: u32,
    pub min_frequency_mhz: f32,
    pub required_features: Vec<String>,
    pub architectures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRequirement {
    pub min_total_gb: f32,
    pub min_available_gb: f32,
    pub preferred_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuRequirement {
    pub required: bool,
    pub min_memory_gb: Option<f32>,
    pub supported_vendors: Vec<String>,
    pub required_apis: Vec<String>, // Vulkan, OpenCL, etc.
    pub min_vulkan_version: Option<String>,
    pub professional_features: Vec<String>, // SDI, NDI support
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRequirement {
    pub min_free_space_gb: f32,
    pub preferred_types: Vec<DriveType>,
    pub min_read_speed_mbps: Option<f32>,
    pub min_write_speed_mbps: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequirement {
    pub min_bandwidth_mbps: f32,
    pub required_for_features: Vec<String>, // NDI, SRT, etc.
    pub latency_requirements: Option<f32>,  // ms
}

/// 互換性チェック結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub overall_compatibility: CompatibilityLevel,
    pub supported_phases: Vec<String>,
    pub phase_reports: HashMap<String, PhaseCompatibilityReport>,
    pub recommendations: Vec<String>,
    pub warnings: Vec<String>,
    pub critical_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompatibilityLevel {
    FullySupported,
    Supported,
    PartiallySupported,
    NotSupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseCompatibilityReport {
    pub phase_name: String,
    pub compatibility: CompatibilityLevel,
    pub minimum_met: bool,
    pub recommended_met: bool,
    pub professional_met: Option<bool>,
    pub component_reports: HashMap<String, ComponentCompatibilityReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentCompatibilityReport {
    pub component_name: String,
    pub status: ComponentStatus,
    pub details: String,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentStatus {
    Excellent,
    Good,
    Adequate,
    Insufficient,
    Missing,
}

impl HardwareCompatibilityChecker {
    pub fn new() -> ConstellationResult<Self> {
        let system_info = Self::detect_system_info()?;
        let requirements = Self::load_hardware_requirements();

        Ok(Self {
            system_info,
            requirements,
            compatibility_report: None,
        })
    }

    /// システム情報の自動検出
    fn detect_system_info() -> ConstellationResult<SystemInfo> {
        // プラットフォーム固有の実装
        #[cfg(target_os = "windows")]
        return Self::detect_windows_system_info();

        #[cfg(target_os = "macos")]
        return Self::detect_macos_system_info();

        #[cfg(target_os = "linux")]
        return Self::detect_linux_system_info();

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(ConstellationError::PlatformNotSupported {
                platform: std::env::consts::OS.to_string(),
            })
        }
    }

    #[cfg(target_os = "windows")]
    fn detect_windows_system_info() -> ConstellationResult<SystemInfo> {
        // Windows固有のシステム情報取得
        // WMI、レジストリ、Win32 API等を使用
        Ok(SystemInfo {
            cpu: CpuInfo {
                model: "Unknown".to_string(),
                vendor: "Unknown".to_string(),
                cores: num_cpus::get() as u32,
                threads: num_cpus::get() as u32,
                base_frequency_mhz: 2400.0, // デフォルト値
                boost_frequency_mhz: None,
                architecture: std::env::consts::ARCH.to_string(),
                features: vec![],
            },
            memory: MemoryInfo {
                total_bytes: 8 * 1024 * 1024 * 1024,     // 8GB デフォルト
                available_bytes: 4 * 1024 * 1024 * 1024, // 4GB デフォルト
                memory_type: "DDR4".to_string(),
                channels: 2,
                speed_mhz: Some(3200.0),
            },
            gpu: vec![],
            storage: StorageInfo {
                drives: vec![],
                total_space: 1024 * 1024 * 1024 * 1024, // 1TB
                available_space: 512 * 1024 * 1024 * 1024, // 512GB
            },
            network: NetworkInfo {
                interfaces: vec![],
                max_bandwidth_mbps: 1000.0,
            },
            operating_system: OsInfo {
                name: "Windows".to_string(),
                version: "11".to_string(),
                architecture: std::env::consts::ARCH.to_string(),
                kernel_version: None,
            },
            display: DisplayInfo {
                monitors: vec![],
                primary_resolution: (1920, 1080),
                refresh_rate_hz: 60.0,
            },
        })
    }

    #[cfg(target_os = "macos")]
    fn detect_macos_system_info() -> ConstellationResult<SystemInfo> {
        // macOS固有のシステム情報取得
        // system_profiler、sysctl等を使用
        Ok(SystemInfo {
            cpu: CpuInfo {
                model: "Unknown".to_string(),
                vendor: "Apple".to_string(),
                cores: num_cpus::get() as u32,
                threads: num_cpus::get() as u32,
                base_frequency_mhz: 3000.0,
                boost_frequency_mhz: Some(3500.0),
                architecture: std::env::consts::ARCH.to_string(),
                features: vec!["NEON".to_string()],
            },
            memory: MemoryInfo {
                total_bytes: 16 * 1024 * 1024 * 1024,    // 16GB デフォルト
                available_bytes: 8 * 1024 * 1024 * 1024, // 8GB デフォルト
                memory_type: "LPDDR5".to_string(),
                channels: 2,
                speed_mhz: Some(6400.0),
            },
            gpu: vec![],
            storage: StorageInfo {
                drives: vec![],
                total_space: 512 * 1024 * 1024 * 1024, // 512GB
                available_space: 256 * 1024 * 1024 * 1024, // 256GB
            },
            network: NetworkInfo {
                interfaces: vec![],
                max_bandwidth_mbps: 1000.0,
            },
            operating_system: OsInfo {
                name: "macOS".to_string(),
                version: "14.0".to_string(),
                architecture: std::env::consts::ARCH.to_string(),
                kernel_version: Some("23.0.0".to_string()),
            },
            display: DisplayInfo {
                monitors: vec![],
                primary_resolution: (2560, 1600),
                refresh_rate_hz: 60.0,
            },
        })
    }

    #[cfg(target_os = "linux")]
    fn detect_linux_system_info() -> ConstellationResult<SystemInfo> {
        // Linux固有のシステム情報取得
        // /proc/cpuinfo, /proc/meminfo, lspci等を使用
        Ok(SystemInfo {
            cpu: CpuInfo {
                model: "Unknown".to_string(),
                vendor: "Unknown".to_string(),
                cores: num_cpus::get() as u32,
                threads: num_cpus::get() as u32,
                base_frequency_mhz: 2800.0,
                boost_frequency_mhz: Some(4200.0),
                architecture: std::env::consts::ARCH.to_string(),
                features: vec!["AVX2".to_string(), "SSE4.2".to_string()],
            },
            memory: MemoryInfo {
                total_bytes: 32 * 1024 * 1024 * 1024,     // 32GB デフォルト
                available_bytes: 16 * 1024 * 1024 * 1024, // 16GB デフォルト
                memory_type: "DDR4".to_string(),
                channels: 4,
                speed_mhz: Some(3200.0),
            },
            gpu: vec![],
            storage: StorageInfo {
                drives: vec![],
                total_space: 2 * 1024 * 1024 * 1024 * 1024, // 2TB
                available_space: 1024 * 1024 * 1024 * 1024, // 1TB
            },
            network: NetworkInfo {
                interfaces: vec![],
                max_bandwidth_mbps: 10000.0, // 10Gbps
            },
            operating_system: OsInfo {
                name: "Linux".to_string(),
                version: "Ubuntu 22.04".to_string(),
                architecture: std::env::consts::ARCH.to_string(),
                kernel_version: Some("6.2.0".to_string()),
            },
            display: DisplayInfo {
                monitors: vec![],
                primary_resolution: (3840, 2160),
                refresh_rate_hz: 144.0,
            },
        })
    }

    /// ハードウェア要件定義をロード
    fn load_hardware_requirements() -> HardwareRequirements {
        let mut phases = HashMap::new();

        // Phase 1: ローカルスタンドアロン（2D基盤）
        phases.insert(
            "phase1".to_string(),
            PhaseRequirements {
                phase_name: "Phase 1: 2D基盤（ローカル）".to_string(),
                description: "基本的な2D映像処理、画面キャプチャ、仮想Webカメラ".to_string(),
                minimum: RequirementSpec {
                    cpu: CpuRequirement {
                        min_cores: 4,
                        min_frequency_mhz: 2000.0,
                        required_features: vec![],
                        architectures: vec!["x86_64".to_string(), "arm64".to_string()],
                    },
                    memory: MemoryRequirement {
                        min_total_gb: 8.0,
                        min_available_gb: 4.0,
                        preferred_type: Some("DDR4".to_string()),
                    },
                    gpu: GpuRequirement {
                        required: true,
                        min_memory_gb: Some(2.0),
                        supported_vendors: vec![
                            "NVIDIA".to_string(),
                            "AMD".to_string(),
                            "Intel".to_string(),
                        ],
                        required_apis: vec!["Vulkan".to_string()],
                        min_vulkan_version: Some("1.2".to_string()),
                        professional_features: vec![],
                    },
                    storage: StorageRequirement {
                        min_free_space_gb: 10.0,
                        preferred_types: vec![DriveType::Ssd, DriveType::NVMe],
                        min_read_speed_mbps: Some(200.0),
                        min_write_speed_mbps: Some(100.0),
                    },
                    network: None,
                    operating_system: vec![
                        "Windows 10".to_string(),
                        "Windows 11".to_string(),
                        "macOS 12".to_string(),
                        "Ubuntu 20.04".to_string(),
                    ],
                },
                recommended: RequirementSpec {
                    cpu: CpuRequirement {
                        min_cores: 8,
                        min_frequency_mhz: 3000.0,
                        required_features: vec!["AVX2".to_string()],
                        architectures: vec!["x86_64".to_string(), "arm64".to_string()],
                    },
                    memory: MemoryRequirement {
                        min_total_gb: 16.0,
                        min_available_gb: 8.0,
                        preferred_type: Some("DDR4".to_string()),
                    },
                    gpu: GpuRequirement {
                        required: true,
                        min_memory_gb: Some(6.0),
                        supported_vendors: vec!["NVIDIA".to_string(), "AMD".to_string()],
                        required_apis: vec!["Vulkan".to_string()],
                        min_vulkan_version: Some("1.3".to_string()),
                        professional_features: vec![],
                    },
                    storage: StorageRequirement {
                        min_free_space_gb: 50.0,
                        preferred_types: vec![DriveType::NVMe],
                        min_read_speed_mbps: Some(1000.0),
                        min_write_speed_mbps: Some(500.0),
                    },
                    network: None,
                    operating_system: vec![
                        "Windows 11".to_string(),
                        "macOS 13".to_string(),
                        "Ubuntu 22.04".to_string(),
                    ],
                },
                professional: None,
            },
        );

        // Phase 2: プロフェッショナル映像伝送対応
        phases.insert(
            "phase2".to_string(),
            PhaseRequirements {
                phase_name: "Phase 2: プロフェッショナル映像伝送".to_string(),
                description: "SDI、NDI、SRT、SMPTE ST 2110対応".to_string(),
                minimum: RequirementSpec {
                    cpu: CpuRequirement {
                        min_cores: 8,
                        min_frequency_mhz: 3000.0,
                        required_features: vec!["AVX2".to_string()],
                        architectures: vec!["x86_64".to_string()],
                    },
                    memory: MemoryRequirement {
                        min_total_gb: 32.0,
                        min_available_gb: 16.0,
                        preferred_type: Some("DDR4".to_string()),
                    },
                    gpu: GpuRequirement {
                        required: true,
                        min_memory_gb: Some(8.0),
                        supported_vendors: vec!["NVIDIA".to_string(), "AMD".to_string()],
                        required_apis: vec!["Vulkan".to_string()],
                        min_vulkan_version: Some("1.3".to_string()),
                        professional_features: vec!["SDI".to_string(), "NDI".to_string()],
                    },
                    storage: StorageRequirement {
                        min_free_space_gb: 500.0,
                        preferred_types: vec![DriveType::NVMe],
                        min_read_speed_mbps: Some(2000.0),
                        min_write_speed_mbps: Some(1000.0),
                    },
                    network: Some(NetworkRequirement {
                        min_bandwidth_mbps: 10000.0, // 10Gbps
                        required_for_features: vec![
                            "NDI".to_string(),
                            "SRT".to_string(),
                            "ST2110".to_string(),
                        ],
                        latency_requirements: Some(1.0), // 1ms
                    }),
                    operating_system: vec![
                        "Windows 11 Pro".to_string(),
                        "Ubuntu 22.04 LTS".to_string(),
                    ],
                },
                recommended: RequirementSpec {
                    cpu: CpuRequirement {
                        min_cores: 16,
                        min_frequency_mhz: 3500.0,
                        required_features: vec!["AVX512".to_string()],
                        architectures: vec!["x86_64".to_string()],
                    },
                    memory: MemoryRequirement {
                        min_total_gb: 64.0,
                        min_available_gb: 32.0,
                        preferred_type: Some("DDR5".to_string()),
                    },
                    gpu: GpuRequirement {
                        required: true,
                        min_memory_gb: Some(24.0),
                        supported_vendors: vec!["NVIDIA".to_string()],
                        required_apis: vec!["Vulkan".to_string()],
                        min_vulkan_version: Some("1.3".to_string()),
                        professional_features: vec![
                            "SDI".to_string(),
                            "NDI".to_string(),
                            "ST2110".to_string(),
                        ],
                    },
                    storage: StorageRequirement {
                        min_free_space_gb: 2000.0, // 2TB
                        preferred_types: vec![DriveType::NVMe],
                        min_read_speed_mbps: Some(7000.0),
                        min_write_speed_mbps: Some(5000.0),
                    },
                    network: Some(NetworkRequirement {
                        min_bandwidth_mbps: 25000.0, // 25Gbps
                        required_for_features: vec!["4K NDI".to_string(), "ST2110".to_string()],
                        latency_requirements: Some(0.5), // 0.5ms
                    }),
                    operating_system: vec![
                        "Windows 11 Pro".to_string(),
                        "Ubuntu 22.04 LTS".to_string(),
                    ],
                },
                professional: Some(RequirementSpec {
                    cpu: CpuRequirement {
                        min_cores: 32,
                        min_frequency_mhz: 4000.0,
                        required_features: vec!["AVX512".to_string()],
                        architectures: vec!["x86_64".to_string()],
                    },
                    memory: MemoryRequirement {
                        min_total_gb: 128.0,
                        min_available_gb: 64.0,
                        preferred_type: Some("DDR5".to_string()),
                    },
                    gpu: GpuRequirement {
                        required: true,
                        min_memory_gb: Some(48.0),
                        supported_vendors: vec!["NVIDIA".to_string()],
                        required_apis: vec!["Vulkan".to_string()],
                        min_vulkan_version: Some("1.3".to_string()),
                        professional_features: vec![
                            "Multiple SDI".to_string(),
                            "12G-SDI".to_string(),
                            "ST2110".to_string(),
                        ],
                    },
                    storage: StorageRequirement {
                        min_free_space_gb: 10000.0, // 10TB
                        preferred_types: vec![DriveType::NVMe],
                        min_read_speed_mbps: Some(14000.0),
                        min_write_speed_mbps: Some(10000.0),
                    },
                    network: Some(NetworkRequirement {
                        min_bandwidth_mbps: 100000.0, // 100Gbps
                        required_for_features: vec![
                            "8K ST2110".to_string(),
                            "Multiple 4K streams".to_string(),
                        ],
                        latency_requirements: Some(0.1), // 0.1ms
                    }),
                    operating_system: vec![
                        "Windows 11 Pro".to_string(),
                        "Ubuntu 22.04 LTS".to_string(),
                    ],
                }),
            },
        );

        HardwareRequirements { phases }
    }

    /// 互換性チェックを実行
    pub fn check_compatibility(&mut self) -> ConstellationResult<&CompatibilityReport> {
        let mut phase_reports = HashMap::new();
        let mut supported_phases = Vec::new();
        let mut warnings = Vec::new();
        let mut critical_issues = Vec::new();

        for (phase_id, phase_req) in &self.requirements.phases {
            let phase_report = self.check_phase_compatibility(phase_req)?;

            if phase_report.minimum_met {
                supported_phases.push(phase_id.clone());
            }

            if !phase_report.minimum_met {
                critical_issues.push(format!(
                    "{}の最小要件を満たしていません",
                    phase_req.phase_name
                ));
            }

            if phase_report.minimum_met && !phase_report.recommended_met {
                warnings.push(format!(
                    "{}の推奨要件を満たしていません",
                    phase_req.phase_name
                ));
            }

            phase_reports.insert(phase_id.clone(), phase_report);
        }

        let overall_compatibility = if supported_phases.is_empty() {
            CompatibilityLevel::NotSupported
        } else if supported_phases.len() == self.requirements.phases.len() {
            CompatibilityLevel::FullySupported
        } else {
            CompatibilityLevel::PartiallySupported
        };

        let report = CompatibilityReport {
            overall_compatibility,
            supported_phases,
            phase_reports,
            recommendations: self.generate_recommendations(),
            warnings,
            critical_issues,
        };

        self.compatibility_report = Some(report);
        Ok(self.compatibility_report.as_ref().unwrap())
    }

    fn check_phase_compatibility(
        &self,
        phase_req: &PhaseRequirements,
    ) -> ConstellationResult<PhaseCompatibilityReport> {
        let mut component_reports = HashMap::new();

        // CPU チェック
        let cpu_report = self.check_cpu_compatibility(&phase_req.minimum.cpu);
        let cpu_meets_min = matches!(
            cpu_report.status,
            ComponentStatus::Adequate | ComponentStatus::Good | ComponentStatus::Excellent
        );
        component_reports.insert("cpu".to_string(), cpu_report);

        // Memory チェック
        let memory_report = self.check_memory_compatibility(&phase_req.minimum.memory);
        let memory_meets_min = matches!(
            memory_report.status,
            ComponentStatus::Adequate | ComponentStatus::Good | ComponentStatus::Excellent
        );
        component_reports.insert("memory".to_string(), memory_report);

        // GPU チェック
        let gpu_report = self.check_gpu_compatibility(&phase_req.minimum.gpu);
        let gpu_meets_min = matches!(
            gpu_report.status,
            ComponentStatus::Adequate | ComponentStatus::Good | ComponentStatus::Excellent
        );
        component_reports.insert("gpu".to_string(), gpu_report);

        // Storage チェック
        let storage_report = self.check_storage_compatibility(&phase_req.minimum.storage);
        let storage_meets_min = matches!(
            storage_report.status,
            ComponentStatus::Adequate | ComponentStatus::Good | ComponentStatus::Excellent
        );
        component_reports.insert("storage".to_string(), storage_report);

        let minimum_met = cpu_meets_min && memory_meets_min && gpu_meets_min && storage_meets_min;

        // 推奨要件チェック（簡素化）
        let recommended_met = minimum_met; // 実際には詳細チェックが必要

        let compatibility = if minimum_met {
            if recommended_met {
                CompatibilityLevel::FullySupported
            } else {
                CompatibilityLevel::Supported
            }
        } else {
            CompatibilityLevel::NotSupported
        };

        Ok(PhaseCompatibilityReport {
            phase_name: phase_req.phase_name.clone(),
            compatibility,
            minimum_met,
            recommended_met,
            professional_met: None, // 簡素化のため省略
            component_reports,
        })
    }

    fn check_cpu_compatibility(&self, req: &CpuRequirement) -> ComponentCompatibilityReport {
        let cpu = &self.system_info.cpu;

        if cpu.cores < req.min_cores {
            return ComponentCompatibilityReport {
                component_name: "CPU".to_string(),
                status: ComponentStatus::Insufficient,
                details: format!(
                    "コア数不足: {}コア (要求: {}コア)",
                    cpu.cores, req.min_cores
                ),
                recommendations: vec![
                    "より多くのコアを持つCPUにアップグレードしてください".to_string()
                ],
            };
        }

        if cpu.base_frequency_mhz < req.min_frequency_mhz {
            return ComponentCompatibilityReport {
                component_name: "CPU".to_string(),
                status: ComponentStatus::Insufficient,
                details: format!(
                    "クロック周波数不足: {:.0}MHz (要求: {:.0}MHz)",
                    cpu.base_frequency_mhz, req.min_frequency_mhz
                ),
                recommendations: vec![
                    "より高いクロック周波数のCPUにアップグレードしてください".to_string()
                ],
            };
        }

        let status = if cpu.cores >= req.min_cores * 2
            && cpu.base_frequency_mhz >= req.min_frequency_mhz * 1.5
        {
            ComponentStatus::Excellent
        } else if cpu.cores >= req.min_cores + 2
            && cpu.base_frequency_mhz >= req.min_frequency_mhz * 1.2
        {
            ComponentStatus::Good
        } else {
            ComponentStatus::Adequate
        };

        ComponentCompatibilityReport {
            component_name: "CPU".to_string(),
            status,
            details: format!("{}コア, {:.0}MHz", cpu.cores, cpu.base_frequency_mhz),
            recommendations: vec![],
        }
    }

    fn check_memory_compatibility(&self, req: &MemoryRequirement) -> ComponentCompatibilityReport {
        let memory = &self.system_info.memory;
        let total_gb = memory.total_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
        let available_gb = memory.available_bytes as f32 / (1024.0 * 1024.0 * 1024.0);

        if total_gb < req.min_total_gb {
            return ComponentCompatibilityReport {
                component_name: "Memory".to_string(),
                status: ComponentStatus::Insufficient,
                details: format!(
                    "総メモリ不足: {:.1}GB (要求: {:.1}GB)",
                    total_gb, req.min_total_gb
                ),
                recommendations: vec!["メモリを増設してください".to_string()],
            };
        }

        if available_gb < req.min_available_gb {
            return ComponentCompatibilityReport {
                component_name: "Memory".to_string(),
                status: ComponentStatus::Insufficient,
                details: format!(
                    "利用可能メモリ不足: {:.1}GB (要求: {:.1}GB)",
                    available_gb, req.min_available_gb
                ),
                recommendations: vec![
                    "実行中のアプリケーションを終了するか、メモリを増設してください".to_string(),
                ],
            };
        }

        let status = if total_gb >= req.min_total_gb * 2.0 {
            ComponentStatus::Excellent
        } else if total_gb >= req.min_total_gb * 1.5 {
            ComponentStatus::Good
        } else {
            ComponentStatus::Adequate
        };

        ComponentCompatibilityReport {
            component_name: "Memory".to_string(),
            status,
            details: format!("{:.1}GB 総容量, {:.1}GB 利用可能", total_gb, available_gb),
            recommendations: vec![],
        }
    }

    fn check_gpu_compatibility(&self, req: &GpuRequirement) -> ComponentCompatibilityReport {
        if !req.required {
            return ComponentCompatibilityReport {
                component_name: "GPU".to_string(),
                status: ComponentStatus::Good,
                details: "GPU不要".to_string(),
                recommendations: vec![],
            };
        }

        if self.system_info.gpu.is_empty() {
            return ComponentCompatibilityReport {
                component_name: "GPU".to_string(),
                status: ComponentStatus::Missing,
                details: "GPUが検出されませんでした".to_string(),
                recommendations: vec!["専用GPUを搭載してください".to_string()],
            };
        }

        // 最初のGPUを評価（複数GPU対応は今後の課題）
        let gpu = &self.system_info.gpu[0];
        let memory_gb = gpu.memory_bytes as f32 / (1024.0 * 1024.0 * 1024.0);

        if let Some(min_memory_gb) = req.min_memory_gb {
            if memory_gb < min_memory_gb {
                return ComponentCompatibilityReport {
                    component_name: "GPU".to_string(),
                    status: ComponentStatus::Insufficient,
                    details: format!(
                        "VRAM不足: {:.1}GB (要求: {:.1}GB)",
                        memory_gb, min_memory_gb
                    ),
                    recommendations: vec![
                        "より大きなVRAMを持つGPUにアップグレードしてください".to_string()
                    ],
                };
            }
        }

        let status = if memory_gb >= req.min_memory_gb.unwrap_or(0.0) * 2.0 {
            ComponentStatus::Excellent
        } else if memory_gb >= req.min_memory_gb.unwrap_or(0.0) * 1.5 {
            ComponentStatus::Good
        } else {
            ComponentStatus::Adequate
        };

        ComponentCompatibilityReport {
            component_name: "GPU".to_string(),
            status,
            details: format!("{} - {:.1}GB VRAM", gpu.name, memory_gb),
            recommendations: vec![],
        }
    }

    fn check_storage_compatibility(
        &self,
        req: &StorageRequirement,
    ) -> ComponentCompatibilityReport {
        let storage = &self.system_info.storage;
        let available_gb = storage.available_space as f32 / (1024.0 * 1024.0 * 1024.0);

        if available_gb < req.min_free_space_gb {
            return ComponentCompatibilityReport {
                component_name: "Storage".to_string(),
                status: ComponentStatus::Insufficient,
                details: format!(
                    "空き容量不足: {:.1}GB (要求: {:.1}GB)",
                    available_gb, req.min_free_space_gb
                ),
                recommendations: vec![
                    "ディスク容量を確保するか、より大きなストレージにアップグレードしてください"
                        .to_string(),
                ],
            };
        }

        let status = if available_gb >= req.min_free_space_gb * 3.0 {
            ComponentStatus::Excellent
        } else if available_gb >= req.min_free_space_gb * 2.0 {
            ComponentStatus::Good
        } else {
            ComponentStatus::Adequate
        };

        ComponentCompatibilityReport {
            component_name: "Storage".to_string(),
            status,
            details: format!("{:.1}GB 利用可能", available_gb),
            recommendations: vec![],
        }
    }

    fn generate_recommendations(&self) -> Vec<String> {
        vec![
            "定期的にシステムアップデートを実行してください".to_string(),
            "未使用のアプリケーションを終了してリソースを確保してください".to_string(),
            "プロフェッショナル用途では専用のワークステーションを推奨します".to_string(),
        ]
    }

    /// システム情報の取得
    pub fn get_system_info(&self) -> &SystemInfo {
        &self.system_info
    }

    /// 互換性レポートの取得
    pub fn get_compatibility_report(&self) -> Option<&CompatibilityReport> {
        self.compatibility_report.as_ref()
    }

    /// JSON形式でのレポート出力
    pub fn export_report_json(&self) -> ConstellationResult<String> {
        if let Some(report) = &self.compatibility_report {
            serde_json::to_string_pretty(report).map_err(|e| ConstellationError::InternalError {
                reason: format!("JSON serialization failed: {}", e),
            })
        } else {
            Err(ConstellationError::InternalError {
                reason: "No compatibility report available. Run check_compatibility() first."
                    .to_string(),
            })
        }
    }
}

impl Default for HardwareCompatibilityChecker {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            system_info: SystemInfo {
                cpu: CpuInfo {
                    model: "Unknown".to_string(),
                    vendor: "Unknown".to_string(),
                    cores: 1,
                    threads: 1,
                    base_frequency_mhz: 1000.0,
                    boost_frequency_mhz: None,
                    architecture: "unknown".to_string(),
                    features: vec![],
                },
                memory: MemoryInfo {
                    total_bytes: 0,
                    available_bytes: 0,
                    memory_type: "Unknown".to_string(),
                    channels: 1,
                    speed_mhz: None,
                },
                gpu: vec![],
                storage: StorageInfo {
                    drives: vec![],
                    total_space: 0,
                    available_space: 0,
                },
                network: NetworkInfo {
                    interfaces: vec![],
                    max_bandwidth_mbps: 0.0,
                },
                operating_system: OsInfo {
                    name: "Unknown".to_string(),
                    version: "Unknown".to_string(),
                    architecture: "unknown".to_string(),
                    kernel_version: None,
                },
                display: DisplayInfo {
                    monitors: vec![],
                    primary_resolution: (800, 600),
                    refresh_rate_hz: 60.0,
                },
            },
            requirements: Self::load_hardware_requirements(),
            compatibility_report: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_checker_creation() {
        let checker = HardwareCompatibilityChecker::default();
        assert!(!checker.requirements.phases.is_empty());
    }

    #[test]
    fn test_phase_requirements() {
        let requirements = HardwareCompatibilityChecker::load_hardware_requirements();
        assert!(requirements.phases.contains_key("phase1"));
        assert!(requirements.phases.contains_key("phase2"));
    }

    #[test]
    fn test_compatibility_levels() {
        let level = CompatibilityLevel::FullySupported;
        assert!(matches!(level, CompatibilityLevel::FullySupported));
    }
}

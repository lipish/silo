// 推理引擎管理器 - 根据硬件自动选择最优后端

use crate::engine::backend::{InferenceBackend, LlamaCppBackend, MlxBackend, InferflowBackend};
use crate::engine::{BackendType, InferenceConfig, InferenceResponse};
use anyhow::Result;
use sysinfo::System;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct EngineManager {
    backend: Arc<RwLock<Box<dyn InferenceBackend>>>,
    current_backend_type: BackendType,
    initialized: bool,
}

impl EngineManager {
    pub fn new() -> Self {
        Self {
            backend: Arc::new(RwLock::new(Box::new(LlamaCppBackend::new()))),
            current_backend_type: BackendType::LlamaCppCpu,
            initialized: false,
        }
    }
    
    /// 检测硬件并选择最优后端
    pub async fn detect_and_select_backend(&mut self) -> Result<BackendType> {
        let mut sys = System::new();
        sys.refresh_all();
        
        // 策略 1: Apple Silicon (Mac)
        if cfg!(target_os = "macos") {
            // 检查是否为 Apple Silicon
            let cpu_name = sys.cpus().first().map(|c| c.name().to_lowercase()).unwrap_or_default();
            if cpu_name.contains("apple") || cpu_name.contains("m1") || cpu_name.contains("m2") || cpu_name.contains("m3") {
                let mlx_backend = MlxBackend {};
                if mlx_backend.is_available() {
                    *self.backend.write().await = Box::new(mlx_backend);
                    self.current_backend_type = BackendType::MlxSidecar;
                    tracing::info!("Selected MLX backend for Apple Silicon");
                    return Ok(BackendType::MlxSidecar);
                }
            }
        }
        
        // 策略 2: NVIDIA GPU (PC/Server)
        // TODO: 使用 sysinfo 或 nvidia-smi 检测 GPU
        let inferflow_backend = InferflowBackend {};
        if inferflow_backend.is_available() {
            *self.backend.write().await = Box::new(inferflow_backend);
            self.current_backend_type = BackendType::InferflowCpp;
            tracing::info!("Selected Inferflow backend for NVIDIA GPU");
            return Ok(BackendType::InferflowCpp);
        }
        
        // 策略 3: 默认 CPU 后端
        *self.backend.write().await = Box::new(LlamaCppBackend::new());
        self.current_backend_type = BackendType::LlamaCppCpu;
        tracing::info!("Selected Llama.cpp CPU backend");
        Ok(BackendType::LlamaCppCpu)
    }
    
    /// 初始化推理引擎
    pub async fn initialize(&mut self, config: InferenceConfig) -> Result<()> {
        let mut backend = self.backend.write().await;
        backend.initialize(config).await?;
        self.initialized = true;
        Ok(())
    }
    
    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// 执行推理
    pub async fn infer(&self, prompt: &str) -> Result<InferenceResponse> {
        let backend = self.backend.read().await;
        backend.infer(prompt).await
    }
    
    /// 流式推理
    pub async fn infer_stream(&self, prompt: &str) -> Result<tokio::sync::mpsc::Receiver<String>> {
        let backend = self.backend.read().await;
        backend.infer_stream(prompt).await
    }
    
    /// 获取当前后端类型
    pub fn current_backend_type(&self) -> BackendType {
        self.current_backend_type.clone()
    }
}

impl Default for EngineManager {
    fn default() -> Self {
        Self::new()
    }
}

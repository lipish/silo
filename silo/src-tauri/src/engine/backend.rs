// 推理后端抽象接口

use crate::engine::{InferenceConfig, InferenceResponse};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait InferenceBackend: Send + Sync {
    /// 初始化后端
    async fn initialize(&mut self, config: InferenceConfig) -> Result<()>;
    
    /// 执行推理
    async fn infer(&self, prompt: &str) -> Result<InferenceResponse>;
    
    /// 流式推理
    async fn infer_stream(&self, prompt: &str) -> Result<tokio::sync::mpsc::Receiver<String>>;
    
    /// 获取后端类型
    fn backend_type(&self) -> crate::engine::BackendType;
    
    /// 检查后端是否可用
    fn is_available(&self) -> bool;
}

// MLX Sidecar 后端 (Mac 优化)
pub struct MlxBackend {
    // MLX Python 侧车进程句柄
    // 通过 gRPC 或 Unix Domain Socket 通信
}

#[async_trait]
impl InferenceBackend for MlxBackend {
    async fn initialize(&mut self, _config: InferenceConfig) -> Result<()> {
        // TODO: 启动 Python MLX 侧车进程
        todo!()
    }
    
    async fn infer(&self, _prompt: &str) -> Result<InferenceResponse> {
        // TODO: 通过 IPC 调用 MLX
        todo!()
    }
    
    async fn infer_stream(&self, _prompt: &str) -> Result<tokio::sync::mpsc::Receiver<String>> {
        // TODO: 流式调用 MLX
        todo!()
    }
    
    fn backend_type(&self) -> crate::engine::BackendType {
        crate::engine::BackendType::MlxSidecar
    }
    
    fn is_available(&self) -> bool {
        // 检查是否为 macOS 且为 Apple Silicon
        cfg!(target_os = "macos")
    }
}

// Inferflow 后端 (PC/Server)
pub struct InferflowBackend {
    // Inferflow C++ 库绑定
}

#[async_trait]
impl InferenceBackend for InferflowBackend {
    async fn initialize(&mut self, _config: InferenceConfig) -> Result<()> {
        // TODO: 加载 Inferflow 库并初始化
        todo!()
    }
    
    async fn infer(&self, _prompt: &str) -> Result<InferenceResponse> {
        // TODO: 调用 Inferflow 推理
        todo!()
    }
    
    async fn infer_stream(&self, _prompt: &str) -> Result<tokio::sync::mpsc::Receiver<String>> {
        // TODO: 流式调用 Inferflow
        todo!()
    }
    
    fn backend_type(&self) -> crate::engine::BackendType {
        crate::engine::BackendType::InferflowCpp
    }
    
    fn is_available(&self) -> bool {
        // 检查是否有 NVIDIA GPU
        // TODO: 使用 sysinfo 或 nvidia-smi 检测
        false
    }
}

// Llama.cpp 后端 (通用 CPU)
pub struct LlamaCppBackend {
    // llama.cpp Rust 绑定
}

#[async_trait]
impl InferenceBackend for LlamaCppBackend {
    async fn initialize(&mut self, _config: InferenceConfig) -> Result<()> {
        // TODO: 加载 llama.cpp 模型
        todo!()
    }
    
    async fn infer(&self, _prompt: &str) -> Result<InferenceResponse> {
        // TODO: 调用 llama.cpp 推理
        todo!()
    }
    
    async fn infer_stream(&self, _prompt: &str) -> Result<tokio::sync::mpsc::Receiver<String>> {
        // TODO: 流式调用 llama.cpp
        todo!()
    }
    
    fn backend_type(&self) -> crate::engine::BackendType {
        crate::engine::BackendType::LlamaCppCpu
    }
    
    fn is_available(&self) -> bool {
        // CPU 后端总是可用
        true
    }
}

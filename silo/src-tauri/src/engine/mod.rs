// 自适应混合推理引擎管理器
// Adaptive Hybrid Inference Engine Manager

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod backend;
pub mod manager;

pub use manager::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendType {
    /// Apple Silicon 优化后端 (MLX)
    MlxSidecar,
    /// PC/Server 端 (Inferflow 或 llama.cpp)
    InferflowCpp,
    /// 通用 CPU 后端 (llama.cpp)
    LlamaCppCpu,
    /// 蜂群模式 (分布式推理)
    Swarm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub model_path: PathBuf,
    pub backend: BackendType,
    pub context_size: usize,
    pub temperature: f32,
    pub top_p: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub tokens: Vec<String>,
    pub finish_reason: String,
}

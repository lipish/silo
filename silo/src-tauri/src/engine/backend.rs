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
// 目前使用模拟实现，后续可集成实际的 llama.cpp
pub struct LlamaCppBackend {
    initialized: bool,
    model_path: Option<std::path::PathBuf>,
}

impl LlamaCppBackend {
    pub fn new() -> Self {
        Self {
            initialized: false,
            model_path: None,
        }
    }
}

#[async_trait]
impl InferenceBackend for LlamaCppBackend {
    async fn initialize(&mut self, config: InferenceConfig) -> Result<()> {
        // 检查模型文件是否存在
        if !config.model_path.exists() {
            tracing::warn!("Model file not found: {:?}, using mock mode", config.model_path);
            self.initialized = true;
            return Ok(());
        }
        
        self.model_path = Some(config.model_path);
        self.initialized = true;
        tracing::info!("LlamaCppBackend initialized (mock mode)");
        Ok(())
    }
    
    async fn infer(&self, prompt: &str) -> Result<InferenceResponse> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Backend not initialized"));
        }
        
        // 模拟推理响应（实际应该调用 llama.cpp）
        // 这里返回一个简单的模拟响应，让应用可以运行
        let response_text = if prompt.contains("你好") || prompt.contains("hello") {
            "你好！我是 Silo AI，一个隐私优先的本地 Agent 操作系统。我可以帮助你完成各种任务，同时确保你的数据完全保留在本地。"
        } else if prompt.contains("帮助") || prompt.contains("help") {
            "我可以帮助你：\n1. 执行代码任务\n2. 搜索本地文档\n3. 管理文件\n4. 回答基于本地知识库的问题\n\n所有操作都在本地完成，确保数据隐私。"
        } else {
            &format!("我理解你的指令：\"{}\"。\n\n（注意：当前运行在模拟模式下。要使用真实的 AI 模型，请配置模型文件路径并集成 llama.cpp。）", prompt)
        };
        
        // 将响应分割成 tokens（模拟）
        let tokens: Vec<String> = response_text
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        
        Ok(InferenceResponse {
            tokens,
            finish_reason: "stop".to_string(),
        })
    }
    
    async fn infer_stream(&self, prompt: &str) -> Result<tokio::sync::mpsc::Receiver<String>> {
        // 创建通道用于流式输出
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        // 异步发送模拟的流式响应（需要拥有字符串）
        let response_text = if prompt.contains("你好") || prompt.contains("hello") {
            "你好！我是 Silo AI...".to_string()
        } else {
            format!("处理中：{}...", prompt)
        };
        
        tokio::spawn(async move {
            for word in response_text.split_whitespace() {
                if tx.send(format!("{} ", word)).await.is_err() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        });
        
        Ok(rx)
    }
    
    fn backend_type(&self) -> crate::engine::BackendType {
        crate::engine::BackendType::LlamaCppCpu
    }
    
    fn is_available(&self) -> bool {
        // CPU 后端总是可用
        true
    }
}

impl Default for LlamaCppBackend {
    fn default() -> Self {
        Self::new()
    }
}

// Silo AI - 隐私优先的本地 Agent 操作系统

mod agent;
mod engine;
mod sandbox;
mod swarm;
mod vault;

use agent::{AgentExecutor, AgentTask};
use engine::{EngineManager, InferenceConfig};
use sandbox::{SandboxConfig, SandboxExecutor};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use vault::{Document, VaultDatabase};

// 全局状态
pub struct AppState {
    pub engine: Arc<RwLock<EngineManager>>,
    pub vault: Arc<RwLock<VaultDatabase>>,
    pub sandbox: Arc<RwLock<SandboxExecutor>>,
    pub agent: Arc<RwLock<AgentExecutor>>,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        // 初始化推理引擎
        let mut engine = EngineManager::new();
        engine.detect_and_select_backend().await?;

        // 初始化向量数据库
        let vault_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("silo")
            .join("vault");
        std::fs::create_dir_all(&vault_path)?;
        let vault = VaultDatabase::new(vault_path)?;

        // 初始化沙箱
        let sandbox_config = SandboxConfig {
            memory_limit: 512 * 1024 * 1024, // 512MB
            timeout_seconds: 30,
            allowed_files: vec![],
        };
        let sandbox = SandboxExecutor::new(sandbox_config)?;

        // 初始化 Agent
        let engine_arc = Arc::new(RwLock::new(engine));
        let vault_arc = Arc::new(RwLock::new(vault));
        let sandbox_arc = Arc::new(RwLock::new(sandbox));

        let agent = AgentExecutor::new(
            engine_arc.clone(),
            vault_arc.clone(),
            sandbox_arc.clone(),
        );

        Ok(Self {
            engine: engine_arc,
            vault: vault_arc,
            sandbox: sandbox_arc,
            agent: Arc::new(RwLock::new(agent)),
        })
    }
}

// API - 供 GPUI 调用

pub async fn get_backend_type(state: &AppState) -> Result<String, String> {
    let engine = state.engine.read().await;
    Ok(format!("{:?}", engine.current_backend_type()))
}

pub async fn get_vault_stats(state: &AppState) -> Result<serde_json::Value, String> {
    let vault = state.vault.read().await;
    let count = vault.document_count().await;
    Ok(serde_json::json!({ "document_count": count }))
}

pub async fn execute_agent_task(
    state: &AppState,
    instruction: String,
    context: Option<String>,
) -> Result<serde_json::Value, String> {
    let task = AgentTask { instruction, context };
    let agent = state.agent.read().await;
    let response = agent.execute(task).await.map_err(|e: anyhow::Error| e.to_string())?;
    Ok(serde_json::to_value(response).unwrap())
}

pub async fn add_document(
    state: &AppState,
    content: String,
    file_path: Option<String>,
) -> Result<String, String> {
    let document = Document {
        id: uuid::Uuid::new_v4().to_string(),
        content,
        metadata: vault::DocumentMetadata {
            file_path: file_path.map(PathBuf::from),
            mime_type: None,
            created_at: chrono::Utc::now(),
            tags: vec![],
        },
    };

    let vault = state.vault.read().await;
    vault.add_document(document.clone()).await.map_err(|e| e.to_string())?;

    Ok(document.id)
}

fn infer_mime_type(file_path: &str) -> String {
    let ext = file_path
        .split('.')
        .last()
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "txt" | "md" | "markdown" => "text/plain",
        "json" => "application/json",
        "yaml" | "yml" => "text/yaml",
        "py" => "text/x-python",
        "rs" => "text/x-rust",
        "js" | "ts" | "jsx" | "tsx" => "text/javascript",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
    .to_string()
}

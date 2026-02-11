// Silo AI - 隐私优先的本地 Agent 操作系统

mod agent;
mod engine;
mod sandbox;
mod swarm;
mod vault;

use agent::{AgentExecutor, AgentTask};
use engine::{EngineManager, InferenceConfig};
use sandbox::{ExecutionResult, SandboxConfig, SandboxExecutor};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use vault::{Document, VaultDatabase};

// 全局状态
pub struct AppState {
    engine: Arc<RwLock<EngineManager>>,
    vault: Arc<RwLock<VaultDatabase>>,
    sandbox: Arc<RwLock<SandboxExecutor>>,
    agent: Arc<RwLock<AgentExecutor>>,
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
        
        // 初始化 Agent（需要 Arc 包装）
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

// Tauri Commands

#[tauri::command]
async fn initialize_engine(
    state: tauri::State<'_, AppState>,
    model_path: String,
    context_size: usize,
) -> Result<String, String> {
    let config = InferenceConfig {
        model_path: PathBuf::from(model_path),
        backend: engine::BackendType::LlamaCppCpu, // 默认，实际会由管理器选择
        context_size,
        temperature: 0.7,
        top_p: 0.9,
    };
    
    let mut engine = state.engine.write().await;
    engine
        .initialize(config)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok("Engine initialized".to_string())
}

#[tauri::command]
async fn infer(
    state: tauri::State<'_, AppState>,
    prompt: String,
) -> Result<String, String> {
    let engine = state.engine.read().await;
    let response = engine.infer(&prompt).await.map_err(|e| e.to_string())?;
    Ok(response.tokens.join(""))
}

#[tauri::command]
async fn execute_agent_task(
    state: tauri::State<'_, AppState>,
    instruction: String,
    context: Option<String>,
) -> Result<serde_json::Value, String> {
    let task = AgentTask { instruction, context };
    let agent = state.agent.read().await;
    let response = agent.execute(task).await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(response).unwrap())
}

#[tauri::command]
async fn add_document(
    state: tauri::State<'_, AppState>,
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

#[tauri::command]
async fn search_vault(
    state: tauri::State<'_, AppState>,
    query: String,
    limit: usize,
) -> Result<Vec<serde_json::Value>, String> {
    let vault = state.vault.read().await;
    let results = vault.search(&query, limit).await.map_err(|e| e.to_string())?;
    
    Ok(results
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect())
}

#[tauri::command]
async fn get_backend_type(
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let engine = state.engine.read().await;
    let backend_type = engine.current_backend_type();
    Ok(format!("{:?}", backend_type))
}

#[tauri::command]
async fn get_vault_stats(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let vault = state.vault.read().await;
    let count = vault.document_count().await;
    Ok(serde_json::json!({
        "document_count": count
    }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 初始化日志
            tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .init();
            
            let app_handle = app.handle().clone();
            
            tauri::async_runtime::spawn(async move {
                match AppState::new().await {
                    Ok(state) => {
                        app_handle.manage(state);
                        tracing::info!("Silo AI initialized successfully");
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize app state: {}", e);
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            initialize_engine,
            infer,
            execute_agent_task,
            add_document,
            search_vault,
            get_backend_type,
            get_vault_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

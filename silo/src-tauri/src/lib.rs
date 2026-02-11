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

#[tauri::command]
async fn read_file_content(
    file_path: String,
) -> Result<String, String> {
    use std::fs;
    
    let path = PathBuf::from(&file_path);
    
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    
    if !path.is_file() {
        return Err(format!("Path is not a file: {}", file_path));
    }
    
    // 检查文件大小（限制为 10MB）
    let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
    if metadata.len() > 10 * 1024 * 1024 {
        return Err("File too large (max 10MB)".to_string());
    }
    
    // 读取文件内容
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    
    tracing::info!("Read file: {} ({} bytes)", file_path, content.len());
    Ok(content)
}

#[tauri::command]
async fn add_file_to_vault(
    state: tauri::State<'_, AppState>,
    file_path: String,
) -> Result<String, String> {
    // 读取文件内容
    let content = read_file_content(file_path.clone()).await?;
    
    // 检测 MIME 类型
    let mime_type = infer_mime_type(&file_path);
    
    // 添加到 Vault
    let document = Document {
        id: uuid::Uuid::new_v4().to_string(),
        content,
        metadata: vault::DocumentMetadata {
            file_path: Some(PathBuf::from(&file_path)),
            mime_type: Some(mime_type),
            created_at: chrono::Utc::now(),
            tags: vec![],
        },
    };
    
    let vault = state.vault.read().await;
    vault.add_document(document.clone()).await.map_err(|e| e.to_string())?;
    
    tracing::info!("Added file to vault: {} -> {}", file_path, document.id);
    Ok(document.id)
}

#[tauri::command]
async fn list_vault_documents(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let vault = state.vault.read().await;
    let docs = vault.list_all_documents().await.map_err(|e| e.to_string())?;
    
    Ok(docs
        .into_iter()
        .map(|doc| serde_json::to_value(doc).unwrap())
        .collect())
}

/// 根据文件扩展名推断 MIME 类型
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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
            get_vault_stats,
            read_file_content,
            add_file_to_vault,
            list_vault_documents
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

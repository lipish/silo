// Agent 执行器 - 协调推理、检索和执行

use crate::agent::executor::AgentExecutor;
use crate::engine::EngineManager;
use crate::sandbox::SandboxExecutor;
use crate::vault::VaultDatabase;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod executor;
pub mod utils;

pub use executor::*;
pub use utils::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub instruction: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub reasoning: String,
    pub actions: Vec<AgentAction>,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentAction {
    FileOperation { path: String, operation: String },
    CodeExecution { code: String, language: String },
    SearchQuery { query: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub content: String,
    pub mime_type: String,
}

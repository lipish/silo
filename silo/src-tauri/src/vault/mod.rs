// Silo Vault - 隐私数据地窖
// 基于 LanceDB 的向量数据库，支持长上下文和 P2P 同步

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod database;
pub mod sync;
pub mod chunker;

pub use database::*;
pub use chunker::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub file_path: Option<PathBuf>,
    pub mime_type: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: Document,
    pub similarity: f32,
}

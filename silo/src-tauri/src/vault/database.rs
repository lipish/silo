// LanceDB 向量数据库封装
// 目前使用内存存储作为临时实现，后续集成 LanceDB

use crate::vault::{Document, SearchResult};
use anyhow::Result;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct VaultDatabase {
    db_path: PathBuf,
    // 临时使用内存存储，后续替换为 LanceDB
    documents: Arc<RwLock<HashMap<String, Document>>>,
}

impl VaultDatabase {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&db_path)?;
        tracing::info!("VaultDatabase initialized at: {:?}", db_path);
        Ok(Self {
            db_path,
            documents: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// 添加文档到向量库
    pub async fn add_document(&self, document: Document) -> Result<()> {
        let mut docs = self.documents.write().await;
        docs.insert(document.id.clone(), document.clone());
        tracing::info!("Added document to vault: {} ({} bytes)", document.id, document.content.len());
        Ok(())
    }
    
    /// 搜索相似文档（简单文本匹配，后续替换为向量搜索）
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let docs = self.documents.read().await;
        let query_lower = query.to_lowercase();
        
        let mut results: Vec<SearchResult> = docs
            .values()
            .filter_map(|doc| {
                let content_lower = doc.content.to_lowercase();
                if content_lower.contains(&query_lower) {
                    // 简单的相似度计算（关键词匹配数量）
                    let similarity = query_lower
                        .split_whitespace()
                        .filter(|word| content_lower.contains(word))
                        .count() as f32 / query_lower.split_whitespace().count().max(1) as f32;
                    
                    Some(SearchResult {
                        document: doc.clone(),
                        similarity,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        // 按相似度排序
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        tracing::info!("Search for '{}' returned {} results", query, results.len());
        Ok(results)
    }
    
    /// 获取文档
    pub async fn get_document(&self, id: &str) -> Result<Option<Document>> {
        let docs = self.documents.read().await;
        Ok(docs.get(id).cloned())
    }
    
    /// 删除文档
    pub async fn delete_document(&self, id: &str) -> Result<()> {
        let mut docs = self.documents.write().await;
        docs.remove(id);
        tracing::info!("Deleted document from vault: {}", id);
        Ok(())
    }
    
    /// 获取所有文档数量
    pub async fn document_count(&self) -> usize {
        let docs = self.documents.read().await;
        docs.len()
    }
}

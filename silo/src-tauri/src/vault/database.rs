// LanceDB 向量数据库封装
// 目前使用内存存储作为临时实现，后续集成 LanceDB

use crate::vault::{Document, SearchResult, DocumentChunker};
use anyhow::Result;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct DocumentChunk {
    id: String,
    document_id: String,
    content: String,
    chunk_index: usize,
    // 简单的特征向量（词频向量）
    features: Vec<f32>,
}

pub struct VaultDatabase {
    db_path: PathBuf,
    // 临时使用内存存储，后续替换为 LanceDB
    documents: Arc<RwLock<HashMap<String, Document>>>,
    chunks: Arc<RwLock<HashMap<String, Vec<DocumentChunk>>>>,
    chunker: DocumentChunker,
}

impl VaultDatabase {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&db_path)?;
        tracing::info!("VaultDatabase initialized at: {:?}", db_path);
        Ok(Self {
            db_path,
            documents: Arc::new(RwLock::new(HashMap::new())),
            chunks: Arc::new(RwLock::new(HashMap::new())),
            chunker: DocumentChunker::default(),
        })
    }
    
    /// 生成简单的文本特征向量（词频向量）
    fn extract_features(text: &str) -> Vec<f32> {
        use std::collections::HashMap;
        
        let words: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|w| w.chars().filter(|c| c.is_alphanumeric()).collect())
            .filter(|w: &String| !w.is_empty())
            .collect();
        
        let total_words = words.len().max(1);
        let mut word_freq: HashMap<String, usize> = HashMap::new();
        
        for word in &words {
            *word_freq.entry(word.clone()).or_insert(0) += 1;
        }
        
        // 创建固定大小的特征向量（使用最常见的词）
        let mut features: Vec<(String, f32)> = word_freq
            .into_iter()
            .map(|(word, count)| (word, count as f32 / total_words as f32))
            .collect();
        
        features.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        features.truncate(100); // 取前 100 个最常见的词
        
        // 归一化
        let sum: f32 = features.iter().map(|(_, f)| f * f).sum::<f32>().sqrt().max(1e-10);
        features.iter().map(|(_, f)| f / sum).collect()
    }
    
    /// 计算两个特征向量的余弦相似度
    fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
        if v1.is_empty() || v2.is_empty() {
            return 0.0;
        }
        
        let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = v1.iter().map(|a| a * a).sum::<f32>().sqrt();
        let norm2: f32 = v2.iter().map(|a| a * a).sum::<f32>().sqrt();
        
        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm1 * norm2)
    }
    
    /// 添加文档到向量库（自动分块和向量化）
    pub async fn add_document(&self, document: Document) -> Result<()> {
        // 存储完整文档
        let mut docs = self.documents.write().await;
        docs.insert(document.id.clone(), document.clone());
        drop(docs);
        
        // 分块处理
        let chunks = self.chunker.chunk_by_paragraphs(&document.content);
        let mut document_chunks = Vec::new();
        
        for (idx, chunk_text) in chunks.iter().enumerate() {
            let features = Self::extract_features(chunk_text);
            let chunk = DocumentChunk {
                id: format!("{}_chunk_{}", document.id, idx),
                document_id: document.id.clone(),
                content: chunk_text.clone(),
                chunk_index: idx,
                features,
            };
            document_chunks.push(chunk);
        }
        
        // 存储分块
        let mut chunks_map = self.chunks.write().await;
        chunks_map.insert(document.id.clone(), document_chunks);
        
        tracing::info!("Added document to vault: {} ({} bytes, {} chunks)", 
            document.id, document.content.len(), chunks.len());
        Ok(())
    }
    
    /// 搜索相似文档（使用特征向量相似度）
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let query_features = Self::extract_features(query);
        let chunks_map = self.chunks.read().await;
        let docs = self.documents.read().await;
        
        // 计算每个文档的最高相似度（基于最佳匹配块）
        let mut doc_similarities: Vec<(String, f32, String)> = Vec::new();
        
        for (doc_id, chunks) in chunks_map.iter() {
            let mut max_similarity = 0.0;
            let mut best_chunk_content = String::new();
            
            for chunk in chunks {
                let similarity = Self::cosine_similarity(&query_features, &chunk.features);
                if similarity > max_similarity {
                    max_similarity = similarity;
                    best_chunk_content = chunk.content.clone();
                }
            }
            
            if max_similarity > 0.1 { // 阈值过滤
                doc_similarities.push((doc_id.clone(), max_similarity, best_chunk_content));
            }
        }
        
        // 按相似度排序
        doc_similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        doc_similarities.truncate(limit);
        
        // 构建搜索结果
        let results: Vec<SearchResult> = doc_similarities
            .into_iter()
            .filter_map(|(doc_id, similarity, _)| {
                docs.get(&doc_id).map(|doc| SearchResult {
                    document: doc.clone(),
                    similarity,
                })
            })
            .collect();
        
        tracing::info!("Search for '{}' returned {} results (best similarity: {:.3})", 
            query, results.len(), results.first().map(|r| r.similarity).unwrap_or(0.0));
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
    
    /// 列出所有文档
    pub async fn list_all_documents(&self) -> Result<Vec<Document>> {
        let docs = self.documents.read().await;
        Ok(docs.values().cloned().collect())
    }
}

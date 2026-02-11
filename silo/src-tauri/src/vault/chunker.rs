// 文档分块器 - 将长文档分割成适合向量化的块

use anyhow::Result;

pub struct DocumentChunker {
    chunk_size: usize,
    chunk_overlap: usize,
}

impl DocumentChunker {
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
    
    /// 将文本分割成块
    pub fn chunk_text(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        if words.is_empty() {
            return chunks;
        }
        
        let mut start = 0;
        while start < words.len() {
            let end = (start + self.chunk_size).min(words.len());
            let chunk = words[start..end].join(" ");
            chunks.push(chunk);
            
            if end >= words.len() {
                break;
            }
            
            // 重叠处理
            start = end.saturating_sub(self.chunk_overlap);
        }
        
        chunks
    }
    
    /// 按段落分割（更智能的分块方式）
    pub fn chunk_by_paragraphs(&self, text: &str) -> Vec<String> {
        let paragraphs: Vec<&str> = text
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();
        
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        
        for paragraph in paragraphs {
            let para_words: Vec<&str> = paragraph.split_whitespace().collect();
            
            if current_chunk.split_whitespace().count() + para_words.len() > self.chunk_size {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.trim().to_string());
                }
                current_chunk = paragraph.to_string();
            } else {
                if !current_chunk.is_empty() {
                    current_chunk.push_str("\n\n");
                }
                current_chunk.push_str(paragraph);
            }
        }
        
        if !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }
        
        chunks
    }
}

impl Default for DocumentChunker {
    fn default() -> Self {
        Self::new(500, 50) // 默认 500 词，50 词重叠
    }
}

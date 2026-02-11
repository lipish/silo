// Agent 执行器实现

use crate::agent::{AgentAction, AgentResponse, AgentTask, Artifact, extract_keywords, extract_search_query, extract_code_block};
use crate::engine::EngineManager;
use crate::sandbox::SandboxExecutor;
use crate::vault::VaultDatabase;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AgentExecutor {
    engine: Arc<RwLock<EngineManager>>,
    vault: Arc<RwLock<VaultDatabase>>,
    sandbox: Arc<RwLock<SandboxExecutor>>,
}

impl AgentExecutor {
    pub fn new(
        engine: Arc<RwLock<EngineManager>>,
        vault: Arc<RwLock<VaultDatabase>>,
        sandbox: Arc<RwLock<SandboxExecutor>>,
    ) -> Self {
        Self {
            engine,
            vault,
            sandbox,
        }
    }
    
    /// 执行 Agent 任务
    pub async fn execute(&self, task: AgentTask) -> Result<AgentResponse> {
        // 1. 从 Vault 检索相关上下文（自动提取关键词）
        let context_query = if let Some(query) = &task.context {
            query.clone()
        } else {
            // 从指令中提取关键词作为搜索查询
            extract_keywords(&task.instruction)
        };
        
        let vault = self.vault.read().await;
        let context = vault.search(&context_query, 5).await?;
        drop(vault);
        
        // 2. 构建增强提示词
        let enhanced_prompt = self.build_enhanced_prompt(&task, &context);
        
        // 3. 调用推理引擎
        let engine = self.engine.read().await;
        let response = engine.infer(&enhanced_prompt).await?;
        let reasoning = response.tokens.join("");
        
        // 4. 解析 Agent 动作（改进的解析逻辑）
        let actions = self.parse_actions(&reasoning, &task.instruction).await?;
        
        // 5. 执行动作（需要用户确认）
        let artifacts = self.execute_actions(actions.clone()).await?;
        
        Ok(AgentResponse {
            reasoning,
            actions,
            artifacts,
        })
    }
    
    
    fn build_enhanced_prompt(&self, task: &AgentTask, context: &[crate::vault::SearchResult]) -> String {
        let mut prompt = format!("你是一个本地 AI Agent，名为 Silo。你的任务是帮助用户完成各种任务，同时确保所有操作都在本地完成，保护用户隐私。\n\n");
        prompt.push_str(&format!("用户指令: {}\n\n", task.instruction));
        
        if !context.is_empty() {
            prompt.push_str("相关上下文（来自本地知识库）:\n");
            for (idx, result) in context.iter().enumerate() {
                let preview = if result.document.content.len() > 200 {
                    format!("{}...", &result.document.content[..200])
                } else {
                    result.document.content.clone()
                };
                prompt.push_str(&format!("[文档 {}] (相似度: {:.2})\n{}\n\n", idx + 1, result.similarity, preview));
            }
        }
        
        prompt.push_str("请分析任务并给出执行计划。如果需要执行代码、搜索文档或操作文件，请明确说明。");
        prompt
    }
    
    async fn parse_actions(&self, reasoning: &str, instruction: &str) -> Result<Vec<AgentAction>> {
        let mut actions = Vec::new();
        
        // 简单的动作解析（基于关键词匹配）
        let reasoning_lower = reasoning.to_lowercase();
        let instruction_lower = instruction.to_lowercase();
        
        // 检测搜索查询
        if reasoning_lower.contains("搜索") || reasoning_lower.contains("查找") || reasoning_lower.contains("search") {
            // 尝试从指令中提取搜索关键词
            let query = extract_search_query(instruction);
            if !query.is_empty() {
                actions.push(AgentAction::SearchQuery { query });
            }
        }
        
        // 检测代码执行需求
        if reasoning_lower.contains("代码") || reasoning_lower.contains("执行") || reasoning_lower.contains("运行") ||
           instruction_lower.contains("python") || instruction_lower.contains("代码") {
            // 尝试提取代码片段
            if let Some(code) = extract_code_block(reasoning) {
                actions.push(AgentAction::CodeExecution {
                    code,
                    language: "python".to_string(),
                });
            }
        }
        
        // 检测文件操作
        if reasoning_lower.contains("文件") || reasoning_lower.contains("重命名") || reasoning_lower.contains("移动") ||
           instruction_lower.contains("文件") {
            // 这里需要更复杂的解析，暂时跳过
            // actions.push(AgentAction::FileOperation { ... });
        }
        
        Ok(actions)
    }
    
    async fn execute_actions(&self, actions: Vec<AgentAction>) -> Result<Vec<Artifact>> {
        let mut artifacts = vec![];
        
        for action in actions {
            match action {
                AgentAction::CodeExecution { code, language } => {
                    let sandbox = self.sandbox.read().await;
                    let result = sandbox.execute(&code, &language).await?;
                    artifacts.push(Artifact {
                        content: result.stdout,
                        mime_type: "text/plain".to_string(),
                    });
                }
                AgentAction::FileOperation { .. } => {
                    // TODO: 需要用户确认权限
                    tracing::warn!("File operation requires user confirmation");
                }
                AgentAction::SearchQuery { query } => {
                    let vault = self.vault.read().await;
                    let results = vault.search(&query, 10).await?;
                    let mut content = format!("找到 {} 个相关结果:\n\n", results.len());
                    for (idx, result) in results.iter().enumerate() {
                        let preview = if result.document.content.len() > 150 {
                            format!("{}...", &result.document.content[..150])
                        } else {
                            result.document.content.clone()
                        };
                        content.push_str(&format!("[{}] (相似度: {:.2})\n{}\n\n", 
                            idx + 1, result.similarity, preview));
                    }
                    artifacts.push(Artifact {
                        content,
                        mime_type: "text/markdown".to_string(),
                    });
                }
            }
        }
        
        Ok(artifacts)
    }
}

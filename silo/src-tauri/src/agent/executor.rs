// Agent 执行器实现

use crate::agent::{AgentAction, AgentResponse, AgentTask, Artifact};
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
        // 1. 从 Vault 检索相关上下文
        let context = if let Some(query) = &task.context {
            let vault = self.vault.read().await;
            vault.search(query, 5).await?
        } else {
            vec![]
        };
        
        // 2. 构建增强提示词
        let enhanced_prompt = self.build_enhanced_prompt(&task, &context);
        
        // 3. 调用推理引擎
        let engine = self.engine.read().await;
        let response = engine.infer(&enhanced_prompt).await?;
        
        // 4. 解析 Agent 动作
        let actions = self.parse_actions(&response.tokens.join("")).await?;
        
        // 5. 执行动作（需要用户确认）
        let artifacts = self.execute_actions(actions.clone()).await?;
        
        Ok(AgentResponse {
            reasoning: response.tokens.join(""),
            actions,
            artifacts,
        })
    }
    
    fn build_enhanced_prompt(&self, task: &AgentTask, context: &[crate::vault::SearchResult]) -> String {
        let mut prompt = format!("Task: {}\n\n", task.instruction);
        
        if !context.is_empty() {
            prompt.push_str("Relevant Context:\n");
            for result in context {
                prompt.push_str(&format!("- {}\n", result.document.content));
            }
            prompt.push_str("\n");
        }
        
        prompt.push_str("Please analyze the task and provide a plan with actions.");
        prompt
    }
    
    async fn parse_actions(&self, _reasoning: &str) -> Result<Vec<AgentAction>> {
        // TODO: 使用 LLM 解析动作（或使用结构化输出）
        Ok(vec![])
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
                    artifacts.push(Artifact {
                        content: format!("Found {} results", results.len()),
                        mime_type: "text/markdown".to_string(),
                    });
                }
            }
        }
        
        Ok(artifacts)
    }
}

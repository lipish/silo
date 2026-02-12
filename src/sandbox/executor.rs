// 代码执行器：使用系统解释器执行，后续可集成 Wasmtime 沙箱

use crate::sandbox::{ExecutionResult, SandboxConfig};
use anyhow::Result;
use tokio::process::Command;

pub struct SandboxExecutor {
    config: SandboxConfig,
}

impl SandboxExecutor {
    pub fn new(config: SandboxConfig) -> Result<Self> {
        tracing::info!("SandboxExecutor initialized (memory_limit: {}MB, timeout: {}s)", 
            config.memory_limit / 1024 / 1024, config.timeout_seconds);
        Ok(Self { config })
    }
    
    /// 执行代码：Python 使用系统 python3，其他语言暂时模拟
    pub async fn execute(&self, code: &str, language: &str) -> Result<ExecutionResult> {
        tracing::info!("Executing {} code ({} bytes)", language, code.len());
        
        let result = match language.to_lowercase().as_str() {
            "python" => self.execute_python(code).await?,
            "javascript" | "js" => {
                ExecutionResult {
                    stdout: format!("JavaScript 执行尚需集成，当前为模拟模式"),
                    stderr: String::new(),
                    exit_code: 0,
                }
            }
            _ => {
                ExecutionResult {
                    stdout: format!("{} 执行尚需集成", language),
                    stderr: String::new(),
                    exit_code: 0,
                }
            }
        };
        
        Ok(result)
    }

    async fn execute_python(&self, code: &str) -> Result<ExecutionResult> {
        let output = Command::new("python3")
            .arg("-c")
            .arg(code)
            .current_dir(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")))
            .output()
            .await;

        match output {
            Ok(out) => Ok(ExecutionResult {
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                exit_code: out.status.code().unwrap_or(-1),
            }),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(ExecutionResult {
                        stdout: String::new(),
                        stderr: format!("未找到 python3，请确保已安装并加入 PATH"),
                        exit_code: 1,
                    })
                } else {
                    Ok(ExecutionResult {
                        stdout: String::new(),
                        stderr: format!("执行失败: {}", e),
                        exit_code: 1,
                    })
                }
            }
        }
    }
}

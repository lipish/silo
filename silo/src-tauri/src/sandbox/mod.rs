// Agent 执行沙箱 - 基于 Wasmtime 的安全代码执行

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod executor;

pub use executor::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub memory_limit: usize,
    pub timeout_seconds: u64,
    pub allowed_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

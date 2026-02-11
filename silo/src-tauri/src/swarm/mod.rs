// Silo Swarm - 蜂群模式，局域网算力聚合

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod node;
pub mod discovery;

pub use node::*;
pub use discovery::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmNode {
    pub peer_id: String,  // 临时使用 String，等启用 libp2p 时可能需要调整
    pub address: String,
    pub capabilities: NodeCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub gpu_memory_gb: Option<u32>,
    pub cpu_cores: u32,
    pub available: bool,
}

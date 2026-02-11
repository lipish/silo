# Silo AI 产品与架构设计文档

## 市场定位分析

基于现有资料，虽然无法直接检索到关于 "Openclew" 的具体信息（可能是非常新的项目、拼写偏误或构想中的名称），但结合**「隐私优先」和「多平台部署」**这两个定位，以及目前市场上 Jan、MLC-LLM 和 Ollama 的竞争格局，可以明确：完全有机会，但不能只做「另一个聊天壳」。

目前的市场在「让模型跑起来」这一步已经很拥挤，但在「让模型在多端协同工作」和「深度隐私场景」上仍有巨大缺口。

### 1. 核心定位分析：「隐私优先 + 多平台」是红海还是蓝海？

目前的「隐私优先」市场由 Jan 和 Ollama 主导，但它们都有明显的局限性，这正是新工具的机会。

**Jan 的局限（你的机会）**：Jan 定位为「100% 离线的 ChatGPT 替代品」，主打简单的聊天体验。

- **缺口**：它更像是一个单机玩具。如果你能做一个**「隐私优先的生产力同步工具」，不仅支持桌面（Win/Mac/Linux），还能支持 iOS/Android，并实现本地数据的端到端加密同步**（类似 Obsidian + AI），这将是一个巨大的差异化卖点。

**Ollama 的局限（你的机会）**：Ollama 是命令行优先（CLI-first），虽然有 WebUI 生态，但对普通用户不够友好，且多平台支持主要集中在桌面端。

- **缺口**：Ollama 在移动端的部署非常薄弱。利用 MLC-LLM 的底层技术（支持 iOS/Android SDK）打造一个真正的「全平台本地 AI 运行时」，让用户在手机上跑的小模型能无缝接管电脑上的任务，是目前的市场空白。

### 2. 形态创新：从「推理引擎」转型为「多功能隐私工作台」

如果只是做一个推理引擎（Inference Engine），很难打败 llama.cpp 或 MLX。机会在于做一个集成应用的「操作系统」。

**A. 真正的「多平台」：打通桌面与移动端**

目前的工具大多是「桌面端孤岛」。

- **机会**：参考 MLC-LLM 的架构，它已经实现了在 iPad、iPhone 和 Android 上运行高性能模型。
- **应用场景**：做一个跨端 App，电脑上用 70B 模型处理重任务，手机上用 3B 量化模型处理即时回复，两者通过局域网或加密通道共享知识库（RAG），而数据永远不出本地。

**B. 隐私场景下的「团队协作」**

目前的本地 AI 工具都是「单人版」。

- **痛点**：企业或小团队想用 AI，但不敢传云端，每个人都在自己电脑上装 Ollama，显存浪费且知识无法共享。
- **新形态**：开发一个支持**「局域网推理集群」**的应用。
  - 利用 Inferflow 提出的**混合分区（Hybrid Partitioning）**技术，让团队内几台闲置的消费级显卡（如几台带 GPU 的笔记本）联合起来推理一个大模型。
  - 这解决了单机显存不足的问题，同时保持了隐私优先（数据不公网）。

### 3. 技术壁垒：利用新特性建立优势

要在技术上站住脚，不能只套壳 llama.cpp。资料显示有几个前沿方向值得整合：

**更极致的量化**：

- Inferflow 引入了 3.5-bit 量化，这是一个很好的平衡点。相比 3-bit 精度损失小，相比 4-bit 显存占用更低。针对桌面端常见的 16GB/24GB 内存环境，这种细粒度的优化能让用户跑起参数更大的模型。

**针对 Apple Silicon 的深度优化**：

- 虽然 MLX 在 Mac 上吞吐量无敌（~230 tokens/s），但它的首字延迟（TTFT）不如 MLC-LLM。
- **策略**：App 可以根据硬件自动切换后端——在 Mac 上调用 MLX 或 MLC 的库，在 PC 上调用针对 Intel/AMD NPU 优化的 OpenVINO 或 ONNX，而不是全盘依赖通用的 llama.cpp。

### 4. 总结：赢面在哪里？

如果定位为 Openclew（或类似新形态），建议避开「模型启动器」的内卷，主攻以下三个方向之一：

1. **Privacy-Sync（隐私同步）**：主打多端（含移动端），数据完全本地/P2P 同步，不仅是聊天，更是跨设备的内容生成与管理。
2. **Local-Team（本地团队版）**：主打局域网算力聚合，让办公室里的多台电脑组成一个私有算力池，服务于团队。
3. **Agent-OS（智能体系统）**：不仅是推理，而是给予 AI 操作本地文件和软件的权限（类似本地版的 "Computer Use"），做真正的桌面自动化助手。

**一句话建议**：现在的用户不缺一个「能对话的本地窗口」，缺的是一个「能协同、能干活且不泄密的本地工作流」。

---

## 分层架构设计建议

针对 OpenClaw 定位为「本地 Agent-OS / 隐私优先 / 协同工作流」，结合最新的技术趋势（Inferflow, MLX, MLC-LLM 等），建议采用以下分层架构设计：

### 1. 核心引擎层：拒绝「单打独斗」，采用「自适应混合后端」

**针对 Apple Silicon 引入 MLX 原生支持**：

- 在 Mac 设备上，不要只用 llama.cpp。研究表明，MLX 在 M2 Ultra 等芯片上的吞吐量（~230 tokens/s）远高于 Ollama（~35 tokens/s）和 llama.cpp。
- **优势**：MLX 提供极其稳定的每 token 延迟（latency），这对于 Agent 需要快速连续调用工具（Tool Calling）的场景至关重要。

**引入「混合模型分区」(Hybrid Model Partitioning)**：

- 参考 Inferflow 的设计，支持将大模型切分到不同的硬件上运行。
- **场景**：用户的笔记本显存不够跑 70B 模型？可以自动检测局域网内的另一台闲置 PC，利用 Hybrid Partition 技术，将模型的一部分层（Layer）或张量（Tensor）卸载到那台 PC 上运行，实现「局域网算力众筹」。

**引入 3.5-bit 极致量化**：

- 集成类似 Inferflow 的 3.5-bit 量化技术。
- **优势**：3-bit 往往太傻，4-bit 显存占用又稍大。3.5-bit 是平衡点，能让用户在 16GB/24GB 的常见显存配置下跑参数更大、更聪明的模型，这对 Agent 的逻辑推理能力至关重要。

### 2. 数据与上下文层：打造「不仅不泄密，而且能记住」的第二大脑

**分页 KV 缓存 (Paged KV Cache)**：

- 必须实现类似 MLC-LLM 或 vLLM 的分页 KV 缓存机制。
- **痛点解决**：普通的本地推理在处理长文档（如读取整个代码库）时，显存会爆炸且速度极慢。Paged KV Cache 能高效管理 100k+ token 的上下文，让 Agent 真正「读懂」你的所有本地文件。

**隐私优先的 P2P 知识同步**：

- 设计一种基于局域网或加密 P2P（类似 Syncthing 协议）的向量数据库同步机制。
- **场景**：团队里的三个人都在本地跑，A 整理的项目文档进了本地知识库，B 和 C 可以通过局域网授权直接检索 A 的知识库，数据从未离开局域网。

### 3. Agent 工作流层：从「对话」进化为「操作系统」

**系统级工具链 (Native Tool Use)**：

- 直接集成系统级 API。目前的 Ollama 虽然支持 API，但本身不具备执行能力。应该内置一个安全的 Python 沙箱或 Shell 执行器。
- **能力**：允许 Agent 直接执行「查找过去一周所有发票 PDF 并压缩」这样的指令，而不仅仅是告诉你怎么写这段 Python 代码。

**多端协同 (Local Federation)**：

- 参考 MLC-LLM 的跨平台能力（iOS/Android/Web）。
- **场景**：「手机下指令，电脑跑推理」。用户在手机端语音输入任务，任务通过局域网发送到性能强大的桌面端处理，处理完的结果（如生成的报表）再推回手机。

### 4. 商业/产品差异化总结

| 功能模块 | 传统竞品 (LM Studio/Ollama) | 建议设计 |
| --- | --- | --- |
| 推理后端 | 单一 llama.cpp，通用但非极致 | 自适应后端：Mac 用 MLX (高吞吐)，PC/移动端用 MLC (高并发) |
| 协同方式 | 无，单机单人 | 局域网算力集群：Hybrid Partitioning 连接多设备 |
| 上下文能力 | 依赖内存硬抗，长文易崩 | Paged KV Cache：流畅处理 100k+ 上下文 |
| 隐私策略 | 本地运行 | 隐私同步：P2P 向量库同步，团队知识共享不上云 |
| 交互形态 | 聊天窗口 (Chat UI) | Agent OS：集成文件系统权限，能执行操作的智能体 |

**一句话总结设计思路**：不应再造一个「本地 ChatGPT」，而应打造一个**「分布式的本地 AI 操作系统」**。利用 MLX 榨干单机性能，利用 Inferflow 的思路榨干局域网算力，利用 Agent 权限接管繁琐工作，最终实现「数据不离本地，算力无处不在」。

---

# Silo AI: 产品设计白皮书

**核心理念**：Your Data's Fortress. (你数据的堡垒)

**定位**：隐私优先的本地 Agent 操作系统，支持多端算力聚合与协作。

## 1. 核心架构设计 (The Architecture)

Silo 不仅仅是一个聊天客户端，它是一个**「本地分布式推理操作系统」**。它由三层架构组成：

### A. 底层：自适应混合推理引擎 (Adaptive Hybrid Engine)

目前的竞品（如 LM Studio）通常只封装通用的 llama.cpp。Silo 将根据硬件环境动态切换最优后端，并引入「算力众筹」机制。

**针对 Apple Silicon 的原生优化**：

- **策略**：在 Mac 上直接调用 MLX 框架，而不是 llama.cpp。
- **依据**：研究显示，在 M2 Ultra 等芯片上，MLX 的吞吐量（~230 tokens/s）远高于 Ollama/llama.cpp（~35 tokens/s），且提供极其稳定的每 token 延迟，这对 Agent 的流畅执行至关重要。

**针对 PC/Server 的 Inferflow 集成**：

- **策略**：集成 Inferflow 引擎，支持 3.5-bit 量化。
- **依据**：3-bit 精度太低，4-bit 显存占用高。3.5-bit 是完美的平衡点，能让 24GB 显存的消费级显卡跑起来更聪明的模型（如 Llama-3-70B 的特定量化版）。

**Silo Swarm (蜂群模式)**：

- **功能**：局域网算力聚合。
- **技术实现**：利用 Inferflow 的混合模型分区 (Hybrid Model Partitioning) 技术。
- **场景**：你在开会，只带了轻薄本（算力弱），但你的主力台式机在工位上开着。Silo 允许笔记本作为「控制端」，通过局域网调用台式机的 GPU 进行推理，或者将模型的一半层（Layers）放在台式机跑，另一半在笔记本跑。

### B. 数据层：Silo Vault (数据地窖)

这是 Silo 的核心差异化——数据不仅不出公网，而且在你的授信设备间加密流动。

**Paged KV Cache (分页上下文缓存)**：

- **痛点**：普通本地模型读长文档（RAG）很慢，且容易爆显存。
- **方案**：引入类似 MLC-LLM 或 vLLM 的分页缓存技术，支持 100k+ token 的长上下文，让 Silo 能真正「读懂」整个项目的代码库或几百份 PDF。

**Syncthing 协议集成的向量库**：

- **设计**：内置一个基于文件的向量数据库（如 LanceDB），通过修改版的 Syncthing 协议在局域网或 P2P 加密通道内同步。
- **体验**：你在台式机上「喂」给 Silo 的文档，哪怕断网，你的手机端 Silo 也能通过蓝牙/局域网同步索引，并进行问答。

### C. 应用层：The Workbench (工作台)

拒绝单一的「聊天气泡」界面，采用「双栏工作台」设计。

- **左侧**：Agent 对话与指令流。
- **右侧**：实时生成的 Live Artifacts（实时工件）。
- **功能**：类似 Claude 的 Artifacts，但完全本地渲染。支持预览生成的 HTML、Python 代码运行结果（内置沙箱）、Markdown 文档。

---

## 2. 关键功能场景 (User Scenarios)

### 场景一：隐私敏感的「本地特工」(The Local Agent)

用户不再是和 Silo 聊天，而是指派任务。

- **用户指令**：「Silo，扫描我的 ~/Downloads 文件夹，找到上个月所有的发票 PDF，重命名为 日期_金额_公司.pdf，然后归档到 ~/Finance。」
- **Silo 执行**：
  1. 调用本地 Python 沙箱（基于 Docker 或 Wasm）。
  2. 编写 OCR 脚本读取 PDF。
  3. 在执行文件操作前，弹窗请求用户确认权限（这是安全的关键）。
  4. 执行并反馈结果。
- **竞品对比**：LM Studio 做不到（无系统权限），Ollama 需要用户自己写代码调用 API。Silo 直接内置了这个能力。

### 场景二：团队协作的「局域网大脑」(The Team Brain)

- **背景**：一个 5 人的法律或研发小团队，禁止使用云端 AI。
- **Silo 方案**：
  - 一台配有双 4090 的 PC 作为 Silo Node (主节点)。
  - 5 名员工的笔记本安装 Silo Client。
  - **知识共享**：所有员工上传的案件文档，自动在局域网内向量化并加密同步。A 员工问：「上周 B 员工传的那个合同里，关于赔偿的条款是怎么写的？」Silo 能直接通过局域网检索回答。
- **技术支撑**：Inferflow 的高吞吐和 Hybrid Partitioning 保证多人并发时的响应速度。

---

## 3. UI/UX 设计风格

Silo 的视觉语言应体现「安全」、「工业级」和「极简」。

- **Logo**：一个极简的混凝土圆柱体（筒仓）俯视图，或者一个坚固的六边形结构。
- **色调**：
  - **深色模式（默认）**：Deep Charcoal (深炭灰) 背景，配合 Electric Amber (琥珀黄) 或 Signal Orange (信号橙) 作为高亮色——致敬工业控制台和终端机。
  - **字体**：等宽字体（Monospace）用于代码和数据展示，强调专业性。
- **交互细节**：
  - **物理开关感**：开启「联网搜索」或「执行系统命令」时，设计类似物理开关的 UI 动效，暗示「权限的授予是严肃的」。
  - **数据流可视化**：在进行局域网推理（Swarm 模式）时，在界面底部显示简单的拓扑图，展示算力正从哪台设备流向哪台设备。

---

## 4. 商业化与发布路线图 (Roadmap)

**阶段 1：Silo Solo (MVP)**

- **目标**：打造最强的单机 Agent 启动器。
- **功能**：集成 MLX (Mac) 和 Inferflow (PC)，支持 3.5-bit 量化，内置 Python 沙箱，支持本地文件操作。
- **定价**：免费开源（核心引擎），通过 Github 建立社区声誉。

**阶段 2：Silo Sync (Pro)**

- **目标**：多设备个人用户。
- **功能**：推出移动端 App (iOS/Android，基于 MLC-LLM)，实现 PC 与手机的 P2P 知识库同步与算力接力。
- **定价**：一次性买断或订阅制（针对高级同步功能）。

**阶段 3：Silo Team (Enterprise)**

- **目标**：隐私敏感的小微企业。
- **功能**：局域网算力池（Swarm 模式）、团队共享向量库、权限管理。
- **定价**：按节点收费，提供部署服务。

---

**总结**：Silo 不做「第 101 个 Llama 启动器」。它要做的是 Local AI OS。它利用 Inferflow 的技术让算力流动，利用 MLX 榨干硬件性能，利用 P2P 协议保护数据主权。

---

# Silo AI (Silo OS) 技术架构与设计文档 v1.0

**项目代号**：`Silo`

**核心定位**：隐私优先的本地 Agent 操作系统 / 分布式推理工作台

**技术栈**：Rust (Core), GPUI (Shell), LanceDB (Vector Store)

---

## 1. 总体架构图 (System Architecture)

Silo 采用 **「双核驱动 + 插件化」** 的架构。前端轻量化，后端重算力与 I/O 控制。

```
┌─────────────┐
│ 用户 / User │
└──────┬──────┘
       │
       ▼
┌─────────────────────────┐
│ GPUI Frontend (Native)  │
└──────────┬──────────────┘
           │ IPC / Direct Call
           ▼
┌─────────────────────────┐
│   Rust Core (The Brain)  │
│  ┌─────────────────────┐ │
│  │ 指令路由/Intent Router│ │
│  │ Swarm Net / libp2p   │ │
│  │ 向量数据库 / LanceDB  │ │
│  │ 安全沙箱 / Wasmtime   │ │
│  └─────────────────────┘ │
└──────────┬──────────────┘
           │
           ▼
┌─────────────────────────┐
│ 推理引擎管理器            │
│  ┌─────────────────────┐ │
│  │ Llama.cpp (Standard) │ │
│  │ MLX Sidecar (Mac)    │ │
│  │ Inferflow (PC/Cluster)│ │
│  └─────────────────────┘ │
└─────────────────────────┘
```

---

## 2. 详细模块设计

### 2.1. 应用外壳与前端 (Frontend & Shell)

- **框架**：**GPUI** (Rust 原生 GPU 加速 UI)
  - *选择理由*：纯 Rust 实现，无 WebView 依赖，内存占用极低，跨平台支持。
- **设计风格**：「Industrial Cyberpunk」(工业赛博风)。
  - 主色调：深炭灰 (#1a1a1a) 配琥珀色 (#ffbf00) 强调。
  - 布局：双栏设计——左侧为 Agent 对话流，右侧为 **Live Artifacts** (实时代码预览/文档渲染)。

### 2.2. 核心后端 (Rust Core)

#### A. 推理后端管理器 (Inference Manager)

**策略 1: Apple Silicon (Mac)**

- **实现**：Rust 通过 `Command` 模块生成轻量级 Python 侧车 (Sidecar) 进程，运行 **MLX** 服务。
- **通信**：使用 gRPC 或 Unix Domain Socket 进行高频 Token 流传输。
- *依据*：只有 MLX 能在 Mac 上达到 ~230 tokens/s 的吞吐量，且提供稳定的延迟。

**策略 2: 通用 PC / Linux**

- **实现**：使用 `llama.cpp` 的 Rust 绑定，或集成 **Inferflow** 的 C++ 库。
- *亮点*：在 PC 端开启 **Inferflow** 的 3.5-bit 量化模式，以平衡显存和精度。

**策略 3: 蜂群模式 (Swarm/Cluster)**

- **实现**：基于 **Inferflow** 的混合分区思想。
- **逻辑**：主节点 (Rust) 将 Prompt 编码后，通过局域网将部分层 (Layers) 的计算任务分发给从节点。

#### B. 数据层：Silo Vault (隐私地窖)

- **向量数据库**：**LanceDB** (Rust Native)。
  - *优势*：嵌入式，无需额外部署 Docker，性能极高，完美适配本地环境。
- **长文本处理**：实现 **Paged KV Cache**。
  - 当 Agent 需要阅读整个代码库时，Rust 后端负责分块 (Chunking) 并存入 LanceDB。推理时，动态加载相关的 KV Cache 页，避免重算。

#### C. Agent 执行沙箱 (The Hand)

- **技术**：**Wasmtime** (WebAssembly Runtime)。
- **安全机制**：
  - 即使用户让 Agent「写一个 Python 脚本处理 Excel」，Silo 实际上是将 Python 编译为 Wasm (或使用 Pyodide) 在沙箱内运行，严格限制文件系统读写权限。
  - 使用能力白名单进行文件访问控制。

#### D. 网络层：Silo Sync (同步)

- **技术**：**libp2p** (Rust crate)。
- **功能**：
  - 实现设备发现 (mDNS)。
  - 建立加密隧道 (Noise Protocol)。
  - 同步 LanceDB 的索引文件和 Markdown 笔记，实现「断网局域网同步」。

---

## 3. Rust Crate 依赖清单 (推荐)

| 模块 | 推荐 Crate | 用途 |
| --- | --- | --- |
| **Async Runtime** | `tokio` | 异步运行时核心 |
| **App Shell** | `gpui` | 原生 GPU 加速 UI |
| **Inference** | `llama-cpp-2` / `candle-core` | 模型加载与推理 |
| **Vector DB** | `lancedb` | 本地向量存储 |
| **Networking** | `libp2p`, `tonic` (gRPC) | P2P 同步与进程间通信 |
| **System Info** | `sysinfo` | 检测硬件以切换后端 |
| **Sandbox** | `wasmtime` | 安全执行 Agent 生成的代码 |

---

## 4. 关键功能实现流程

### 4.1. 启动时的「自适应后端」检测逻辑

```rust
// 伪代码示例
fn detect_and_select_backend() -> BackendType {
    let sys = System::new_all();
    
    if cfg!(target_os = "macos") && has_apple_silicon() {
        return BackendType::MlxSidecar; 
    } else if has_nvidia_gpu() {
        return BackendType::InferflowCpp;
    } else {
        return BackendType::LlamaCppCpu;
    }
}
```

### 4.2. 蜂群模式 (Silo Swarm) 的发现机制

利用 `libp2p` 的 mDNS 发现局域网内的其他 Silo 实例。

1. **Handshake**：交换公钥，建立加密通道。
2. **Capability Check**：询问对方「你有多少空闲显存？」
3. **Partitioning**：如果本地显存不足，将模型的第 20-40 层 (Layers) 标记为 "Remote"，推理时通过网络发送 Tensor 数据。

---

## 5. 开发路线图 (Roadmap)

- **Phase 1: Foundation (MVP)** ✅
  - 搭建 GPUI + Rust 骨架。
  - 实现 `llama.cpp` 的 Rust 绑定，跑通基本的 Chat 功能。
  - 集成 LanceDB 实现简单的文档问答。

- **Phase 2: Optimization (The "Silo" Difference)**
  - **Mac 特化**：实现 Python Sidecar 启动 MLX 后端。
  - **UI 升级**：实现 Artifacts 预览窗口。

- **Phase 3: Connection (Swarm)**
  - 引入 `libp2p` 实现局域网发现。
  - 实现简单的算力卸载 (Offloading)。

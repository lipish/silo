import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import "./App.css";

interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: Date;
}

interface Artifact {
  content: string;
  mime_type: string;
}

function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [artifacts, setArtifacts] = useState<Artifact[]>([]);
  const [backendType, setBackendType] = useState<string>("");
  const [isLoading, setIsLoading] = useState(false);
  const [vaultStats, setVaultStats] = useState<{ document_count: number }>({ document_count: 0 });
  const [isUploading, setIsUploading] = useState(false);

  useEffect(() => {
    // 获取后端类型
    invoke<string>("get_backend_type")
      .then((type) => setBackendType(type))
      .catch(console.error);
    
    // 获取 Vault 统计信息
    invoke<{ document_count: number }>("get_vault_stats")
      .then((stats) => setVaultStats(stats))
      .catch(console.error);
    
    // 定期更新统计信息
    const interval = setInterval(() => {
      invoke<{ document_count: number }>("get_vault_stats")
        .then((stats) => setVaultStats(stats))
        .catch(console.error);
    }, 5000);
    
    return () => clearInterval(interval);
  }, []);

  const handleFileUpload = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Text Files',
          extensions: ['txt', 'md', 'markdown', 'json', 'yaml', 'yml', 'py', 'rs', 'js', 'ts', 'jsx', 'tsx', 'html', 'css']
        }]
      });

      if (!selected) {
        return; // 用户取消
      }

      setIsUploading(true);
      const filePath = typeof selected === 'string' ? selected : selected.path || selected;
      
      await invoke<string>("add_file_to_vault", { filePath });
      
      // 更新统计信息
      const stats = await invoke<{ document_count: number }>("get_vault_stats");
      setVaultStats(stats);
      
      // 显示成功消息
      const successMessage: Message = {
        id: Date.now().toString(),
        role: "assistant",
        content: `✅ 文件已添加到 Vault: ${filePath.split('/').pop() || filePath}`,
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, successMessage]);
    } catch (error) {
      console.error("Error uploading file:", error);
      const errorMessage: Message = {
        id: Date.now().toString(),
        role: "assistant",
        content: `❌ 上传文件失败: ${error instanceof Error ? error.message : String(error)}`,
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsUploading(false);
    }
  };

  const handleSend = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    const userMessage: Message = {
      id: Date.now().toString(),
      role: "user",
      content: input,
      timestamp: new Date(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInput("");
    setIsLoading(true);

    try {
      const response = await invoke<any>("execute_agent_task", {
        instruction: input,
        context: null,
      });

      const assistantMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: "assistant",
        content: response.reasoning || "任务执行完成",
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, assistantMessage]);
      setArtifacts(response.artifacts || []);
    } catch (error) {
      console.error("Error executing task:", error);
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: "assistant",
        content: `❌ 错误: ${error instanceof Error ? error.message : String(error)}\n\n提示：当前运行在模拟模式下。要使用真实 AI 模型，请配置模型文件。`,
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="h-screen flex flex-col bg-charcoal text-gray-200">
      {/* 顶部状态栏 */}
      <header className="border-b border-amber/20 px-4 py-2 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-bold text-amber font-mono">SILO</h1>
          <div className="flex items-center gap-4">
            <span className="text-xs text-gray-400 font-mono">
              Backend: {backendType || "检测中..."}
            </span>
            <span className="text-xs text-gray-500 font-mono">
              Vault: {vaultStats.document_count} docs
            </span>
          </div>
        </div>
        <div className="text-xs text-gray-500 font-mono">
          Your Data's Fortress
        </div>
      </header>

      {/* 主工作区 - 双栏布局 */}
      <div className="flex-1 flex overflow-hidden">
        {/* 左侧：对话区 */}
        <div className="flex-1 flex flex-col border-r border-amber/10">
          <div className="flex-1 overflow-y-auto p-4 space-y-4">
            {messages.length === 0 && (
              <div className="text-center text-gray-500 mt-20">
                <p className="text-lg mb-2">欢迎使用 Silo AI</p>
                <p className="text-sm">隐私优先的本地 Agent 操作系统</p>
                <p className="text-xs mt-4 text-gray-600">
                  输入指令，让 Silo 帮你完成任务
                </p>
              </div>
            )}
            {messages.map((msg) => (
              <div
                key={msg.id}
                className={`flex ${
                  msg.role === "user" ? "justify-end" : "justify-start"
                }`}
              >
                <div
                  className={`max-w-[80%] rounded-lg p-3 font-mono text-sm ${
                    msg.role === "user"
                      ? "bg-amber/20 text-amber border border-amber/30"
                      : "bg-gray-800 text-gray-200 border border-gray-700"
                  }`}
                >
                  <div className="whitespace-pre-wrap">{msg.content}</div>
                  <div className="text-xs text-gray-500 mt-1">
                    {msg.timestamp.toLocaleTimeString()}
                  </div>
                </div>
              </div>
            ))}
            {isLoading && (
              <div className="flex justify-start">
                <div className="bg-gray-800 rounded-lg p-3 border border-gray-700">
                  <div className="flex gap-1">
                    <span className="w-2 h-2 bg-amber rounded-full animate-pulse"></span>
                    <span className="w-2 h-2 bg-amber rounded-full animate-pulse delay-75"></span>
                    <span className="w-2 h-2 bg-amber rounded-full animate-pulse delay-150"></span>
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* 输入框 */}
          <form onSubmit={handleSend} className="border-t border-amber/10 p-4">
            <div className="flex gap-2">
              <input
                type="text"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                placeholder="输入指令..."
                className="flex-1 bg-gray-900 border border-gray-700 rounded px-4 py-2 text-sm font-mono focus:outline-none focus:border-amber/50 text-gray-200"
                disabled={isLoading}
              />
              <button
                type="button"
                onClick={handleFileUpload}
                disabled={isLoading || isUploading}
                className="px-4 py-2 bg-gray-700 text-gray-200 rounded hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed font-mono text-sm transition-colors"
                title="添加文件到 Vault"
              >
                {isUploading ? "..." : "📁"}
              </button>
              <button
                type="submit"
                disabled={isLoading || !input.trim()}
                className="px-6 py-2 bg-amber text-charcoal font-bold rounded hover:bg-amber/80 disabled:opacity-50 disabled:cursor-not-allowed font-mono text-sm transition-colors"
              >
                执行
              </button>
            </div>
          </form>
        </div>

        {/* 右侧：Artifacts 预览区 */}
        <div className="w-96 flex flex-col border-l border-amber/10 bg-gray-900/50">
          <div className="border-b border-amber/10 px-4 py-2">
            <h2 className="text-sm font-bold text-amber font-mono">
              LIVE ARTIFACTS
            </h2>
          </div>
          <div className="flex-1 overflow-y-auto p-4 space-y-4">
            {artifacts.length === 0 ? (
              <div className="text-center text-gray-500 mt-20">
                <p className="text-xs">实时工件将显示在这里</p>
              </div>
            ) : (
              artifacts.map((artifact, idx) => (
                <div
                  key={idx}
                  className="bg-gray-800 rounded-lg p-3 border border-gray-700"
                >
                  <div className="text-xs text-gray-400 mb-2 font-mono">
                    {artifact.mime_type}
                  </div>
                  <pre className="text-xs text-gray-200 font-mono whitespace-pre-wrap overflow-x-auto">
                    {artifact.content}
                  </pre>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;

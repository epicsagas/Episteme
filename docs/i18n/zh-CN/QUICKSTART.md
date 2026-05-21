# Episteme — 快速入门指南

在2分钟内开始使用Episteme。

---

## 前提条件

- **Rust 1.95及以上版本**（需要2024版）— [通过rustup安装](https://rustup.rs)
- 网络连接（用于初始数据下载）

---

## 方式一：AI工具集成（推荐）

**适用于：** Claude Code、Cursor、Codex、Gemini用户

```bash
# 1. 安装Episteme
cargo install --git https://github.com/epicsagas/Episteme

# 2. 安装到你的AI工具（下载数据、配置MCP、复制代理）
epis install claude      # Claude Code
epis install cursor      # Cursor
epis install codex       # OpenAI Codex
epis install gemini      # Antigravity
epis install all         # 一次性安装所有工具
```

> 如果`epis install claude`下载数据失败，请使用下面的源码安装方式。

**完成。** 重启你的AI工具，Episteme即可生效。

---

## 方式二：Docker（无需Rust）

```bash
docker-compose up -d

# 访问地址
# API:       http://localhost:8000
# 健康检查:    http://localhost:8000/health
```

通过Docker进行MCP集成，请将以下内容添加到MCP配置中：
```json
{
  "mcpServers": {
    "episteme": {
      "command": "docker",
      "args": ["exec", "-i", "episteme-api", "episteme", "mcp"]
    }
  }
}
```

---

## 方式三：从源码构建

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme

# 构建
cargo build --release

# 填充数据并构建向量数据库（构建会自动运行）
./target/release/epis install --local
```

---

## 图谱可视化

Episteme包含一个交互式D3力导向图查看器：

```bash
episteme web               # 默认: http://localhost:8080
episteme web --port 9001   # 自定义端口
episteme web --host 0.0.0.0 --port 8080  # 暴露到网络
```

---

## 常用命令

```bash
# 分析代码异味
epis analyze my_code.py --language python
epis analyze my_code.py --json

# 获取重构建议
episteme infer my_code.py --top-k 5

# 探索知识图谱
epis explore "strategy pattern"
epis graph path DP-005 RF-001

# 启动服务器
epis api              # REST API :8000
episteme mcp --http       # MCP服务器 :43175
episteme web --port 8080  # Web界面

# 后台MCP守护进程（HTTP代理）
epis service start
epis service status
epis service stop

# 创建发布归档
episteme dist --out-dir release
```

---

## 故障排除

### "Database not found"
```bash
epis install claude   # 重新下载数据归档
# 或
epis install --local
```

### "Port already in use"
```bash
episteme web --port 9001
epis api --port 9000
```

---

## 下一步

- **[README](../../README.md)** — 完整功能概述与架构
- **[MCP集成指南](./mcp-integration-guide.md)** — 工具参考与代理示例
- **[API参考](./api.md)** — REST端点
- **[贡献指南](../../CONTRIBUTING.md)** — 开发工作流

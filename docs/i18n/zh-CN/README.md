<p align="center">
<img src="../assets/icon.png" alt="Episteme" width="60%" />
</p>

<p align="center"><sub>Episteme (συνταγμα) —— 希腊语中意为"有组织的系统"或"辨识力"</sub></p>

<p align="center">一个离线优先、单二进制文件的知识图谱，通过语义关系将设计模式、重构技术和软件法则连接在一起。<br><b>优先为 AI 智能体而生</b> —— 将软件工程专业知识直接集成到 Claude Code、Cursor 及其他兼容 MCP 的工具中。</p>

<p align="center">使用 Rust 编写 · 单一二进制文件 · 完全离线</p>

<p align="center">
    <a href="https://github.com/epicsagas/Episteme/actions"><img src="https://img.shields.io/github/actions/workflow/status/epicsagas/Episteme/ci.yml?branch=main&label=CI" alt="CI" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.95+-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/episteme"><img src="https://img.shields.io/badge/crates.io-v0.1.0-orange.svg" alt="crates.io" /></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

<p align="center">
  <a href="../../README.md">English</a> |
  <a href="../ja/">日本語</a> |
  <a href="../ko/">한국어</a> |
  <a href="../de/">Deutsch</a> |
  <a href="../fr/">Français</a> |
  简体中文 |
  <a href="../zh-TW/">繁體中文</a> |
  <a href="../pt/">Português</a> |
  <a href="../es/">Español</a> |
  <a href="../hi/">हिन्दी</a>
</p>

---

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/features.png">
  <img src="../assets/features.png" align="center" width="100%" alt="Episteme 功能概览" />
</picture>

---

## 快速开始

### Claude Code

```
/plugin marketplace add epicsagas/plugins
/plugin install episteme@epicsagas
```

安装后，运行一次以下命令下载知识图谱数据（MCP 正常工作所必需）：

```bash
epis install   # 从 GitHub Releases 下载知识图谱数据
```

MCP 工具和 4 个专业代理将自动注册。重启 Claude Code 即可使用。

更新：`/plugin update episteme@epicsagas`

### Codex CLI

```bash
codex plugin marketplace add epicsagas/plugins
```

安装后，运行一次以下命令下载知识图谱数据（MCP 正常工作所必需）：

```bash
epis install   # 从 GitHub Releases 下载知识图谱数据
```

重启后立即可用。

更新：`codex plugin update episteme@epicsagas`

### 其他工具

```bash
epis install cursor       # Cursor IDE
epis install opencode     # OpenCode
epis install cline        # Cline
epis install --all        # 所有支持的工具
```

### 手动安装

| 方法 | 命令 |
|------|------|
| **Homebrew** | `brew install epicsagas/tap/episteme` |
| **Shell 脚本** | `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/epicsagas/Episteme/releases/latest/download/episteme-installer.sh \| sh` |
| **cargo** | `cargo binstall episteme` ⚡ 或 `cargo install episteme` |
| **Docker** | 见 [选项 3](#选项-3docker无需-rust) |

### 验证

```bash
epis --version
epis stats
```

也可以在 Claude Code / Codex CLI 中直接运行:

```
/episteme verify
```

### 30 秒上手体验

**方式 A —— CLI：** 指向项目中的任意文件。

```bash
epis analyze src/domain/engine.rs
```

```
✓ 2 smells detected in src/domain/engine.rs

  SMELL-07 (Large Class) — RefactoringRanker, 743 lines
  → RF-018 Extract Class          priority 0.89  effort: medium
  → RF-001 Extract Method         priority 0.76  effort: small
  → Violates: LAW-001 Single Responsibility Principle

  SMELL-01 (Long Method) — rank_refactorings(), 58 lines
  → RF-001 Extract Method         priority 0.92  effort: small
  → Violates: LAW-001 SRP, LAW-004 DRY
```

**方式 B —— Claude Code：** 打开项目中的任意文件，自然地提问即可。

```
Find code smells in this project and suggest refactorings.
```

Episteme 自动触发 —— 无需特殊语法。它会将你的描述映射到知识图谱并返回带有引用的排序结果。

---

## 为什么选择 Episteme？

LLM 已经知道策略模式是什么。它们能背诵 SOLID 原则、列举 GoF 模式、解释代码坏味道。那么这个项目为什么存在？

**差距不在于知识 —— 而在于结构化的、关联性的推理。**

当你问 LLM "如何修复上帝对象？"时，它会给出一个合理的答案。但答案在不同的对话中会变化，缺乏可追溯性，并且没有将问题与其根本原因或下游后果联系起来。Episteme 将孤立的事实转化为可遍历的图谱，其中每条建议都有据可依、可引用，并与更广泛的设计领域相互关联。

### 这与精心设计的 LLM 提示词有什么不同？

|  | 精心设计的 LLM 提示词 | Episteme + LLM |
|---|---|---|
| 主动检测 | 仅当用户提出正确的问题时才会触发 | 在问题描述上自动触发 |
| Token 效率 | 冗长的解释 + 多轮追问 | 一次工具调用返回结构化结果 |
| 关系遍历 | 最多单跳，常常是幻觉产生的 | 多跳图谱遍历，经过验证 |
| 交叉引用 | 手动的，容易出错 | 通过 201 条语义关系自动完成 |
| 一致性 | 不同对话之间会有变化 | 每次返回相同的结构化答案 |
| 可引用性 | "我认为你应该使用 Extract Class" | "Extract Class (RF-018)，优先级 0.89" |
| 离线 / 物理隔离 | 需要互联网才能获得最佳结果 | 完全本地运行，单一二进制文件 |

### 什么时候有用？

<details>
<summary><b>1. 当你希望 AI 智能体主动检测问题，而不是等待被询问时</b></summary>

MCP 集成会在问题描述上自动触发。当用户说"这个类做的事情太多了"时，智能体不需要知道要询问上帝对象 —— Episteme 将这个描述映射到 `SMELL-03`，展示排序的重构方案，并将违规追溯到第一性原则。这将模糊的抱怨变成了结构化的修复方案。
</details>

<details>
<summary><b>2. 当你想减少 token 消耗 —— 而不是把它浪费在解释上时</b></summary>

没有 Episteme，LLM 回答"如何修复上帝对象？"时会解释这个坏味道、列出重构方案、描述 SOLID 原则，并逐一讲解每个选项 —— 每次响应消耗数百个 token。有了 Episteme，一次 MCP 工具调用即可返回 `SMELL-03 → RF-018 (0.89) → LAW-001`。同样的专业知识，仅需极少部分的 token 预算。
</details>

<details>
<summary><b>3. 当你需要将代码分析与修复方案连接起来 —— 而不仅仅是检测时</b></summary>

像 SonarQube 这样的工具可以检测坏味道。LLM 可以建议模式。Episteme 两者兼做并将它们连接起来：检测 Long Method → 追踪到它违反的法则 → 对解决它的重构方案进行排序 → 展示哪些模式强化了这些重构。
</details>

<details>
<summary><b>4. 当孤立的模式知识不够时 —— 你需要的是关系</b></summary>

知道 Extract Method 做什么是基本要求。知道它*解决* Long Method (SMELL-01)，而 Long Method *违反*单一职责原则 (LAW-001)，单一职责原则又被 Facade 模式 (DP-012) *强化* —— 这是一条 LLM 无法可靠自行构建的推理链。Episteme 的 201 条语义关系让 AI 智能体能够确定性地遍历这些路径。
</details>

<details>
<summary><b>5. 当你在做架构决策时需要的是证据，而非观点时</b></summary>

"我应该使用微服务吗？" —— Episteme 将问题连接到康威定律 (LAW-017)、SRP (LAW-001) 和绞杀者无花果模式 (DP-026)，然后展示它们之间的关系。决策可追溯到工程法则，而非博客文章。
</details>

<details>
<summary><b>6. 当你需要一致、可引用的工程建议 —— 而不是幻觉产生的推荐时</b></summary>

每个发现都引用明确的实体 ID（`DP-005`、`RF-001`、`LAW-021`）。推荐附带优先级分数和工作量估算。相同的查询始终返回相同的结构化答案。
</details>

<details>
<summary><b>7. 当你在物理隔离或受限网络环境中工作时</b></summary>

Episteme 完全离线运行：单一二进制文件、本地 SQLite 数据库、通过 fastembed (ONNX Runtime) 实现的本地嵌入。无遥测、无回传、无外部 API 调用。你的代码和分析结果永远不会离开你的机器。
</details>

---

## 功能

| | 功能 | 为什么重要 |
|--|------|-----------|
| 🧠 | **22个GoF设计模式** | 包含实际示例的完整目录 |
| 🔧 | **66个重构技术** | Fowler目录，附带代码示例 |
| ⚖️ | **56条软件定律与原则** | SOLID、康威定律、CAP定理等 |
| 👃 | **17种代码异味类型** | Long Method、God Object、Feature Envy等 ¹ |
| 🔗 | **201条语义关系** | "解决"、"强制"、"违反"、"关联" |
| 🤖 | **9个MCP工具 + 4个代理** | 高保真AI代理交互，支持代理间交接 |
| 🌍 | **10种语言支持** | Python（AST）、Java、TypeScript、Go、Rust、C++、C#、PHP、Ruby、Kotlin |
| 📊 | **确定性分析** | 基于AST的Python + 正则多语言，每次结果一致 |
| 🏷️ | **可引用的知识** | 每个发现都链接到明确的实体ID（`RF-001`、`LAW-021`） |
| 🌐 | **REST API（17个端点）** | 认证、速率限制、健康探针、Prometheus指标 |
| 📦 | **单一二进制文件** | 无运行时依赖，跨平台（macOS、Linux、Windows） |
| 🔌 | **本地嵌入** | fastembed（ONNX Runtime），零配置语义搜索 |
| 🐳 | **Docker支持** | 带健康检查的多阶段构建 |

> ¹ Duplicate Code（SMELL-13）和Shotgun Surgery（SMELL-09）需要多文件上下文，在单文件模式下会跳过。

---

## 安装

### 选项 1：cargo-binstall（推荐）

```bash
cargo binstall episteme    # 下载预编译二进制文件 — 无需编译
epis install cursor        # 种子数据 + 配置 MCP + 安装智能体
```

如果没有 cargo-binstall：`cargo install cargo-binstall`

> 运行 `epis install cursor` 后，**重启 Claude Code** 以使 MCP 工具和智能体生效。

### 选项 2：从源码构建

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme && cargo build --release
```

然后运行适用于你平台的二进制文件：

| 平台 | 命令 |
|----------|---------|
| **macOS / Linux** | `./target/release/epis install --local cursor` |
| **Windows** | `.\target\release\episteme.exe install --local cursor` |

### 选项 3：Docker（无需 Rust）

```bash
docker-compose up -d
```

添加到你的 MCP 配置文件：

| 工具 | 配置文件路径 |
|------|-----------------|
| Claude Code | `~/.claude.json` |
| Cursor | `.cursor/mcp.json` |
| VS Code (Copilot) | `.vscode/mcp.json` |

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

### 选项 4：预编译二进制文件（无需 Rust）

从 [GitHub Releases](https://github.com/epicsagas/Episteme/releases) 下载适用于你平台的最新二进制文件：

| 平台 | 文件 |
|----------|------|
| **macOS**（Apple Silicon） | `episteme-aarch64-apple-darwin.tar.xz` |
| **Linux**（x86_64） | `episteme-x86_64-unknown-linux-gnu.tar.xz` |

```bash
# macOS / Linux
tar xzf episteme-*.tar.gz
sudo mv episteme /usr/local/bin/

# Windows — 解压 zip 文件并将 episteme.exe 添加到你的 PATH
```

然后安装：
```bash
epis install cursor
```

### 验证

```bash
epis --version
epis stats
epis explore "strategy pattern"    # 探索知识图谱
```

---

## MCP 工具与智能体

> **什么是 MCP？** [模型上下文协议](https://modelcontextprotocol.io) 是一个开放标准，允许 AI 工具调用外部服务。Episteme 将其知识图谱作为 MCP 工具暴露出来，Claude Code、Cursor 及其他兼容的编辑器可以自动调用。

### 9 个 MCP 工具

#### 规范知识（6 个工具）

| 工具 | 用途 | 使用示例 |
|------|---------|-------------|
| **`search_knowledge`** | 跨所有实体的语义搜索 | "查找重试逻辑的模式" |
| **`get_entity`** | 按ID获取特定实体详情 | "解释策略模式 (DP-023)" |
| **`get_neighbors`** | 探索相关实体 | "哪些重构能解决长方法？" |
| **`find_path`** | 查找两个实体之间的连接 | "SRP与提取类有何关系？" |
| **`analyze_code`** | 通过正则/AST分析检测代码异味 | "审查此支付验证代码" |
| **`suggest_refactorings`** | 排序的重构建议 | "这个类应该重构什么？" |

#### 隐性知识（3 个工具）

| 工具 | 用途 | 使用示例 |
|------|---------|-------------|
| **`add_insight`** | 记录团队决策、经验教训 | "选择事件驱动而非轮询的原因" |
| **`search_insights`** | 搜索过去的团队知识 | "我们对认证中间件做了什么决定？" |
| **`confirm_links`** | 验证自动检测到的规范实体链接 | 确认 TK-001 与 SMELL-03 相关联 |

Episteme 将隐性知识存储在独立的数据库（`~/.episteme/user_knowledge.db`）中，并在运行时通过组合层与规范图合并。团队洞察会自动链接到模式、法则和异味，将经验转化为可导航的知识。

完整设计请参阅[隐性知识架构](./tacit-knowledge.md)。

### 4 个专用智能体（互联网络）

智能体协同工作 —— 每次分析都以**后续步骤**选项结束，可移交给其他智能体。

| 智能体 | 使用时机 | 核心能力 | 移交给 |
|-------|-------------|----------------|--------------|
| **`code-reviewer`** | 代码坏味道、SOLID 违规 | 因果分析（根本原因 → 下游症状） | advisor, architecture-analyst, refactoring-expert |
| **`episteme-advisor`** | 工程决策、权衡取舍 | 多实体权衡链及行动计划 | code-reviewer, architecture-analyst, researcher |
| **`episteme-researcher`** | 知识图谱探索 | 模式、法则、坏味道之间的连接图 | advisor, code-reviewer |
| **`architecture-analyst`** | 对照法则评估架构 | 带风险加权评估的合规评分 | advisor, code-reviewer, researcher |

**工作流示例**：`code-reviewer` 检测到 God Object → 追踪因果关系到 3 个下游坏味道 → 提供"应用 RF-018"（→ refactoring-expert）或"深入分析根本原因"（→ episteme-advisor）或"架构检查"（→ architecture-analyst）。

[完整 MCP 集成指南](./mcp-integration-guide.md)

---

## CLI 使用

```bash
# 分析代码中的坏味道
epis analyze my_code.py --language python --json
episteme infer my_code.py

# 探索知识图谱
epis explore "strategy pattern"
epis graph path DP-005 RF-001   # 例如：Factory Method → Extract Method

# 构建 RAG 索引
epis build

# 启动服务器
epis api              # REST API，端口 :8000
episteme mcp --http       # MCP 服务器，端口 :43175
episteme web --port 8080  # Web UI（交互式图谱浏览器）

# 分发打包
episteme dist --out-dir release/
```

---

## 文档

| 文档 | 说明 |
|----------|-------------|
| [快速开始](./QUICKSTART.md) | 逐步设置、首次运行、故障排除 |
| [MCP 集成指南](./mcp-integration-guide.md) | 工具参考、智能体示例、对话流程 |
| [隐性知识架构](./tacit-knowledge.md) | 双数据库设计、洞察生命周期、模式 |
| [Alcove 生态系统对比](./alcove-ecosystem.md) | 存储模型、搜索能力、用例矩阵 |
| [Alcove 集成指南](./alcove-integration.md) | 双上下文工作流、设置、最佳实践 |
| [API 参考](./api.md) | REST 端点、身份验证、示例 |
| [分发](./distribution.md) | 发布打包和部署 |
| [开发与贡献](./DEVELOPMENT.md) | 架构说明、如何贡献 |
| [更新日志](./CHANGELOG.md) | 发布历史和版本说明 |

---

## 配置

### 环境变量

```bash
# 数据位置
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# API 服务器
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEY=your-secret-key

# MCP 服务器
EPISTEME_MCP_HOST=127.0.0.1
EPISTEME_MCP_PORT=43175
```

---

## 故障排除

**安装后找不到 `episteme` 命令**

| 平台 | 解决方法 |
|----------|-----|
| **macOS / Linux** | `export PATH="$HOME/.cargo/bin:$PATH"` —— 添加到 `~/.bashrc` 或 `~/.zshrc` 以持久化 |
| **Windows** | 将 `%USERPROFILE%\.cargo\bin` 添加到系统 PATH，或打开一个新终端 |

**MCP 工具未在 Claude Code / Cursor 中出现**

运行 `epis install` 后重启编辑器。如果仍然缺失，检查配置是否已写入：
```bash
cat ~/.claude.json   # Claude Code
```

**端口已被占用**
```bash
episteme mcp --http --port 43176   # 使用不同的端口
```

**首次启动缓慢**

Episteme 在首次运行时会构建本地嵌入索引。这需要 30–60 秒，是一次性成本。后续启动是即时的。

**`cargo install` 时出现编译错误**

确保已安装 Rust 1.95+：
```bash
rustup update stable
rustup show   # 确认当前工具链
```

> 更多帮助：[QUICKSTART.md 故障排除部分](../../QUICKSTART.md#troubleshooting) · [提交 Issue](https://github.com/epicsagas/Episteme/issues)

---

## 路线图

- [ ] **自定义实体** —— 添加团队特定的模式/坏味道
- [ ] **交互式教程** —— 应用内 MCP 工具引导教程
- [ ] **多语言元数据** —— 实体标题和摘要的韩语、日语、中文支持（README 翻译已完成）
- [ ] **MCP 工具描述** —— 替代 IDE 专用插件的增强工具描述
- [ ] **团队指标** —— 跨组织的模式使用聚合

---

## 贡献

欢迎贡献！请参阅 [DEVELOPMENT.md](./DEVELOPMENT.md) 了解架构概览和贡献指南。

```bash
# 运行测试
cargo test

# 代码检查
cargo clippy -- -D warnings

# 格式化
cargo fmt
```

有问题？[开启讨论](https://github.com/epicsagas/Episteme/discussions) 或 [提交 Issue](https://github.com/epicsagas/Episteme/issues)。

---

## 许可证

Apache 2.0 —— 详情请参阅 [LICENSE](../../LICENSE)。

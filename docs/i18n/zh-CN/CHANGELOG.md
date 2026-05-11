# 更新日志

本文件记录了Episteme的所有重要变更。

格式基于[Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，
本项目遵循[语义化版本](https://semver.org/spec/v2.0.0.html)。

## [Unreleased]

### Changed

- CLI: `explore`重命名为`search`（旧名称作为已弃用别名仍然可用）
- CLI: `mcp`和`api`现在管理完整的服务生命周期（`start`、`stop`、`restart`、`status`、`enable [--now]`、`disable [--now]`）
- CLI: `service`顶层命令已弃用 — 请使用`mcp start/stop/restart/status/enable/disable`
- CLI: `mcp --http`已弃用 — 请使用`mcp start`进行HTTP守护进程模式
- CLI: `launchd-install/uninstall/status`已弃用 — 请使用`mcp enable/disable/status`
- `enable/disable`现已跨平台支持: macOS（launchd）和Linux（systemd用户单元）

### Added

- `api start/stop/restart/status/enable/disable` — REST API守护进程生命周期管理
- 为`mcp enable`生成Linux systemd用户单元

- **面向Claude Code的MCP HTTP传输** — 传输选择器TUI、HTTP作为默认值、launchd自动启用
- **代理提示词自动安装** — `epis install`将Episteme代理提示词复制到`~/.claude/agents/`
- **实体描述** — 从Markdown源文件自动提取描述字段，在Web查看器详情面板中显示
- **基准测试可视化SPA** — 趋势分析、查询分解仪表板
- **Web查看器重新设计** — Sankey图布局、侧边栏树形结构、详情面板、子图可读性改进
- **MCP配置更新插入** — 再次运行`epis install`时，配置不同则更新传输方式（stdio ↔ HTTP）
- **MCP YAML配置** — `config.yaml`中的`mcp.host` / `mcp.port`（yaml → 环境变量回退）
- **监控** — 通过环境变量支持原生和远程Prometheus抓取目标
- **CI加固** — cargo audit、gitleaks、SBOM生成、固定操作SHA
- **发布流水线** — Windows目标、crates.io发布、Homebrew tap
- `examples/`中的**God模块架构诊断示例**

### Changed

- **安装向导** — 所有步骤（传输、Redis、遥测）迁移到全屏TUI
- **安装流程** — 填充数据后自动构建RAG索引，数据库已存在时跳过
- **知识图谱** — 丰富了跨实体语义关系
- **许可证** — MIT → Apache-2.0

### Fixed

- 遥测同步`main()`中的Tokio运行时panic
- 搜索质量 — 解决NDCG测量错误，hit@1准确率提升至100%
- 搜索召回 — 跨类型提升、稀疏实体处理、意图同义词
- fastembed模型缓存固定到`~/.episteme/models`
- launchd引导UID替换和端口占用处理
- CORS来源现可通过`EPISTEME_CORS_ORIGINS`配置

## [0.1.0] - 2026-05-03

### Added

- **完全Rust重写** — 用惯用Rust完全替换Python代码库
- **六边形架构** — `ports/`（特征）、`domain/`（业务逻辑）、`adapters/`（基础设施）、`server/`（HTTP）
- **GenericParser框架** — 8个基于大括号的解析器合并为带有`ParserConfig`的`GenericParser`；正则表达式模式通过`OnceLock`和`Box::leak`缓存
- **Python AST解析** — 使用`rustpython-parser`进行准确的Python异味检测（Long Method、Large Class、God Object）
- **TieredAccum + build_detection()** — 消除`detectors.rs`中14个相同的异味检测构建的重复（1,253 → 591行）
- **MCP模块拆分** — 将`EpistemeMCP`（675行）拆分为`mcp_search`、`mcp_graph`、`mcp_analysis`服务
- **CLI命令拆分** — 将`main.rs`（1,741行）拆分为带有`cli.rs`定义clap的`commands/`模块
- **API处理器去重** — 将重复的`search`/`search_post`合并为共享的`do_search()`
- **16个异味检测函数** — 从14个增加，覆盖所有GoF异味类别
- **17个REST API端点** — 健康探测、Prometheus指标、CORS、速率限制
- **速率限制器TTL驱逐** — MAX_BUCKETS=10,000，1小时TTL，防止无限内存增长
- **ReDoS缓解** — 将三元运算符正则从`[^:]+`限制为`[^:\n]{1,50}`
- **本地嵌入** — 使用fastembed（ONNX Runtime）实现零配置语义搜索
- **交互式安装向导** — 带有crossterm、Vim键绑定、备用屏幕的TUI
- **分发打包** — `episteme dist`命令，用于创建带有自动数据库引导的发布归档
- **跨平台CI** — 面向linux/macOS（x86_64 + aarch64）的GitHub Actions发布工作流
- **多阶段Dockerfile** — Rust构建器 + 精简Debian运行时

### Changed

- **语言**: Python 3.11+ → Rust（2024版）
- **Web框架**: FastAPI → axum
- **数据库**: Python sqlite3 → rusqlite（捆绑）
- **嵌入**: sentence-transformers/PyTorch → fastembed/ONNX Runtime
- **CLI**: argparse → clap（derive）
- **所有正则表达式模式已缓存** — 通过全局`REGEX_CACHE`在热路径上零重编译

### Removed

- Python运行时依赖
- ChromaDB依赖
- tree-sitter依赖
- PyPI发布工作流
- `episteme-hook`独立二进制文件（原为Python专用PyPI入口点） — 请使用`episteme hooks ground|sniff|audit`

## [0.0.5] - 2026-04-30

### Added

- 使用D3-force的图谱可视化Web界面（`episteme web`）
- 发布归档中包含预构建的向量数据库
- 用于开发工作流的`epis install --local`标志
- 涵盖所有161个实体的650多个语义关系
- 发布时CI自动生成向量数据库

## [0.0.4] - 2026-04-29

### Added

- 包含6个工具的MCP服务器
- 4个专用代理
- `epis install`命令
- `epis service`守护进程管理
- 混合搜索（FTS5 + 向量）
- Redis缓存、GPU加速
- 10种语言代码异味检测
- Prometheus + Grafana监控

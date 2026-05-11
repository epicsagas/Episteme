# Episteme 开发指南

**项目：** Episteme v0.1.0
**语言：** Rust（2024版）
**最后更新：** 2026-05-03

---

## 当前状态

| 组件 | 状态 | 详情 |
|-----------|--------|---------|
| **知识库** | 已完成 | 22个模式、66个重构、56条法则、23个异味、201条关系 |
| **代码异味检测** | 生产就绪 | 16个检测函数、10种语言 |
| **REST API** | 生产就绪 | 17个端点（axum）、速率限制、认证 |
| **MCP服务器** | 生产就绪 | 6个工具、stdio + HTTP传输 |
| **RAG管道** | 生产就绪 | SQLite + FTS5 + fastembed（ONNX） |
| **图谱可视化** | 生产就绪 | 带有D3力导向的交互式Web界面 |

---

## 架构

六边形（端口与适配器）架构：

```
src/
├── commands/          # CLI子命令处理器（clap）
│   ├── analysis.rs    # analyze, infer
│   ├── build.rs       # build（RAG管道）
│   ├── explore.rs     # explore（搜索/REPL）
│   ├── graph.rs       # graph查询
│   ├── install.rs     # 安装向导（TUI）
│   ├── service.rs     # MCP HTTP守护进程管理
│   └── other.rs       # api, mcp, web, telemetry, hooks
├── adapters/          # 基础设施层
│   ├── regex_parsers.rs   # GenericParser（10种语言，OnceLock正则缓存）
│   ├── python_ast_parser.rs  # Python AST（rustpython-parser）
│   ├── search_engines.rs  # FTS5关键词 + 余弦相似度
│   ├── service.rs         # MCP HTTP守护进程
│   ├── sqlite_db.rs       # SQLite连接池
│   ├── cache.rs           # Redis缓存（可选）
│   └── ...
├── domain/            # 业务逻辑（无外部依赖）
│   ├── graph.rs       # KnowledgeGraph（BFS、子图、矛盾、Jaccard）
│   ├── detectors.rs   # 带有TieredAccum的16个异味检测器
│   ├── engine.rs      # RefactoringInferenceEngine + RefactoringRanker
│   ├── summarizer.rs  # 详细级别响应优化
│   └── types.rs       # EntityType、RelationType、核心类型
├── server/            # HTTP层（axum）
│   ├── api_routes.rs  # 17个REST端点
│   ├── mcp_handler.rs # MCP薄外观层
│   ├── mcp_search.rs  # 搜索服务
│   ├── mcp_graph.rs   # 图谱服务
│   └── mcp_analysis.rs # 代码分析服务
└── ports/             # 特征（六边形边界）
    ├── parser.rs      # CodeParser特征
    ├── search.rs      # SearchEngine特征
    ├── graph.rs       # GraphStore特征
    └── embeddings.rs  # EmbeddingProvider特征
```

---

## 技术栈

| 组件 | 技术 | 用途 |
|-----------|-----------|---------|
| **语言** | Rust（2024版） | 安全性、性能、单一二进制 |
| **Web框架** | axum | REST API + MCP HTTP传输 |
| **数据库** | rusqlite（捆绑SQLite） | 知识图谱 + 向量存储 |
| **搜索** | FTS5 + 余弦相似度 | 关键词 + 语义混合搜索 |
| **嵌入** | fastembed（ONNX Runtime） | 本地、零配置的嵌入生成 |
| **CLI** | clap（derive） | 15个子命令 |
| **Python AST** | rustpython-parser | 基于AST的Python异味检测 |
| **其他语言** | regex（OnceLock缓存） | GenericParser框架 |

---

## 代码异味检测器（16）

| ID | 异味 | 检测方式 |
|----|-------|-----------|
| SMELL-01 | Long Method | LOC阈值 |
| SMELL-02 | Long Parameter List | 参数数量 |
| SMELL-03 | Primitive Obsession | 基本类型参数比率 |
| SMELL-04 | Large Class | 方法 + 字段数量 |
| SMELL-05 | Data Clumps | 重复参数组（存根） |
| SMELL-06 | Switch Statements | switch/match数量 |
| SMELL-07 | Data Class | 方法与字段比率 |
| SMELL-08 | Temporary Field | 条件字段使用（存根） |
| SMELL-09 | Shotgun Surgery | 变更耦合（存根） |
| SMELL-10 | Divergent Change | 方法内聚性度量 |
| SMELL-11 | Lazy Class | 低LOC + 方法数 |
| SMELL-12 | Speculative Generality | 有抽象无具体实现 |
| SMELL-13 | Duplicate Code | 基于哈希的相似度（部分） |
| SMELL-14 | Middle Man | 委托比率 |
| SMELL-15 | Parallel Inheritance Hierarchies | 继承层级镜像（存根） |
| SMELL-16 | Comments | 注释与代码比率（存根） |
| SMELL-17 | Dead Code | 不可达/未使用检测（存根） |
| SMELL-18 | Feature Envy | 外部调用比率 |
| SMELL-19 | Inappropriate Intimacy | 跨类私有访问（存根） |
| SMELL-20 | Message Chains | 调用链深度 |
| SMELL-21 | God Object | 复合: LOC + 方法 + 耦合度 |
| SMELL-22 | Refused Bequest | 覆盖与空操作比率（存根） |
| SMELL-23 | Alternative Classes with Different Interfaces | 接口分歧（存根） |

---

## 开发环境搭建

```bash
# 克隆并构建（需要Rust 1.95及以上版本）
git clone https://github.com/epicsagas/Episteme.git
cd Episteme
cargo build

# 运行测试
cargo test

# 代码检查
cargo clippy -- -D warnings

# 本地安装（自动填充数据并构建数据库）
cargo install --path .
epis install --local
```

---

## API端点（17）

| 方法 | 路径 | 说明 |
|--------|------|-------------|
| GET | `/` | 服务信息 |
| GET | `/health` | 健康检查 |
| GET | `/live` | 存活探测 |
| GET | `/ready` | 就绪探测 |
| GET | `/stats` | 图谱统计 |
| POST | `/analyze` | 代码异味检测 |
| POST | `/refactor` | 重构建议 |
| GET | `/search` | 知识搜索 |
| POST | `/search` | 知识搜索（POST） |
| GET | `/graph/{id}` | 获取实体 |
| GET | `/graph/{id}/neighbors` | 获取相邻实体 |
| POST | `/graph/neighbors` | 获取相邻实体（POST） |
| POST | `/graph/subgraph` | 提取子图 |
| GET | `/graph/path` | 最短路径 |
| GET | `/graph/contradictions` | 发现矛盾 |
| POST | `/graph/infer-transitive` | 推断传递关系 |
| GET | `/metrics` | Prometheus指标 |

---

## 未来路线图

- **IDE插件** — VSCode、IntelliJ原生集成
- **自定义实体** — 添加团队特有的模式/异味
- **团队指标** — 聚合组织内的模式使用情况
- **多语言文档** — 韩语、日语、中文知识库
- **交互式教程** — MCP工具的应用内引导式教程

---

*最后更新: 2026-05-03*

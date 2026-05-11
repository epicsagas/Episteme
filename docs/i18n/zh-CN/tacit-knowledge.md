# 隐性知识架构

Episteme管理两个不同的知识层: **规范知识**（不可变、策划）和**隐性知识**（可变、用户贡献）。本文档描述双数据库架构、数据流和洞察生命周期。

## 概述

| | 规范知识 | 隐性知识（洞察） |
|---|---|---|
| **存储** | `~/.episteme/db/episteme.db` | `~/.episteme/user_knowledge.db` |
| **可变性** | 只读（通过`epis build`重建） | 读写（通过MCP实时操作） |
| **ID前缀** | `DP-NNN`、`RF-NNN`、`LAW-NNN`、`SMELL-NNN` | `TK-NNN` |
| **来源** | `raw/`中的策划Markdown文件 | MCP `add_insight`工具 / CLI `epis insight` |
| **实体** | 22个模式、66个重构、56条法则、23个异味 | 无限用户洞察 |

这两个数据库在物理上是分离的，但在运行时合并为单一的可遍历图谱。

## 双数据库设计

```
┌─────────────────────────────────┐     ┌──────────────────────────────┐
│  规范DB (episteme.db)     │     │  用户知识DB           │
│                                 │     │  (user_knowledge.db)         │
│  ┌───────────┐  ┌────────────┐  │     │  ┌────────────────────────┐  │
│  │  chunks   │  │ embeddings │  │     │  │  user_entities         │  │
│  │  (914)    │  │  (914)     │  │     │  │  (TK-xxx条目)      │  │
│  └───────────┘  └────────────┘  │     │  ├────────────────────────┤  │
│                                 │     │  │  user_relations        │  │
│  构建: epis build           │     │  ├────────────────────────┤  │
│  数据来源: raw/*.md       │     │  │  user_embeddings       │  │
│                                 │     │  ├────────────────────────┤  │
│  运行时不可变           │     │  │  user_entities_fts     │  │
│                                 │     │  │  (FTS5搜索索引)   │  │
└──────────────┬──────────────────┘     │  ├────────────────────────┤  │
               │                        │  │  insight_seq           │  │
               │                        │  │  (原子ID计数器)   │  │
               │                        │  └────────────────────────┘  │
               │                        │                              │
               │                        │  写入: MCP add_insight │
               │                        │  读取: search_insights    │
               │                        └──────────────┬───────────────┘
               │                                       │
               └───────────────┬───────────────────────┘
                               │
                    ┌──────────▼──────────┐
                    │   CompositeGraph    │
                    │   (内存合并) │
                    │                     │
                    │  - 统一实体   │
                    │    查找           │
                    │  - 跨层BFS  │
                    │  - 跨层      │
                    │    相邻查询 │
                    │                     │
                    │  处理所有MCP     │
                    │  工具请求              │
                    └─────────────────────┘
```

### 为什么要分离数据库？

1. **保护** — 用户输入不会破坏策划的规范知识。
2. **独立的生命周期** — 规范知识通过构建管道更新；隐性知识实时更新。
3. **可移植性** — 可以在不触及规范层的情况下跨机器或团队共享`user_knowledge.db`。

## CompositeGraph

`CompositeGraph`结构体（位于`src/domain/composite_graph.rs`）在启动时将两个层合并为单一的`GraphRepository`接口:

- 从`relations.json`加载规范`KnowledgeGraph`
- 通过`UserGraphStore`打开`user_knowledge.db`
- 提供跨两层的统一`get_entity()`、`get_neighbors()`、`find_path()`
- 用户操作从不修改规范图谱

### 优雅降级

如果`user_knowledge.db`无法打开（文件缺失、权限错误），系统会回退到仅规范模式。6个规范MCP工具继续工作；3个隐性知识工具返回错误。

## 用户知识Schema

```sql
-- 核心实体表
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,                    -- 例如: "TK-001"
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author TEXT NOT NULL DEFAULT 'user',
    confidence REAL NOT NULL DEFAULT 0.5,   -- 0.0 到 1.0
    evidence_count INTEGER NOT NULL DEFAULT 0,
    last_validated TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',        -- JSON数组
    relations TEXT NOT NULL DEFAULT '{}',   -- JSON: type -> [target_ids]
    created_at TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT '',
    link_provenance TEXT NOT NULL DEFAULT '{}'  -- JSON: entity_id -> metadata
);

-- 显式关系边
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    to_id TEXT NOT NULL,
    UNIQUE(from_id, relation_type, to_id)
);

-- 嵌入向量（f32，小端序）
CREATE TABLE user_embeddings (
    entity_id TEXT PRIMARY KEY,
    embedding BLOB NOT NULL
);

-- 全文搜索索引
CREATE VIRTUAL TABLE user_entities_fts USING fts5(
    title, content, tags,
    content=user_entities, content_rowid=rowid
);

-- 原子ID序列
CREATE TABLE insight_seq (key TEXT PRIMARY KEY, val INTEGER NOT NULL);
```

## MCP工具

### add_insight

从自由文本创建`TK-NNN`实体。系统自动执行:

1. **检测规范实体链接** — 两阶段关键词匹配（停用词过滤 + 复合评分）查找相关的模式、法则和异味。
2. **检查重复** — 与现有洞察进行比较。
3. **创建`derives_from`关系** — 对于高置信度链接（分数 >= 0.5），自动链接到规范实体。
4. **计算相关性** — 使用Jaccard相似度查找相关洞察。

参数:
- `text`（必需） — 自由文本洞察内容
- `project`（可选） — 项目名称标签
- `tags`（可选） — 分类标签
- `linked_entities`（可选） — 显式链接的实体ID（例如: `["DP-005", "SMELL-01"]`）

### search_insights

用户贡献洞察的FTS5关键词搜索。返回匹配的`TK-*`实体及其内容和关系。

参数:
- `query`（必需） — 自然语言搜索查询
- `limit`（可选） — 最大结果数（默认10，最大20）

### confirm_links

验证或拒绝洞察与规范实体之间自动检测的链接。每次确认:

- 提升洞察的置信度分数（每个确认链接+0.05，上限1.0）
- 记录链接来源（来源、分数、时间戳）
- 支持洞察之间的合并/替代关系

参数:
- `insight_id`（必需） — `TK-NNN`的ID
- `accepted`（必需） — 确认为有效链接的实体ID
- `rejected`（可选） — 拒绝的实体ID
- `merged_with`（可选） — 合并/替代目标的洞察ID

## 洞察生命周期

```
1. add_insight("决定在微服务拆分时先识别领域边界")
       │
       ▼
2. 自动检测链接: CONWAY-001 (康威定律)、DP-026 (Strangler Fig)
       │
       ▼
3. 创建TK-001，包含 derives_from → LAW-017、DP-026
       │
       ▼
4. confirm_links(insight_id="TK-001", accepted=["LAW-017"])
       │
       ▼
5. 置信度提升: 0.5 → 0.55
       │
       ▼
6. 后来: search_insights("微服务拆分") → 返回TK-001
       │
       ▼
7. find_path("TK-001", "SMELL-03") → 遍历跨层图谱
```

## 关系类型

| 关系 | 方向 | 说明 |
|----------|-----------|-------------|
| `derives_from` | TK → 规范 | 洞察基于规范实体 |
| `applies_to` | TK → 规范 | 洞察将模式/法则应用于特定上下文 |
| `supersedes` | TK → TK | 较新的洞察替代较旧的 |
| `related_to` | TK → TK/规范 | 一般语义连接 |

## CLI使用

```bash
# 添加洞察
epis insight add "团队在重构God Class时Extract Class不如Facade Pattern有效"

# 搜索洞察
epis insight search "认证中间件"

# 列出所有洞察
epis insight list
```

## 主要源文件

| 文件 | 角色 |
|------|------|
| `src/domain/composite_graph.rs` | 规范 + 用户层的运行时合并 |
| `src/adapters/user_graph_store.rs` | 基于SQLite的`MutableGraphRepository` |
| `src/server/mcp_insight.rs` | 3个隐性知识工具的MCP处理器 |
| `src/adapters/insight_utils.rs` | ID生成、时间戳、文本工具 |
| `src/domain/types.rs` | `UserEntity`、`LinkProvenance`、`EntityType::Insight` |
| `src/ports/graph.rs` | `MutableGraphRepository`特征（14个方法） |

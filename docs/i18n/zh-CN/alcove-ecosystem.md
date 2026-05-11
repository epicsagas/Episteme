# Alcove生态系统 — 架构与能力分析

> Episteme的隐性知识层（TK-*）与Alcove文档生态系统的详细比较，涵盖存储模型、搜索能力、生命周期管理和用例指导。

---

## 1. 架构概述

### Episteme隐性知识（TK-*）

| 方面 | 详情 |
|--------|--------|
| **存储** | SQLite单文件（`~/.episteme/user_knowledge.db`） |
| **Schema** | 5张表: `user_entities`、`user_relations`、`user_embeddings`、`user_entities_fts`（FTS5虚拟表）、`insight_seq` |
| **单位** | 一个洞察 = 一条`UserEntity`记录（TK-xxx ID） |
| **图谱** | 运行时通过`CompositeGraph`与规范图谱合并 — 实现跨层路径遍历（TK-001 → DP-005 → SMELL-01） |
| **并发** | `Mutex<Connection>` + WAL模式，支持MCP + CLI同时访问 |

### Alcove文档系统

| 方面 | 详情 |
|--------|--------|
| **存储** | 文件系统上的Markdown文件 + Tantivy BM25索引 + sqlite-vec嵌入 |
| **结构** | 3层分类: 核心（7）、补充（19）、公开（15）文件/项目 |
| **单位** | 一个结构化Markdown文件（PRD、ARCHITECTURE、DECISIONS等） |
| **图谱** | wikilink + 文件路径的松散连接 |
| **并发** | 每个文档根目录的基于文件的锁（`.index_lock`），每个库的索引隔离 |
| **库** | 指向Obsidian PARA文件夹的3个符号链接: areas（8个文档）、resources（71个）、zettelkasten（17个） |

---

## 2. 存储模型比较

### Episteme TK-* Schema

```sql
-- 核心表
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,           -- TK-001, TK-002, ...
    title TEXT,                    -- 自动: 第一行，最多80字符
    content TEXT,                  -- 自由文本（无长度限制）
    author TEXT DEFAULT 'user',
    confidence REAL DEFAULT 0.5,   -- 每确认链接+0.05，上限1.0
    evidence_count INTEGER DEFAULT 0,
    last_validated TEXT DEFAULT '',
    tags TEXT DEFAULT '[]',        -- JSON数组
    relations TEXT DEFAULT '{}',   -- JSON HashMap<relation_type, Vec<entity_id>>
    link_provenance TEXT DEFAULT '{}',
    created_at TEXT,
    updated_at TEXT
);

-- 规范化关系（derives_from、applies_to、supersedes）
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT,
    relation_type TEXT,
    to_id TEXT,
    UNIQUE(from_id, relation_type, to_id)
);

-- FTS5全文搜索
CREATE VIRTUAL TABLE user_entities_fts USING fts5(title, content, tags, content=user_entities);
```

### Alcove文件结构

```
~/.alcove/
  config.toml                    # 全局配置（docs_root、core/team/public文件列表、嵌入模型）
  docs -> symlink                # → Obsidian/SecondBrain/99-Archives/projects
  vaults/
    areas -> symlink             # → Obsidian/02-Areas (8个文档)
    resources -> symlink         # → Obsidian/03-Resources (71个文档)
    zettelkasten -> symlink      # → Obsidian/10-Zettelkasten (17个文档)
  models/                        # 缓存的ONNX嵌入模型
  logs/

<docs_root>/<project>/
  .alcove/
    index/                       # Tantivy BM25索引文件
    index_meta.json              # 文件指纹（mtime + size）
    vectors.db                   # sqlite-vec嵌入
  PRD.md                         # 产品需求
  ARCHITECTURE.md                # 系统设计
  PROGRESS.md                    # 里程碑与状态
  DECISIONS.md                   # 架构决策记录
  CONVENTIONS.md                 # 编码标准
  SECRETS_MAP.md                 # 环境变量与密钥
  DEBT.md                        # 技术债务登记
```

---

## 3. 知识特征

| 维度 | Episteme TK-* | Alcove |
|-----------|---------------|--------|
| **类型** | 瞬时洞察、经验教训、团队决策 | 结构化项目文档（需求、架构、决策） |
| **可变性** | 可变（SQLite CRUD） | 可变（文件编辑 + 索引重建） |
| **来源** | 用户贡献的自由文本 | 用户编写 + 从模板代理生成 |
| **权威性** | 个人/团队观察 | 团队决定 / 组织策略 |
| **粒度** | 原子化（每条一个洞察） | 分节（DECISIONS.md中有多个ADR） |
| **链接** | 自动检测到规范实体（关键词评分） | 手动wikilink + Markdown链接 |
| **版本控制** | 无（仅SQLite） | 基于Git（文件 = 事实来源） |

### 洞察生命周期（Episteme TK-*）

```
add_insight(text, tags?, project?, linked_entities?)
  │
  ├── 生成TK-xxx ID（原子序列）
  ├── detect_canonical_links() — 关键词匹配 → 前5个规范实体
  │     score >= 0.5 → 自动链接 (derives_from)
  │     score < 0.5 → 建议链接
  ├── FTS5重复检测 → DuplicateCandidate[]
  ├── 持久化到SQLite + 内存缓存
  └── 返回: { id, auto_links, suggested_links, duplicates, confidence }

confirm_links(id, accepted[], rejected[])
  │
  ├── 添加derives_from/applies_to关系
  ├── 将link_provenance来源升级为"manual"
  ├── 提升置信度（+0.05/链接，上限1.0）
  └── 持久化更新

search_insights(query, limit?)
  │
  └── FTS5 MATCH查询 → 排名结果
```

### 文档生命周期（Alcove）

```
init_project(project_name, project_path?)
  │
  ├── 从模板创建7个核心文档（PRD、ARCHITECTURE、...）
  ├── 可选创建公开文档（README、CHANGELOG、...）
  └── 重建搜索索引

validate_docs()
  │
  ├── 检查必需文件是否存在
  ├── 检查模板占位符（TODO、FIXME）
  ├── 检查必需章节标题
  ├── 检查最小列表项数量
  └── 返回: 每个文件 pass/warn/fail

lint_project()
  │
  ├── 检测损坏的[[wikilinks]]和Markdown链接
  ├── 查找孤立文件（未被任何文档链接）
  ├── 查找过时标记（WIP、TODO、FIXME、DRAFT、DEPRECATED）
  └── 查找过时的年份引用（2年以上）

audit_project()
  │
  ├── 扫描私有文档仓库中缺失的必需文档
  ├── 扫描公开项目仓库中暴露的内部文档
  ├── 将文件分类到各层级
  └── 返回: suggested_actions[]
```

---

## 4. 搜索能力

| 能力 | Episteme TK-* | Alcove |
|------------|---------------|--------|
| **引擎** | FTS5（关键词匹配） | Tantivy BM25 + sqlite-vec余弦相似度 |
| **融合** | 无 | RRF（倒数排名融合，k=60） |
| **CJK支持** | 无特殊支持 | NgramTokenizer（min=2，max=3） |
| **分块** | 不适用（一行 = 一个洞察） | 200-500字符块 |
| **增量** | 不适用（单表） | mtime + size指纹比较 |
| **向量搜索** | Schema存在（`user_embeddings`）但**未接入** | 完全运行（MultilingualE5Small，384d） |
| **范围** | 单一数据库 | 按项目或全局（跨项目） |
| **回退** | 无 | 无索引时使用grep子字符串匹配 |

---

## 5. 功能完整性

| 功能 | Episteme TK-* | Alcove |
|---------|---------------|--------|
| 创建 | `add_insight` | `init_project`、文件编辑 |
| 读取 | `search_insights`（仅搜索，无按ID获取） | `get_doc_file`、`search_project_docs` |
| 更新 | 未通过MCP暴露 | 直接文件编辑 + `rebuild_index` |
| 删除 | 未通过MCP暴露 | 文件删除 + `rebuild_index` |
| 验证 | 无 | `validate_docs`、`lint_project` |
| 审计 | 无 | `audit_project`（公开/私有分离） |
| 备份 | 无 | `backup_vault`（Git提交快照） |
| 导入 | 无 | `promote_document`（Obsidian → 文档仓库） |
| 策略 | 无 | 带强制级别的`policy.toml` |
| 模板 | 无 | 7核心 + 19补充 + 15公开 |

---

## 6. Alcove库系统

三个库，通过符号链接到Obsidian PARA结构:

| 库 | 链接目标 | 文档数 | 用途 |
|-------|--------|------|---------|
| `areas` | `02-Areas` | 8 | 领域: MCP代理、DevOps、Rust、LLM/RAG、开源 |
| `resources` | `03-Resources` | 71 | 参考资料: AWS、软件工程法则、技术文档 |
| `zettelkasten` | `10-Zettelkasten` | 17 | 原子笔记: AI架构、BM25、知识图谱、Rust模式 |

每个库拥有独立的:
- BM25索引（Tantivy）
- 向量数据库（sqlite-vec）
- 文件指纹跟踪（`index_meta.json`）
- 缓存隔离（独立的`OnceLock<Mutex<HashMap>>`）

---

## 7. Alcove配置系统

### 全局: `~/.alcove/config.toml`

```toml
docs_root = "/path/to/Obsidian/SecondBrain/99-Archives/projects"

[core]
files = ["PRD.md", "ARCHITECTURE.md", "PROGRESS.md", "DECISIONS.md",
         "CONVENTIONS.md", "SECRETS_MAP.md", "DEBT.md"]

[team]
files = ["ENV_SETUP.md", "ONBOARDING.md", "DATA_MODEL.md", "SCHEMA.md",
         "DEPLOYMENT.md", "RUNBOOK.md", "PLAYBOOK.md", "MONITORING.md", ...]  # 19个文件

[public]
files = ["README.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md", ...]  # 15个文件

[embedding]
model = "MultilingualE5Small"
auto_download = true
enabled = true
```

### 按项目: `alcove.toml`

覆盖全局默认值: `diagram_format`、`core_files`、`team_files`、`public_files`。

### 策略: `policy.toml`

定义:
- `enforce`级别: `strict` | `warn` | `off`
- 必需文档及其章节标题和最小项数
- 命名规范（`UPPER_SNAKE`、`lower_snake`、`kebab`、`free`）
- 优先级: 项目 > 团队 > 内置默认值

---

## 8. 用例决策矩阵

| 场景 | 推荐工具 | 理由 |
|-----------|-----------------|-----------|
| "记录一次生产事故的经验教训" | **Episteme TK-*** | 自动链接到相关异味/法则，便于未来交叉引用 |
| "为新项目启动文档" | **Alcove** `init_project` | 自动生成7个核心模板 |
| "检查是否有过时文档" | **Alcove** `lint_project` | 自动检测WIP/TODO/DEPRECATED/过时日期 |
| "查找团队对认证中间件的决策" | **Alcove** `search_project_docs` | 使用BM25 + 向量搜索结构化的DECISIONS.md |
| "检测模块中的代码异味" | **Episteme** `analyze_code` | 基于模式/正则的异味检测 |
| "确保PRD包含所有必需章节" | **Alcove** `validate_docs` | 基于策略的章节和项数验证 |
| "将洞察链接到策略模式" | **Episteme** `confirm_links` | 创建到规范实体的`derives_from`边 |
| "导入Obsidian笔记供代理访问" | **Alcove** `promote_document` | 带自动项目检测导入到文档仓库 |
| "查找SRP和Extract Class之间的关系" | **Episteme** `find_path` | 跨实体类型的多跳图谱遍历 |
| "备份项目文档状态" | **Alcove** `backup_vault` | 带时间戳的Git提交快照 |
| "审计公开仓库中是否暴露了内部文档" | **Alcove** `audit_project` | 扫描私有和公开两个位置 |
| "获取代码的排名重构建议" | **Episteme** `suggest_refactorings` | 复合评分: 严重性 × 工作量 × 原则匹配度 |

---

## 9. 互补角色

```
Episteme TK-*                     Alcove
"这里适用什么普遍               "我们团队对这件事
 的原则？"                       做了什么决定？"

 瞬时洞察 ←────────────→ 结构化决策记录
 关键词自动链接               基于模板的脚手架
 跨层图谱遍历         跨项目文档搜索
 代码分析 → 异味检测            文档分析 → 过时检测
```

**当两者同时活跃时**: Episteme提供普遍的"为什么"（法则、模式），Alcove提供项目特定的"我们决定了什么"（ADR、惯例）。代理应引用两个来源，当团队规则与通用指导冲突时，Alcove优先。

---

## 10. 规模与性能

| 指标 | Episteme TK-* | Alcove |
|--------|---------------|--------|
| **设计容量** | 数百个洞察 | ~10,000个文件 |
| **搜索延迟** | FTS5即时（内存中） | BM25概览 < 500ms |
| **Token效率** | 每结果一个洞察 | 前5个块约1.5k token（grep约8k） |
| **索引重建** | 不需要（FTS5触发器） | 增量: 仅变更的文件 |
| **模型大小** | 不适用（未接入） | 15MB（ArcticEmbedXS）到2.3GB（BGE-M3） |

---

*另见: [Alcove集成指南](./alcove-integration.md)了解使用模式和工作流示例。*

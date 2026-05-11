# 隱性知識架構

Episteme 管理兩個不同的知識層：**正典**（不可變、策展的）和**隱性**（可變、使用者貢獻的）。本文件描述雙資料庫架構、資料流程以及洞察的生命週期。

## 概覽

| | 正典知識 | 隱性知識（洞察） |
|---|---|---|
| **儲存** | `~/.episteme/db/episteme.db` | `~/.episteme/user_knowledge.db` |
| **可變性** | 唯讀（透過 `epis build` 重建） | 可讀寫（透過 MCP 即時更新） |
| **ID 前綴** | `DP-NNN`、`RF-NNN`、`LAW-NNN`、`SMELL-NNN` | `TK-NNN` |
| **來源** | `raw/` 中策展的 Markdown 檔案 | MCP `add_insight` 工具 / CLI `epis insight` |
| **實體** | 22 個模式、66 個重構手法、56 個法則、23 個壞味道 | 無限的使用者洞察 |

這兩個資料庫在實體上是分離的，但在執行時期合併為一個可遍歷的圖譜。

## 雙資料庫設計

```
┌─────────────────────────────────┐     ┌──────────────────────────────┐
│  正典資料庫（episteme.db）        │     │  使用者知識資料庫             │
│                                 │     │  (user_knowledge.db)         │
│  ┌───────────┐  ┌────────────┐  │     │  ┌────────────────────────┐  │
│  │  chunks   │  │ embeddings │  │     │  │  user_entities         │  │
│  │  (914)    │  │  (914)     │  │     │  │  (TK-xxx 條目)         │  │
│  └───────────┘  └────────────┘  │     │  ├────────────────────────┤  │
│                                 │     │  │  user_relations        │  │
│  建立者：epis build              │     │  ├────────────────────────┤  │
│  資料來源：raw/*.md              │     │  │  user_embeddings       │  │
│                                 │     │  ├────────────────────────┤  │
│  執行時期不可變                   │     │  │  user_entities_fts     │  │
│                                 │     │  │  (FTS5 搜尋索引)        │  │
└──────────────┬──────────────────┘     │  ├────────────────────────┤  │
               │                        │  │  insight_seq           │  │
               │                        │  │  (原子 ID 計數器)       │  │
               │                        │  └────────────────────────┘  │
               │                        │                              │
               │                        │  寫入者：MCP add_insight      │
               │                        │  讀取者：search_insights      │
               │                        └──────────────┬───────────────┘
               │                                       │
               └───────────────┬───────────────────────┘
                               │
                    ┌──────────▼──────────┐
                    │   CompositeGraph    │
                    │   (記憶體內合併)     │
                    │                     │
                    │  - 統一實體查詢      │
                    │  - 跨層 BFS         │
                    │  - 跨層鄰居查詢     │
                    │                     │
                    │  服務所有 MCP       │
                    │  工具請求            │
                    └─────────────────────┘
```

### 為什麼要分離資料庫？

1. **保護** — 使用者輸入無法損毀策展的正典知識。
2. **獨立生命週期** — 正典知識透過建置管線更新；隱性知識即時更新。
3. **可攜性** — 可跨機器或團隊共用 `user_knowledge.db`，而不觸及正典層。

## CompositeGraph

`CompositeGraph` 結構體（位於 `src/domain/composite_graph.rs`）在啟動時將兩個層合併為單一的 `GraphRepository` 介面：

- 從 `relations.json` 載入正典 `KnowledgeGraph`
- 透過 `UserGraphStore` 開啟 `user_knowledge.db`
- 提供跨兩個層的統一 `get_entity()`、`get_neighbors()`、`find_path()`
- 使用者操作永不修改正典圖譜

### 優雅回退

若 `user_knowledge.db` 無法開啟（檔案缺少、權限錯誤），系統會回退至僅正典模式。所有 6 個正典 MCP 工具繼續運作；3 個隱性知識工具回傳錯誤。

## 使用者知識結構描述

```sql
-- 核心實體資料表
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,                    -- 例如 "TK-001"
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author TEXT NOT NULL DEFAULT 'user',
    confidence REAL NOT NULL DEFAULT 0.5,   -- 0.0 到 1.0
    evidence_count INTEGER NOT NULL DEFAULT 0,
    last_validated TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',        -- JSON 陣列
    relations TEXT NOT NULL DEFAULT '{}',   -- JSON: 類型 -> [目標 ID]
    created_at TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT '',
    link_provenance TEXT NOT NULL DEFAULT '{}'  -- JSON: 實體 ID -> 元資料
);

-- 明確關聯邊
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

-- 全文搜尋索引
CREATE VIRTUAL TABLE user_entities_fts USING fts5(
    title, content, tags,
    content=user_entities, content_rowid=rowid
);

-- 原子 ID 序列
CREATE TABLE insight_seq (key TEXT PRIMARY KEY, val INTEGER NOT NULL);
```

## MCP 工具

### add_insight

從自由文字建立 `TK-NNN` 實體。系統會自動：

1. **偵測正典實體連結** — 兩階段關鍵字比對（停用詞過濾 + 複合評分）尋找相關的模式、法則和壞味道。
2. **檢查重複項** — 與現有洞察進行比較。
3. **建立 `derives_from` 關聯** — 對於高信心度的連結（分數 >= 0.5），自動連結至正典實體。
4. **計算關聯性** — 使用 Jaccard 相似度尋找相關洞察。

參數：
- `text`（必要）— 自由文字洞察內容
- `project`（選填）— 專案名稱標籤
- `tags`（選填）— 分類標籤
- `linked_entities`（選填）— 要連結的明確實體 ID（例如 `["DP-005", "SMELL-01"]`）

### search_insights

對使用者貢獻的洞察進行 FTS5 關鍵字搜尋。回傳符合條件的 `TK-*` 實體及其內容和關聯。

參數：
- `query`（必要）— 自然語言搜尋查詢
- `limit`（選填）— 最大結果數（預設 10，上限 20）

### confirm_links

驗證或拒絕洞察與正典實體之間自動偵測的連結。每次確認：

- 提升洞察的信心度分數（每個確認連結 +0.05，上限 1.0）
- 記錄連結來源（來源、分數、時間戳記）
- 支援洞察之間的合併/取代關聯

參數：
- `insight_id`（必要）— `TK-NNN` ID
- `accepted`（必要）— 確認為有效連結的實體 ID
- `rejected`（選填）— 要拒絕的實體 ID
- `merged_with`（選填）— 合併/取代的目標洞察 ID

## 洞察生命週期

```
1. add_insight("마이크로서비스 분리 시 도메인 경계를 먼저 식별하기로 결정")
       │
       ▼
2. 自動偵測連結：CONWAY-001（Conway's Law）、DP-026（Strangler Fig）
       │
       ▼
3. 建立 TK-001，帶有 derives_from → LAW-017、DP-026
       │
       ▼
4. confirm_links(insight_id="TK-001", accepted=["LAW-017"])
       │
       ▼
5. 信心度提升：0.5 → 0.55
       │
       ▼
6. 稍後：search_insights("마이크로서비스 분리") → 回傳 TK-001
       │
       ▼
7. find_path("TK-001", "SMELL-03") → 遍歷跨層圖譜
```

## 關聯類型

| 關聯 | 方向 | 說明 |
|------|------|------|
| `derives_from` | TK → 正典 | 洞察基於一個正典實體 |
| `applies_to` | TK → 正典 | 洞察將一個模式/法則應用至特定情境 |
| `supersedes` | TK → TK | 較新的洞察取代較舊的洞察 |
| `related_to` | TK → TK/正典 | 一般語意連結 |

## CLI 使用方式

```bash
# 新增洞察
epis insight add "팀에서 God Class 리팩토링 시 Extract Class보다 Facade Pattern이 효과적이었음"

# 搜尋洞察
epis insight search "인증 미들웨어"

# 列出所有洞察
epis insight list
```

## 關鍵原始碼檔案

| 檔案 | 角色 |
|------|------|
| `src/domain/composite_graph.rs` | 執行時期合併正典 + 使用者層 |
| `src/adapters/user_graph_store.rs` | 基於 SQLite 的 `MutableGraphRepository` |
| `src/server/mcp_insight.rs` | 3 個隱性知識工具的 MCP 處理器 |
| `src/adapters/insight_utils.rs` | ID 產生、時間戳記、文字工具 |
| `src/domain/types.rs` | `UserEntity`、`LinkProvenance`、`EntityType::Insight` |
| `src/ports/graph.rs` | `MutableGraphRepository` 特徵（14 個方法） |

# Alcove 生態系 — 架構與能力分析

> 針對 Episteme 的隱性知識層（TK-*）與 Alcove 文件生態系統進行詳細比較，涵蓋儲存模型、搜尋能力、生命週期管理及使用情境指引。

---

## 1. 架構概覽

### Episteme 隱性知識（TK-*）

| 面向 | 詳細資訊 |
|------|----------|
| **儲存** | SQLite 單一檔案（`~/.episteme/user_knowledge.db`） |
| **結構描述** | 5 個資料表：`user_entities`、`user_relations`、`user_embeddings`、`user_entities_fts`（FTS5 虛擬表）、`insight_seq` |
| **單位** | 一個洞察 = 一筆 `UserEntity` 列（TK-xxx ID） |
| **圖譜** | 在執行時期透過 `CompositeGraph` 與正典圖譜合併 — 支援跨層路徑遍歷（TK-001 → DP-005 → SMELL-01） |
| **並行性** | `Mutex<Connection>` + WAL 模式，支援 MCP + CLI 同時存取 |

### Alcove 文件系統

| 面向 | 詳細資訊 |
|------|----------|
| **儲存** | 檔案系統上的 Markdown 檔案 + Tantivy BM25 索引 + sqlite-vec 嵌入 |
| **結構** | 3 層分類：核心（7）、補充（19）、公開（15）個檔案/專案 |
| **單位** | 一個結構化 Markdown 檔案（PRD、ARCHITECTURE、DECISIONS 等） |
| **圖譜** | wikilink + 檔案路徑為基礎的鬆散連結 |
| **並行性** | 每個文件根目錄的檔案鎖（`.index_lock`），每個保險庫獨立的索引隔離 |
| **保險庫** | 3 個指向 Obsidian PARA 資料夾的符號連結：areas（8 份文件）、resources（71 份）、zettelkasten（17 份） |

---

## 2. 儲存模型比較

### Episteme TK-* 結構描述

```sql
-- 核心資料表
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,           -- TK-001, TK-002, ...
    title TEXT,                    -- 自動：第一行，最多 80 字元
    content TEXT,                  -- 自由文字（無最大長度限制）
    author TEXT DEFAULT 'user',
    confidence REAL DEFAULT 0.5,   -- 每個確認連結 +0.05，上限 1.0
    evidence_count INTEGER DEFAULT 0,
    last_validated TEXT DEFAULT '',
    tags TEXT DEFAULT '[]',        -- JSON 陣列
    relations TEXT DEFAULT '{}',   -- JSON HashMap<relation_type, Vec<entity_id>>
    link_provenance TEXT DEFAULT '{}',
    created_at TEXT,
    updated_at TEXT
);

-- 正規化關聯（derives_from、applies_to、supersedes）
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT,
    relation_type TEXT,
    to_id TEXT,
    UNIQUE(from_id, relation_type, to_id)
);

-- FTS5 全文搜尋
CREATE VIRTUAL TABLE user_entities_fts USING fts5(title, content, tags, content=user_entities);
```

### Alcove 檔案結構

```
~/.alcove/
  config.toml                    # 全域設定（docs_root、core/team/public 檔案清單、嵌入模型）
  docs -> 符號連結               # → Obsidian/SecondBrain/99-Archives/projects
  vaults/
    areas -> 符號連結             # → Obsidian/02-Areas（8 份文件）
    resources -> 符號連結         # → Obsidian/03-Resources（71 份文件）
    zettelkasten -> 符號連結      # → Obsidian/10-Zettelkasten（17 份文件）
  models/                        # 快取的 ONNX 嵌入模型
  logs/

<docs_root>/<project>/
  .alcove/
    index/                       # Tantivy BM25 索引檔案
    index_meta.json              # 檔案指紋（mtime + size）
    vectors.db                   # sqlite-vec 嵌入
  PRD.md                         # 產品需求
  ARCHITECTURE.md                # 系統設計
  PROGRESS.md                    # 里程碑與狀態
  DECISIONS.md                   # 架構決策記錄
  CONVENTIONS.md                 # 程式碼標準
  SECRETS_MAP.md                 # 環境變數與密鑰
  DEBT.md                        # 技術債務登記
```

---

## 3. 知識特性

| 維度 | Episteme TK-* | Alcove |
|------|---------------|--------|
| **類型** | 當下洞察、經驗教訓、團隊決策 | 結構化專案文件（需求、架構、決策） |
| **可變性** | 可變（SQLite CRUD） | 可變（檔案編輯 + 索引重建） |
| **來源** | 使用者貢獻的自由文字 | 使用者撰寫 + 代理從範本產生 |
| **權威性** | 個人/團隊觀察 | 團隊授權/組織政策 |
| **粒度** | 原子級（每筆一個洞察） | 分段式（每份 DECISIONS.md 多個 ADR） |
| **連結** | 自動偵測至正典實體（關鍵字評分） | 手動 wikilink + Markdown 連結 |
| **版本控制** | 無（僅 SQLite） | 基於 Git（檔案 = 真實來源） |

### 洞察生命週期（Episteme TK-*）

```
add_insight(text, tags?, project?, linked_entities?)
  │
  ├── 產生 TK-xxx ID（原子序列）
  ├── detect_canonical_links() — 關鍵字比對 → 前 5 個正典實體
  │     分數 >= 0.5 → 自動連結（derives_from）
  │     分數 < 0.5 → 建議連結
  ├── FTS5 重複偵測 → DuplicateCandidate[]
  ├── 持久化至 SQLite + 記憶體內快取
  └── 回傳：{ id, auto_links, suggested_links, duplicates, confidence }

confirm_links(id, accepted[], rejected[])
  │
  ├── 新增 derives_from/applies_to 關聯
  ├── 將 link_provenance 來源升級為 "manual"
  ├── 提升信心度（每個連結 +0.05，上限 1.0）
  └── 持久化更新

search_insights(query, limit?)
  │
  └── FTS5 MATCH 查詢 → 排序結果
```

### 文件生命週期（Alcove）

```
init_project(project_name, project_path?)
  │
  ├── 從範本建立 7 份核心文件（PRD、ARCHITECTURE 等）
  ├── 選擇性建立公開文件（README、CHANGELOG 等）
  └── 重建搜尋索引

validate_docs()
  │
  ├── 檢查必要檔案是否存在
  ├── 檢查範本佔位符（TODO、FIXME）
  ├── 檢查必要的章節標題
  ├── 檢查最低清單項目數量
  └── 回傳：每個檔案的 pass/warn/fail

lint_project()
  │
  ├── 偵測損壞的 [[wikilink]] 和 Markdown 連結
  ├── 尋找孤立檔案（未被任何文件連結）
  ├── 尋找過時標記（WIP、TODO、FIXME、DRAFT、DEPRECATED）
  └── 尋找過時的年份參考（2 年以上）

audit_project()
  │
  ├── 掃描私有文件庫中缺少的必要文件
  ├── 掃描公開專案庫中暴露的內部文件
  ├── 將檔案分類至各層級
  └── 回傳：suggested_actions[]
```

---

## 4. 搜尋能力

| 能力 | Episteme TK-* | Alcove |
|------|---------------|--------|
| **引擎** | FTS5（關鍵字比對） | Tantivy BM25 + sqlite-vec 餘弦相似度 |
| **融合** | 無 | RRF（倒數排名融合，k=60） |
| **CJK** | 無特殊支援 | NgramTokenizer（最小=2，最大=3） |
| **分段** | 不適用（一列 = 一個洞察） | 200-500 字元分段 |
| **增量** | 不適用（單一資料表） | mtime + size 指紋比較 |
| **向量搜尋** | 結構描述存在（`user_embeddings`）但**尚未連接** | 完全運作（MultilingualE5Small，384 維） |
| **範圍** | 單一資料庫 | 每個專案或全域（跨專案） |
| **回退** | 無 | 無索引時使用 grep 子字串比對 |

---

## 5. 功能完整性

| 功能 | Episteme TK-* | Alcove |
|------|---------------|--------|
| 建立 | `add_insight` | `init_project`、檔案編輯 |
| 讀取 | `search_insights`（僅搜尋，無法依 ID 取得） | `get_doc_file`、`search_project_docs` |
| 更新 | 未透過 MCP 公開 | 直接檔案編輯 + `rebuild_index` |
| 刪除 | 未透過 MCP 公開 | 檔案刪除 + `rebuild_index` |
| 驗證 | 無 | `validate_docs`、`lint_project` |
| 稽核 | 無 | `audit_project`（公開/私有分離） |
| 備份 | 無 | `backup_vault`（Git 提交快照） |
| 匯入 | 無 | `promote_document`（Obsidian → 文件庫） |
| 政策 | 無 | `policy.toml` 搭配強制等級 |
| 範本 | 無 | 7 個核心 + 19 個補充 + 15 個公開 |

---

## 6. Alcove 保險庫系統

三個保險庫，以符號連結至 Obsidian PARA 結構：

| 保險庫 | 目標 | 文件數 | 用途 |
|--------|------|--------|------|
| `areas` | `02-Areas` | 8 | 領域區域：MCP 代理、DevOps、Rust、LLM/RAG、開源 |
| `resources` | `03-Resources` | 71 | 參考資料：AWS、軟體工程法則、技術文件 |
| `zettelkasten` | `10-Zettelkasten` | 17 | 原子筆記：AI 架構、BM25、知識圖譜、Rust 模式 |

每個保險庫擁有獨立的：
- BM25 索引（Tantivy）
- 向量資料庫（sqlite-vec）
- 檔案指紋追蹤（`index_meta.json`）
- 快取隔離（獨立的 `OnceLock<Mutex<HashMap>>`）

---

## 7. Alcove 設定系統

### 全域：`~/.alcove/config.toml`

```toml
docs_root = "/path/to/Obsidian/SecondBrain/99-Archives/projects"

[core]
files = ["PRD.md", "ARCHITECTURE.md", "PROGRESS.md", "DECISIONS.md",
         "CONVENTIONS.md", "SECRETS_MAP.md", "DEBT.md"]

[team]
files = ["ENV_SETUP.md", "ONBOARDING.md", "DATA_MODEL.md", "SCHEMA.md",
         "DEPLOYMENT.md", "RUNBOOK.md", "PLAYBOOK.md", "MONITORING.md", ...]  # 19 個檔案

[public]
files = ["README.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md", ...]  # 15 個檔案

[embedding]
model = "MultilingualE5Small"
auto_download = true
enabled = true
```

### 每個專案：`alcove.toml`

覆寫全域預設值：`diagram_format`、`core_files`、`team_files`、`public_files`。

### 政策：`policy.toml`

定義：
- `enforce` 等級：`strict` | `warn` | `off`
- 必要文件及其章節標題和最低項目數量
- 命名慣例（`UPPER_SNAKE`、`lower_snake`、`kebab`、`free`）
- 優先順序：專案 > 團隊 > 內建預設值

---

## 8. 使用情境決策矩陣

| 情境 | 建議工具 | 理由 |
|------|----------|------|
| 「記錄生產事故的經驗教訓」 | **Episteme TK-*** | 自動連結至相關的壞味道/法則，以便未來交叉參照 |
| 「為新專案建立文件」 | **Alcove** `init_project` | 自動產生 7 份核心範本 |
| 「檢查是否有文件過時」 | **Alcove** `lint_project` | 自動偵測 WIP/TODO/DEPRECATED/過時日期 |
| 「找出團隊對身份驗證中介層的決策」 | **Alcove** `search_project_docs` | 使用 BM25 + 向量搜尋結構化的 DECISIONS.md |
| 「偵測模組中的程式碼壞味道」 | **Episteme** `analyze_code` | 基於模式/正規表示式的壞味道偵測 |
| 「確保 PRD 包含所有必要章節」 | **Alcove** `validate_docs` | 基於政策的章節和項目數量驗證 |
| 「將洞察連結至 Strategy 模式」 | **Episteme** `confirm_links` | 建立指向正典實體的 `derives_from` 邊 |
| 「匯入 Obsidian 筆記供代理存取」 | **Alcove** `promote_document` | 匯入至文件庫，自動偵測專案 |
| 「尋找 SRP 和 Extract Class 之間的關聯」 | **Episteme** `find_path` | 跨實體類型的多跳圖譜遍歷 |
| 「備份專案文件狀態」 | **Alcove** `backup_vault` | 帶有時間戳記的 Git 提交快照 |
| 「稽核公開庫中暴露的內部文件」 | **Alcove** `audit_project` | 掃描私有和公開兩個位置 |
| 「取得程式碼的排序重構建議」 | **Episteme** `suggest_refactorings` | 複合評分：嚴重性 x 工作量 x 原則對齊度 |

---

## 9. 互補角色

```
Episteme TK-*                     Alcove
「哪些通用原則                    「我們團隊針對這件事
 適用於此？」                      做了什麼決策？」

 即時洞察 ←────────────→ 結構化決策記錄
 關鍵字自動連結                    範本式鷹架
 跨層圖譜遍歷                      跨專案文件搜尋
 程式碼分析 → 壞味道偵測           文件分析 → 過時偵測
```

**當兩者同時運作時**：Episteme 提供通用的「為什麼」（法則、模式），Alcove 提供專案特定的「我們決定了什麼」（ADR、慣例）。代理應同時引用兩個來源，當團隊規則與通用指引衝突時，以 Alcove 為優先。

---

## 10. 規模與效能

| 指標 | Episteme TK-* | Alcove |
|------|---------------|--------|
| **設計容量** | 數百個洞察 | 約 10,000 個檔案 |
| **搜尋延遲** | FTS5 即時（記憶體內） | BM25 概覽 < 500ms |
| **Token 效率** | 每個結果一個洞察 | 前 5 個分段約 1.5k token（grep 約 8k） |
| **索引重建** | 不需要（FTS5 觸發器） | 增量：僅已變更的檔案 |
| **模型大小** | 不適用（未連接） | 15MB（ArcticEmbedXS）至 2.3GB（BGE-M3） |

---

*另見：[Alcove 整合指南](./alcove-integration.md) 以了解使用模式和工作流程範例。*

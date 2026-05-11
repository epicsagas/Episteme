# Episteme 開發指南

**專案：** Episteme v0.1.0
**語言：** Rust（edition 2024）
**最後更新：** 2026-05-03

---

## 目前狀態

| 元件 | 狀態 | 詳細資訊 |
|------|------|----------|
| **知識庫** | 完成 | 22 個模式、66 個重構手法、56 條法則、23 個壞味道、201 個關聯 |
| **程式碼壞味道偵測** | 生產就緒 | 16 個偵測器函式、10 種語言 |
| **REST API** | 生產就緒 | 17 個端點（axum）、速率限制、認證 |
| **MCP 伺服器** | 生產就緒 | 6 個工具、stdio + HTTP 傳輸 |
| **RAG 管線** | 生產就緒 | SQLite + FTS5 + fastembed（ONNX） |
| **圖譜視覺化** | 生產就緒 | 帶有 D3-force 的互動式 Web UI |

---

## 架構

六角架構（連接埠與配接器）：

```
src/
├── commands/          # CLI 子指令處理常式（clap）
│   ├── analysis.rs    # analyze, infer
│   ├── build.rs       # build（RAG 管線）
│   ├── explore.rs     # explore（搜尋/REPL）
│   ├── graph.rs       # 圖譜查詢
│   ├── install.rs     # 安裝精靈（TUI）
│   ├── service.rs     # MCP HTTP 常駐服務管理
│   └── other.rs       # api, mcp, web, telemetry, hooks
├── adapters/          # 基礎設施層
│   ├── regex_parsers.rs   # GenericParser（10 種語言，OnceLock 正則快取）
│   ├── python_ast_parser.rs  # Python AST（rustpython-parser）
│   ├── search_engines.rs  # FTS5 關鍵字 + 餘弦相似度
│   ├── service.rs         # MCP HTTP 常駐服務
│   ├── sqlite_db.rs       # SQLite 連線池
│   ├── cache.rs           # Redis 快取（選用）
│   └── ...
├── domain/            # 業務邏輯（無外部依賴）
│   ├── graph.rs       # KnowledgeGraph（BFS、子圖、矛盾、Jaccard）
│   ├── detectors.rs   # 16 個帶有 TieredAccum 的壞味道偵測器
│   ├── engine.rs      # RefactoringInferenceEngine + RefactoringRanker
│   ├── summarizer.rs  # 詳細層級回應最佳化
│   └── types.rs       # EntityType、RelationType、核心型別
├── server/            # HTTP 層（axum）
│   ├── api_routes.rs  # 17 個 REST 端點
│   ├── mcp_handler.rs # MCP 精簡外觀層
│   ├── mcp_search.rs  # 搜尋服務
│   ├── mcp_graph.rs   # 圖譜服務
│   └── mcp_analysis.rs # 程式碼分析服務
└── ports/             # 特徵（六角邊界）
    ├── parser.rs      # CodeParser 特徵
    ├── search.rs      # SearchEngine 特徵
    ├── graph.rs       # GraphStore 特徵
    └── embeddings.rs  # EmbeddingProvider 特徵
```

---

## 技術堆疊

| 元件 | 技術 | 用途 |
|------|------|------|
| **語言** | Rust（edition 2024） | 安全性、效能、單一二進位檔 |
| **Web 框架** | axum | REST API + MCP HTTP 傳輸 |
| **資料庫** | rusqlite（內建 SQLite） | 知識圖譜 + 向量儲存 |
| **搜尋** | FTS5 + 餘弦相似度 | 關鍵字 + 語意混合搜尋 |
| **嵌入模型** | fastembed（ONNX Runtime） | 本地、零設定嵌入生成 |
| **CLI** | clap（derive） | 15 個子指令 |
| **Python AST** | rustpython-parser | 基於 AST 的 Python 壞味道偵測 |
| **其他語言** | regex（OnceLock 快取） | GenericParser 框架 |

---

## 程式碼壞味道偵測器（16 個）

| ID | 壞味道 | 偵測方式 |
|----|--------|----------|
| SMELL-01 | Long Method | LOC 閾值 |
| SMELL-02 | Long Parameter List | 參數數量 |
| SMELL-03 | Primitive Obsession | 基本型別參數比例 |
| SMELL-04 | Large Class | 方法 + 欄位數量 |
| SMELL-05 | Data Clumps | 重複參數群組（樁） |
| SMELL-06 | Switch Statements | switch/match 數量 |
| SMELL-07 | Data Class | 方法與欄位比例 |
| SMELL-08 | Temporary Field | 條件欄位使用（樁） |
| SMELL-09 | Shotgun Surgery | 變更耦合（樁） |
| SMELL-10 | Divergent Change | 方法內聚性指標 |
| SMELL-11 | Lazy Class | 低 LOC + 方法數量 |
| SMELL-12 | Speculative Generality | 抽象但無具體實作 |
| SMELL-13 | Duplicate Code | 基於雜湊的相似度（部分） |
| SMELL-14 | Middle Man | 委派比例 |
| SMELL-15 | Parallel Inheritance Hierarchies | 繼承鏡像（樁） |
| SMELL-16 | Comments | 註解與程式碼比例（樁） |
| SMELL-17 | Dead Code | 不可達/未使用偵測（樁） |
| SMELL-18 | Feature Envy | 外部呼叫比例 |
| SMELL-19 | Inappropriate Intimacy | 跨類別私有存取（樁） |
| SMELL-20 | Message Chains | 呼叫鏈深度 |
| SMELL-21 | God Object | 綜合：LOC + 方法 + 耦合 |
| SMELL-22 | Refused Bequest | 覆寫為空的比例（樁） |
| SMELL-23 | Alternative Classes with Different Interfaces | 介面分歧（樁） |

---

## 開發環境設定

```bash
# 複製並建置（需要 Rust 1.95+）
git clone https://github.com/epicsagas/Episteme.git
cd Episteme
cargo build

# 執行測試
cargo test

# Lint
cargo clippy -- -D warnings

# 本地安裝（自動種子資料並建置資料庫）
cargo install --path .
epis install --local
```

---

## API 端點（17 個）

| 方法 | 路徑 | 說明 |
|------|------|------|
| GET | `/` | 服務資訊 |
| GET | `/health` | 健康檢查 |
| GET | `/live` | 存活探測 |
| GET | `/ready` | 就緒探測 |
| GET | `/stats` | 圖譜統計 |
| POST | `/analyze` | 程式碼壞味道偵測 |
| POST | `/refactor` | 重構建議 |
| GET | `/search` | 知識搜尋 |
| POST | `/search` | 知識搜尋（POST） |
| GET | `/graph/{id}` | 取得實體 |
| GET | `/graph/{id}/neighbors` | 取得鄰居 |
| POST | `/graph/neighbors` | 取得鄰居（POST） |
| POST | `/graph/subgraph` | 擷取子圖 |
| GET | `/graph/path` | 最短路徑 |
| GET | `/graph/contradictions` | 尋找矛盾 |
| POST | `/graph/infer-transitive` | 推論傳遞關聯 |
| GET | `/metrics` | Prometheus 指標 |

---

## 未來路線圖

- **IDE 外掛** — VSCode、IntelliJ 原生整合
- **自訂實體** — 新增團隊特定的模式/壞味道
- **團隊指標** — 跨組織彙整模式使用情形
- **多語言文件** — 韓語、日語、中文知識庫
- **互動式教學** — 應用程式內的 MCP 工具導覽

---

*最後更新：2026-05-03*

# 變更日誌

所有 Episteme 的顯著變更都將記錄於此檔案中。

本格式基於 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，
且本專案遵循 [語意化版本](https://semver.org/spec/v2.0.0.html)。

## [未發佈]

### 變更

- CLI：`explore` 更名為 `search`（舊名稱作為棄用別名保留）
- CLI：`mcp` 與 `api` 現在擁有完整的服務生命週期管理（`start`、`stop`、`restart`、`status`、`enable [--now]`、`disable [--now]`）
- CLI：`service` 頂層指令已棄用 — 請改用 `mcp start/stop/restart/status/enable/disable`
- CLI：`mcp --http` 已棄用 — 請改用 `mcp start` 以啟動 HTTP 常駐模式
- CLI：`launchd-install/uninstall/status` 已棄用 — 請改用 `mcp enable/disable/status`
- `enable/disable` 現已跨平台支援：macOS（launchd）與 Linux（systemd 使用者單元）

### 新增

- `api start/stop/restart/status/enable/disable` — REST API 常駐服務生命週期管理
- Linux systemd 使用者單元生成，用於 `mcp enable`

- **Claude Code 的 MCP HTTP 傳輸** — 傳輸選擇器 TUI、HTTP 為預設值、launchd 自動啟用
- **代理提示詞自動安裝** — `epis install` 將 Episteme 代理提示詞複製至 `~/.claude/agents/`
- **實體描述** — 描述欄位自動從 Markdown 原始檔案擷取，顯示於 Web 檢視器詳細面板
- **基準測試視覺化 SPA** — 趨勢分析、查詢分解儀表板
- **Web 檢視器重新設計** — Sankey 圖佈局、側邊欄樹狀結構、詳細面板、子圖可讀性改善
- **MCP 設定更新插入** — 再次執行 `epis install` 時會在設定不同時更新傳輸方式（stdio ↔ HTTP）
- **MCP yaml 設定** — `config.yaml` 中的 `mcp.host` / `mcp.port`（yaml → 環境變數備援）
- **監控** — 透過環境變數支援原生與遠端 Prometheus 抓取目標
- **CI 強化** — cargo audit、gitleaks、SBOM 生成、固定動作 SHA
- **發行管線** — Windows 目標、crates.io 發佈、Homebrew tap
- `examples/` 中的 **God module 架構診斷範例**

### 變更

- **安裝精靈** — 所有步驟（傳輸、Redis、遙測）遷移至全螢幕 TUI
- **安裝流程** — 種子資料後自動建置 RAG 索引，當資料庫已存在時跳過
- **知識圖譜** — 增強跨實體語意關聯
- **授權條款** — MIT → Apache-2.0

### 修復

- 遙測的同步 `main()` 中 Tokio runtime 恐慌
- 搜尋品質 — NDCG 測量錯誤已解決，hit@1 準確度提升至 100%
- 搜尋召回率 — 跨類型提升、稀疏實體處理、意圖同義詞
- fastembed 模型快取固定至 `~/.episteme/models`
- launchd 啟動 UID 替換與連接埠佔用處理
- CORS 來源現在可透過 `EPISTEME_CORS_ORIGINS` 設定

## [0.1.0] - 2026-05-03

### 新增

- **完整 Rust 重寫** — 以慣用 Rust 完全取代 Python 程式碼庫
- **六角架構** — `ports/`（特徵）、`domain/`（業務邏輯）、`adapters/`（基礎設施）、`server/`（HTTP）
- **GenericParser 框架** — 8 個基於大括號的解析器合併為帶有 `ParserConfig` 的 `GenericParser`；正則表達式透過 `OnceLock` 與 `Box::leak` 快取
- **Python AST 解析** — `rustpython-parser` 用於精確的 Python 壞味道偵測（Long Method、Large Class、God Object）
- **TieredAccum + build_detection()** — 在 `detectors.rs` 中去重 14 個相同的壞味道偵測建構（1,253 → 591 行）
- **MCP 模組分解** — 將 `EpistemeMCP`（675 行）拆分為 `mcp_search`、`mcp_graph`、`mcp_analysis` 服務
- **CLI 指令分解** — 將 `main.rs`（1,741 行）拆分為帶有 `cli.rs` clap 定義的 `commands/` 模組
- **API 處理常式去重** — 將重複的 `search`/`search_post` 合併為共享的 `do_search()`
- **16 個壞味道偵測器函式** — 從 14 個增加，涵蓋所有 GoF 壞味道類別
- **17 個 REST API 端點** — 健康探測、Prometheus 指標、CORS、速率限制
- **速率限制器 TTL 驅逐** — MAX_BUCKETS=10,000 加上 1 小時 TTL 以防止無限記憶體增長
- **ReDoS 緩解** — 將三元運算子正則表達式從 `[^:]+` 限制為 `[^:\n]{1,50}`
- **本地嵌入** — fastembed（ONNX Runtime）實現零設定語意搜尋
- **互動式安裝精靈** — 帶有 crossterm、vim 按鍵綁定、替代畫面的 TUI
- **發行封裝** — `episteme dist` 指令用於建立帶有自動資料庫啟動的發行封存檔
- **跨平台 CI** — GitHub Actions 發行工作流程支援 linux/macOS（x86_64 + aarch64）
- **多階段 Dockerfile** — Rust 建置器 + 精簡 Debian 執行環境

### 變更

- **語言**: Python 3.11+ → Rust（edition 2024）
- **Web 框架**: FastAPI → axum
- **資料庫**: Python sqlite3 → rusqlite（內建）
- **嵌入模型**: sentence-transformers/PyTorch → fastembed/ONNX Runtime
- **CLI**: argparse → clap（derive）
- **所有正則表達式已快取** — 透過全域 `REGEX_CACHE` 在熱路徑上零重新編譯

### 移除

- Python 執行環境依賴
- ChromaDB 依賴
- tree-sitter 依賴
- PyPI 發佈工作流程
- `episteme-hook` 獨立執行檔（原為 Python-only PyPI 進入點）— 請改用 `episteme hooks ground|sniff|audit`

## [0.0.5] - 2026-04-30

### 新增

- 帶有 D3-force 的圖譜視覺化 Web UI（`episteme web`）
- 發行封存檔中的預建向量資料庫
- `epis install --local` 旗標用於開發工作流程
- 涵蓋所有 161 個實體的 650+ 語意關聯
- CI 在發行期間自動生成向量資料庫

## [0.0.4] - 2026-04-29

### 新增

- 帶有 6 個工具的 MCP 伺服器
- 4 個專門化代理
- `epis install` 指令
- `epis service` 常駐服務管理
- 混合搜尋（FTS5 + 向量）
- Redis 快取、GPU 加速
- 10 種語言的程式碼壞味道偵測
- Prometheus + Grafana 監控

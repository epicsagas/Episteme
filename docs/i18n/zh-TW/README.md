<p align="center">
<img src="../assets/icon.png" alt="Episteme" width="60%" />
</p>

<p align="center"><sub>Episteme (συν τᾰγμᾰ) — 希臘文，意為「有組織的系統」或「判斷力」</sub></p>

<p align="center">一個離線優先、單一二進位檔的知識圖譜，透過語意關係連結設計模式、重構技巧與軟體法則。<br><b>以 AI 代理為核心設計</b> — 將軟體工程專業知識直接整合至 Claude Code、Cursor 及其他相容 MCP 的工具中。</p>

<p align="center">以 Rust 撰寫 · 單一二進位檔 · 完全離線</p>

---

<p align="center">
    <a href="https://github.com/epicsagas/Episteme/actions"><img src="https://img.shields.io/github/actions/workflow/status/epicsagas/Episteme/ci.yml?branch=main&label=CI" alt="CI" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.95+-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/episteme"><img src="https://img.shields.io/badge/crates.io-v0.1.0-orange.svg" alt="crates.io" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

<p align="center">
  <a href="../../README.md">English</a> |
  <a href="../ja/">日本語</a> |
  <a href="../ko/">한국어</a> |
  <a href="../de/">Deutsch</a> |
  <a href="../fr/">Français</a> |
  <a href="../zh-CN/">简体中文</a> |
  繁體中文 |
  <a href="../pt/">Português</a> |
  <a href="../es/">Español</a> |
  <a href="../hi/">हिन्दी</a>
</p>

---

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/features.png">
  <img src="../assets/features.png" align="center" width="100%" alt="Episteme 功能總覽" />
</picture>

---

## 快速入門

> **前置需求：** 透過 [rustup](https://rustup.rs) 安裝 Rust 1.95+ · **沒有 Rust？** 請參閱 [Docker](#選項-3docker無需-rust) 或 [預建二進位檔](#選項-4預建二進位檔無需-rust)。

**1. 安裝 Episteme**

| 方法 | 指令 |
|------|------|
| **cargo-binstall** ⚡ | `cargo binstall episteme` |
| **Shell 指令碼** | `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/epicsagas/Episteme/releases/latest/download/install.sh \| sh` |
| **Homebrew** | `brew install epicsagas/tap/episteme` |
| **Windows** | `irm https://github.com/epicsagas/Episteme/releases/latest/download/install.ps1 \| iex` |
| **cargo** | `cargo install episteme` |
| **Docker** | 見 [選項 3](#選項-3docker無需-rust) |

> **推薦：** `cargo-binstall` 直接下載預建二進位檔，無需編譯。

**2. 載入資料 + 設定你的 AI 工具**

```bash
epis install claude    # 或：cursor、codex、gemini
```

**3. 驗證安裝**

```bash
epis --version
epis stats
```

就是這樣。重新啟動 Claude Code，Episteme 工具即可使用。

### 30 秒內試用

**方式 A — CLI：** 指向專案中的任何檔案。

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

**方式 B — Claude Code：** 在專案中開啟任何檔案，然後用自然語言提問。

```
Find code smells in this project and suggest refactorings.
```

Episteme 會自動觸發 — 不需要特殊語法。它會將你的描述對應到知識圖譜，並返回帶有排序與引用的結果。

---

## 為什麼需要 Episteme？

大型語言模型（LLM）已經知道什麼是策略模式（Strategy Pattern）。它們能背誦 SOLID 原則、列舉 GoF 設計模式、解釋程式碼壞味道。那麼這個專案為什麼存在？

**差距不在於知識 — 而在於結構化、相互連結的推理能力。**

當你問 LLM「如何修正 God Object？」時，它會給你一個合理的答案。但這個答案在不同對話中會改變、缺乏可追溯性，且未將問題與其根本原因或後續影響連結起來。Episteme 將孤立的知識轉化為可遍歷的圖譜，讓每個建議都有根據、可引用，並與更廣泛的設計脈絡相連。

### 這與直接向 LLM 提問有何不同？

| | 精心設計的 LLM 提示 | Episteme + LLM |
|---|---|---|
| 主動偵測 | 僅在使用者問對問題時才會觸發 | 在問題描述時自動觸發 |
| Token 效率 | 冗長的解釋 + 多次追問 | 一次工具呼叫即返回結構化結果 |
| 關係遍歷 | 最多一層跳躍，且經常產生幻覺 | 多層圖譜遍歷，經過驗證 |
| 交叉參照 | 手動操作，容易出錯 | 透過 201 條語意關係自動完成 |
| 一致性 | 每次對話結果不同 | 每次都返回相同的結構化答案 |
| 可引用性 | 「我認為你應該使用 Extract Class」 | 「Extract Class (RF-018)，優先順序 0.89」 |
| 離線 / 空氣隔離環境 | 需要網路才能獲得最佳結果 | 完全本機運作，單一二進位檔 |

### 什麼時候適合使用？

<details>
<summary><b>1. 當你的 AI 代理應主動偵測問題，而非被動等待詢問時</b></summary>

MCP 整合會在問題描述時自動觸發。當使用者說「這個類別做了太多事情」時，代理不需要知道要問關於 God Object 的問題 — Episteme 會將這個描述對應到 `SMELL-03`，呈現排序後的重構建議，並將違規追溯到基本原則。這將模糊的抱怨轉化為結構化的修正計畫。
</details>

<details>
<summary><b>2. 當你想減少 Token 消耗 — 而非浪費在冗長解釋上時</b></summary>

沒有 Episteme 時，LLM 回答「如何修正 God Object？」會解釋壞味道、列出重構方法、描述 SOLID 原則，並逐一說明每個選項 — 每次回應消耗數百個 Token。有了 Episteme，一次 MCP 工具呼叫即返回 `SMELL-03 → RF-018 (0.89) → LAW-001`。以極少的 Token 預算獲得同樣的專業知識。
</details>

<details>
<summary><b>3. 當你需要將程式碼分析連結到修正建議 — 而非僅止於偵測時</b></summary>

像 SonarQube 這樣的工具能偵測壞味道。LLM 能建議模式。Episteme 兩者皆做並將其連結：偵測 Long Method → 追蹤它違反的法則 → 對解決它的重構方法排序 → 顯示哪些模式強化了這些重構。
</details>

<details>
<summary><b>4. 當孤立的模式知識不夠用 — 你需要的是關係時</b></summary>

知道 Extract Method 做什麼只是基本要求。知道它*解決* Long Method (SMELL-01)、而 Long Method *違反* Single Responsibility (LAW-001)、而 Single Responsibility 由 Facade Pattern (DP-012) *強制執行* — 這是一條 LLM 無法自行可靠建構的推理鏈。Episteme 的 201 條語意關係讓 AI 代理能以確定性的方式遍歷這些路徑。
</details>

<details>
<summary><b>5. 當你在做架構決策並需要證據而非意見時</b></summary>

「我應該使用微服務嗎？」— Episteme 將這個問題連結到 Conway's Law (LAW-017)、SRP (LAW-001) 和 Strangler Fig 模式 (DP-026)，然後展示它們之間的關係。決策可追溯到工程法則，而非部落格文章。
</details>

<details>
<summary><b>6. 當你需要一致、可引用的工程建議 — 而非幻覺產生的推薦時</b></summary>

每個發現都引用明確的實體 ID（`DP-005`、`RF-001`、`LAW-021`）。建議附帶優先順序分數和工作量估算。相同的查詢始終返回相同的結構化答案。
</details>

<details>
<summary><b>7. 當你在空氣隔離或受限網路環境中工作時</b></summary>

Episteme 完全離線運作：單一二進位檔、本機 SQLite 資料庫、透過 fastembed (ONNX Runtime) 的本機嵌入。無遙測、無回傳資料、無外部 API 呼叫。你的程式碼和分析結果永遠不會離開你的機器。
</details>

---

## 功能

| | 功能 | 為什麼重要 |
|--|------|-----------|
| 🧠 | **22個GoF設計模式** | 包含實際範例的完整目錄 |
| 🔧 | **66個重構技術** | Fowler目錄，附帶程式碼範例 |
| ⚖️ | **56條軟體定律與原則** | SOLID、康威定律、CAP定理等 |
| 👃 | **17種程式碼異味類型** | Long Method、God Object、Feature Envy等 ¹ |
| 🔗 | **201條語意關係** | 「解決」、「強制」、「違反」、「關聯」 |
| 🤖 | **9個MCP工具 + 4個代理** | 高保真AI代理互動，支援代理間交接 |
| 🌍 | **10種語言支援** | Python（AST）、Java、TypeScript、Go、Rust、C++、C#、PHP、Ruby、Kotlin |
| 📊 | **確定性分析** | 基於AST的Python + 正規表示式多語言，每次結果一致 |
| 🏷️ | **可引用的知識** | 每個發現都連結到明確的實體ID（`RF-001`、`LAW-021`） |
| 🌐 | **REST API（17個端點）** | 認證、速率限制、健康探針、Prometheus指標 |
| 📦 | **單一二進位檔案** | 無執行時期依賴，跨平台（macOS、Linux、Windows） |
| 🔌 | **本地嵌入** | fastembed（ONNX Runtime），零配置語意搜尋 |
| 🐳 | **Docker支援** | 帶健康檢查的多階段建置 |

> ¹ Duplicate Code（SMELL-13）和Shotgun Surgery（SMELL-09）需要多檔案上下文，在單檔案模式下會跳過。

---

## 安裝

### 選項 1：cargo-binstall（推薦）

```bash
cargo binstall episteme    # 下載預建二進位檔 — 無需編譯
epis install claude        # 載入資料 + 設定 MCP + 安裝代理
```

如果沒有 cargo-binstall：`cargo install cargo-binstall`

> 執行 `epis install claude` 後，**重新啟動 Claude Code**，MCP 工具和代理才會出現。

### 選項 2：從原始碼建置

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme && cargo build --release
```

然後執行適合你平台的二進位檔：

| 平台 | 指令 |
|------|------|
| **macOS / Linux** | `./target/release/epis install --local claude` |
| **Windows** | `.\target\release\episteme.exe install --local claude` |

### 選項 3：Docker（無需 Rust）

```bash
docker-compose up -d
```

將以下內容加入你的 MCP 設定檔：

| 工具 | 設定檔路徑 |
|------|-----------|
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

### 選項 4：預建二進位檔（無需 Rust）

從 [GitHub Releases](https://github.com/epicsagas/Episteme/releases) 下載適合你平台的最新二進位檔：

| 平台 | 檔案 |
|------|------|
| **macOS**（Apple Silicon） | `episteme-aarch64-apple-darwin.tar.gz` |
| **macOS**（Intel） | `episteme-x86_64-apple-darwin.tar.gz` |
| **Linux**（x86_64） | `episteme-x86_64-unknown-linux-gnu.tar.gz` |
| **Linux**（ARM64） | `episteme-aarch64-unknown-linux-gnu.tar.gz` |
| **Windows**（x86_64） | `episteme-x86_64-pc-windows-msvc.zip` |

```bash
# macOS / Linux
tar xzf episteme-*.tar.gz
sudo mv episteme /usr/local/bin/

# Windows — 解壓縮 zip 並將 episteme.exe 加入你的 PATH
```

然後安裝：
```bash
epis install claude    # 或：cursor、codex、gemini
```

### 驗證安裝

```bash
epis --version
epis stats
epis explore "strategy pattern"    # 探索知識圖譜
```

---

## MCP 工具與代理

> **什麼是 MCP？** [Model Context Protocol](https://modelcontextprotocol.io) 是一個開放標準，讓 AI 工具能呼叫外部服務。Episteme 將其知識圖譜暴露為 MCP 工具，Claude Code、Cursor 及其他相容編輯器可以自動呼叫。

### 9 個 MCP 工具

#### 規範知識（6 個工具）

| 工具 | 用途 | 使用範例 |
|------|---------|-------------|
| **`search_knowledge`** | 跨所有實體的語義搜尋 | "尋找重試邏輯的模式" |
| **`get_entity`** | 按ID取得特定實體詳情 | "解釋策略模式 (DP-023)" |
| **`get_neighbors`** | 探索相關實體 | "哪些重構能解決長方法？" |
| **`find_path`** | 尋找兩個實體之間的連接 | "SRP與提取類有何關係？" |
| **`analyze_code`** | 透過正則/AST分析檢測程式碼異味 | "審查此付款驗證程式碼" |
| **`suggest_refactorings`** | 排序後的重構建議 | "這個類別應該重構什麼？" |

#### 隱性知識（3 個工具）

| 工具 | 用途 | 使用範例 |
|------|---------|-------------|
| **`add_insight`** | 記錄團隊決策、經驗教訓 | "選擇事件驅動而非輪詢的原因" |
| **`search_insights`** | 搜尋過去的團隊知識 | "我們對認證中介軟體做了什麼決定？" |
| **`confirm_links`** | 驗證自動偵測到的規範實體連結 | 確認 TK-001 與 SMELL-03 相關聯 |

Episteme 將隱性知識儲存在獨立的資料庫（`~/.episteme/user_knowledge.db`）中，並在執行時透過組合層與規範圖合併。團隊洞察會自動連結到模式、法則和異味，將經驗轉化為可導航的知識。

完整設計請參閱[隱性知識架構](./tacit-knowledge.md)。

### 4 個專用代理（互聯網路）

代理之間協同運作 — 每次分析結束時都會提供**後續步驟**選項，將工作交接給其他代理。

| 代理 | 使用時機 | 核心能力 | 交接給 |
|------|----------|----------|--------|
| **`code-reviewer`** | 程式碼壞味道、SOLID 違規 | 因果分析（根本原因 → 下游症狀） | advisor、architecture-analyst、refactoring-expert |
| **`episteme-advisor`** | 工程決策、取捨評估 | 多實體取捨鏈與行動計畫 | code-reviewer、architecture-analyst、researcher |
| **`episteme-researcher`** | 知識圖譜探索 | 模式、法則、壞味道之間的連結地圖 | advisor、code-reviewer |
| **`architecture-analyst`** | 針對法則的架構評估 | 含風險加權評估的合規性評分 | advisor、code-reviewer、researcher |

**工作流程範例**：`code-reviewer` 偵測到 God Object → 追蹤因果關係至 3 個下游壞味道 → 提供「套用 RF-018」（→ refactoring-expert）或「深入分析根本原因」（→ episteme-advisor）或「架構檢查」（→ architecture-analyst）。

[完整 MCP 整合指南](./mcp-integration-guide.md)

---

## CLI 使用

```bash
# 分析程式碼的壞味道
epis analyze my_code.py --language python --json
episteme infer my_code.py

# 探索知識圖譜
epis explore "strategy pattern"
epis graph path DP-005 RF-001   # 例如：Factory Method → Extract Method

# 建置 RAG 索引
epis build

# 啟動伺服器
epis api              # REST API，連接埠 :8000
episteme mcp --http       # MCP 伺服器，連接埠 :43175
episteme web --port 8080  # Web UI（互動式圖譜瀏覽器）

# 發行打包
episteme dist --out-dir release/
```

---

## 文件

| 文件 | 說明 |
|------|------|
| [快速入門](./QUICKSTART.md) | 逐步設定、首次執行、疑難排解 |
| [MCP 整合指南](./mcp-integration-guide.md) | 工具參考、代理範例、對話流程 |
| [隱性知識架構](./tacit-knowledge.md) | 雙資料庫設計、洞察生命週期、綱要 |
| [Alcove 生態系比較](./alcove-ecosystem.md) | 儲存模型、搜尋能力、使用案例矩陣 |
| [Alcove 整合指南](./alcove-integration.md) | 雙上下文工作流程、設定、最佳實務 |
| [API 參考](./api.md) | REST 端點、身份驗證、範例 |
| [發行](./distribution.md) | 發行打包與部署 |
| [開發與貢獻](./DEVELOPMENT.md) | 架構說明、如何貢獻 |
| [更新日誌](./CHANGELOG.md) | 版本歷史與版本說明 |

---

## 設定

### 環境變數

```bash
# 資料位置
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# API 伺服器
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEY=your-secret-key

# MCP 伺服器
EPISTEME_MCP_HOST=127.0.0.1
EPISTEME_MCP_PORT=43175
```

---

## 疑難排解

**安裝後找不到 `episteme` 指令**

| 平台 | 解決方法 |
|------|----------|
| **macOS / Linux** | `export PATH="$HOME/.cargo/bin:$PATH"` — 加入 `~/.bashrc` 或 `~/.zshrc` 以永久生效 |
| **Windows** | 將 `%USERPROFILE%\.cargo\bin` 加入系統 PATH，或開啟新的終端機 |

**MCP 工具未出現在 Claude Code / Cursor 中**

執行 `epis install` 後重新啟動編輯器。若仍無法使用，檢查設定檔是否已寫入：
```bash
cat ~/.claude.json   # Claude Code
```

**連接埠已被佔用**
```bash
episteme mcp --http --port 43176   # 使用其他連接埠
```

**首次啟動緩慢**

Episteme 在首次執行時會建置本機嵌入索引。這需要 30–60 秒，且僅此一次。後續啟動將即時完成。

**`cargo install` 時出現編譯錯誤**

確保已安裝 Rust 1.95+：
```bash
rustup update stable
rustup show   # 確認使用中的工具鏈
```

> 更多協助：[QUICKSTART.md 疑難排解章節](../../QUICKSTART.md#troubleshooting) · [提交問題](https://github.com/epicsagas/Episteme/issues)

---

## 發展路線圖

- [ ] **自訂實體** — 新增團隊特定的模式/壞味道
- [ ] **互動式教學** — 應用程式內的 MCP 工具導覽
- [ ] **多語言中繼資料** — 實體標題與摘要的韓文、日文、中文支援（README 翻譯已完成）
- [ ] **MCP 工具描述** — 取代 IDE 專用外掛的增強工具描述
- [ ] **團隊指標** — 跨組織的聚合模式使用分析

---

## 貢獻

歡迎貢獻！請參閱 [DEVELOPMENT.md](./DEVELOPMENT.md) 以了解架構概覽和貢獻指南。

```bash
# 執行測試
cargo test

# 程式碼檢查
cargo clippy -- -D warnings

# 格式化
cargo fmt
```

有任何問題？[開啟討論](https://github.com/epicsagas/Episteme/discussions) 或 [提交問題](https://github.com/epicsagas/Episteme/issues)。

---

## 授權

Apache 2.0 — 詳見 [LICENSE](../../LICENSE)。

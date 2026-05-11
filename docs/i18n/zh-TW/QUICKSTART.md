# Episteme — 快速入門指南

在 2 分鐘內開始使用 Episteme。

---

## 先決條件

- **Rust 1.95+**（需要 edition 2024）— [透過 rustup 安裝](https://rustup.rs)
- 網際網路連線（用於初始資料下載）

---

## 選項一：AI 工具整合（推薦）

**適用於：** Claude Code、Cursor、Codex、Gemini 使用者

```bash
# 1. 安裝 Episteme
cargo install --git https://github.com/epicsagas/Episteme

# 2. 安裝至您的 AI 工具（下載資料、設定 MCP、複製代理）
epis install claude      # Claude Code
epis install cursor      # Cursor
epis install codex       # OpenAI Codex
epis install gemini      # Gemini CLI
epis install all         # 一次安裝所有工具
```

> 若 `epis install claude` 無法下載資料，請改用下方的原始碼安裝方式。

**完成。** 重新啟動您的 AI 工具，Episteme 即已啟用。

---

## 選項二：Docker（無需 Rust）

```bash
docker-compose up -d

# 存取
# API:       http://localhost:8000
# 健康檢查:    http://localhost:8000/health
```

若要透過 Docker 整合 MCP，請將以下內容加入您的 MCP 設定：
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

---

## 選項三：從原始碼安裝

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme

# 建置
cargo build --release

# 種子資料並建置向量資料庫（建置會自動執行）
./target/release/epis install --local
```

---

## 圖譜視覺化

Episteme 包含互動式 D3-force 圖譜檢視器：

```bash
episteme web               # 預設: http://localhost:8080
episteme web --port 9001   # 自訂連接埠
episteme web --host 0.0.0.0 --port 8080  # 暴露至網路
```

---

## 常用指令

```bash
# 分析程式碼中的壞味道
epis analyze my_code.py --language python
epis analyze my_code.py --json

# 取得重構建議
episteme infer my_code.py --top-k 5

# 探索知識圖譜
epis explore "strategy pattern"
epis graph path DP-005 RF-001

# 啟動伺服器
epis api              # REST API 於 :8000
episteme mcp --http       # MCP 伺服器於 :43175
episteme web --port 8080  # Web UI

# 背景 MCP 常駐服務（HTTP 代理）
epis service start
epis service status
epis service stop

# 建立發行封存檔
episteme dist --out-dir release
```

---

## 疑難排解

### 「找不到資料庫」
```bash
epis install claude   # 重新下載資料封存檔
# 或
epis install --local
```

### 「連接埠已被佔用」
```bash
episteme web --port 9001
epis api --port 9000
```

---

## 下一步

- **[README](../../README.md)** — 完整功能概覽與架構說明
- **[MCP 整合指南](./mcp-integration-guide.md)** — 工具參考與代理範例
- **[API 參考文件](./api.md)** — REST 端點說明
- **[貢獻指南](../../CONTRIBUTING.md)** — 開發工作流程

# Episteme REST API 文件

**版本：** 0.1.0
**基礎 URL：** `http://localhost:8000`

---

## 快速開始

```bash
# 啟動伺服器
epis api

# 或使用自訂主機/連接埠
epis api --host 0.0.0.0 --port 8000

# Docker
docker-compose up -d
```

---

## 身份驗證

所有端點除了 `/`、`/health`、`/live`、`/ready` 以外，皆需要 API 金鑰身份驗證。

### API 金鑰身份驗證

**標頭：** `X-API-Key: <your-api-key>`

**模式：**

1. **正式環境模式** — 設定 `EPISTEME_API_KEYS` 環境變數
   - 以逗號分隔的有效 API 金鑰清單
   - 所有受保護的端點皆需要有效的金鑰
   - 若金鑰缺少或無效，回傳 401 Unauthorized

2. **開發模式** — 將 `EPISTEME_API_KEYS` 留空或取消設定
   - 不需要身份驗證

### 產生 API 金鑰

```bash
openssl rand -base64 32
```

### 請求範例

```bash
# 使用身份驗證（正式環境）
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"code": "def long_method(): pass", "min_confidence": 0.5}'

# 不使用身份驗證（開發模式）
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"code": "def long_method(): pass"}'
```

---

## 速率限制

所有端點皆依 IP 位址進行速率限制，並使用基於 TTL 的桶淘汰機制。

| 端點 | 速率限制 | 原因 |
|------|----------|------|
| `/analyze` | 20 次/分鐘 | CPU 密集運算 |
| `/refactor` | 20 次/分鐘 | CPU 密集運算 |
| `/search` | 50 次/分鐘 | 嵌入向量計算 |
| `/stats`、`/graph/*` | 100 次/分鐘 | 標準 |
| `/`、`/health` | 無限制 | 公開 |

超過限制時，回傳 429 狀態碼及 `Retry-After` 標頭。

---

## 端點

### 健康檢查與資訊

#### `GET /`

服務資訊。

**回應：**
```json
{
  "name": "episteme",
  "version": "0.1.0",
  "description": "Software engineering knowledge graph",
  "endpoints": ["analyze", "search", "graph", "refactor", "stats"]
}
```

#### `GET /health`

包含元件狀態的健康檢查。

**回應：**
```json
{
  "status": "healthy",
  "components": {
    "knowledge_graph": "ok",
    "rag_database": "ok",
    "embedding_provider": "local"
  }
}
```

#### `GET /live`

存活探針：`{"status": "alive"}`

#### `GET /ready`

就緒探針：`{"status": "ready"}`（若未就緒則回傳 503）

#### `GET /stats`

知識圖譜統計資料。

**回應：**
```json
{
  "total_entities": 161,
  "total_edges": 201,
  "by_type": {
    "refactoring": 66,
    "law": 56,
    "pattern": 22,
    "smell": 17
  }
}
```

---

### 程式碼分析

#### 支援的程式碼壞味道（16 個偵測器）

| ID | 名稱 | 語言 |
|----|------|------|
| SMELL-01 | Long Method | 全部 |
| SMELL-02 | Long Parameter List | 全部 |
| SMELL-03 | Primitive Obsession | Python |
| SMELL-04 | Large Class | 全部 |
| SMELL-05 | Data Clumps | 全部（存根） |
| SMELL-06 | Switch Statements | 全部 |
| SMELL-07 | Data Class | 全部 |
| SMELL-09 | Shotgun Surgery | 全部（存根） |
| SMELL-10 | Divergent Change | 全部 |
| SMELL-11 | Lazy Class | 全部 |
| SMELL-12 | Speculative Generality | 全部 |
| SMELL-13 | Duplicate Code | 全部（部分） |
| SMELL-14 | Middle Man | 全部 |
| SMELL-18 | Feature Envy | 全部 |
| SMELL-20 | Message Chains | 全部 |
| SMELL-21 | God Object | 全部 |

#### `POST /analyze`

偵測程式碼壞味道。

**請求：**
```json
{
  "code": "def long_method():\n    ...",
  "language": "python",
  "min_confidence": 0.5
}
```

**回應：**
```json
{
  "count": 2,
  "smells": [
    {
      "smell_id": "SMELL-01",
      "smell_name": "Long Method",
      "confidence": 0.90,
      "location": "temp.py:1",
      "function_name": "long_method",
      "metrics": {
        "loc": 94,
        "cyclomatic_complexity": 27,
        "nesting_depth": 5,
        "parameter_count": 9
      },
      "reasons": ["LOC=94 exceeds 30", "CC=27 exceeds 10"]
    }
  ]
}
```

#### `POST /refactor`

取得針對已偵測壞味道的排序重構建議。

**請求：**
```json
{
  "code": "def long_method():\n    ...",
  "top_k": 3,
  "min_confidence": 0.5
}
```

**回應：**
```json
{
  "count": 1,
  "analyses": [
    {
      "smell": { "smell_id": "SMELL-01", "smell_name": "Long Method" },
      "suggestions": [
        {
          "refactoring_id": "RF-001",
          "title": "Extract Method",
          "priority_score": 0.79,
          "effort": "medium",
          "principles_enforced": ["LAW-040", "LAW-042-S"]
        }
      ]
    }
  ]
}
```

---

### 搜尋

#### `GET /search`

透過查詢參數搜尋：`/search?q=strategy+pattern&top_k=5`

#### `POST /search`

對知識庫進行語意搜尋。

**請求：**
```json
{
  "query": "How to fix Long Method?",
  "top_k": 5,
  "entity_type": "refactoring"
}
```

**回應：**
```json
{
  "count": 3,
  "results": [
    {
      "entity_id": "RF-001",
      "title": "Extract Method",
      "category": "refactoring",
      "similarity": 0.85,
      "content": "Extract Method is a refactoring technique..."
    }
  ]
}
```

---

### 知識圖譜

#### `GET /graph/{id}`

依 ID 取得實體詳細資訊。

**範例：** `GET /graph/DP-005`

#### `GET /graph/{id}/neighbors`

取得實體的鄰居：`/graph/SMELL-01/neighbors?relation_type=solved_by`

#### `POST /graph/neighbors`

取得鄰居（POST）。

**請求：**
```json
{
  "entity_id": "SMELL-01",
  "relation_type": "solved_by"
}
```

#### `GET /graph/path`

最短路徑：`/graph/path?from_id=SMELL-01&to_id=LAW-042-S&max_depth=5`

#### `POST /graph/subgraph`

擷取子圖。

**請求：**
```json
{
  "entity_id": "DP-005",
  "depth": 2
}
```

#### `GET /graph/contradictions`

尋找具有衝突關係的實體。

#### `POST /graph/infer-transitive`

推斷傳遞強制關係。

---

### 監控

#### `GET /metrics`

Prometheus 格式的指標，包含：
- `http_requests_total` — 依方法、端點、狀態
- `episteme_smells_detected_total` — 依 smell_id
- `episteme_searches_total` — 依 entity_type
- `episteme_analysis_duration_seconds` — 直方圖

---

## 效能

| 端點 | 平均延遲 | 備註 |
|------|----------|------|
| `/analyze` | ~5ms | 正規表示式 + AST 解析（OnceLock 快取） |
| `/refactor` | ~10ms | 包含圖譜遍歷 |
| `/search` | ~20ms | FTS5 + 餘弦相似度 |
| `/graph/neighbors` | ~1ms | 記憶體內圖譜 |
| `/graph/path` | ~5ms | BFS，深度上限 5 |

---

## 錯誤處理

| 狀態碼 | 意義 |
|--------|------|
| 200 | 成功 |
| 400 | 錯誤的請求 |
| 401 | API 金鑰缺少或無效 |
| 404 | 找不到實體 |
| 429 | 超過速率限制 |
| 500 | 內部錯誤 |

---

## 環境變數

```bash
# 伺服器
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEYS=key1,key2

# 資料
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# 日誌
RUST_LOG=info
```

---

## 授權

APACHE-2.0 授權

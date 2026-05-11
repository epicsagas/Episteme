# Episteme REST API 文档

**版本：** 0.1.0
**基础URL：** `http://localhost:8000`

---

## 快速入门

```bash
# 启动服务器
epis api

# 或使用自定义主机/端口
epis api --host 0.0.0.0 --port 8000

# Docker
docker-compose up -d
```

---

## 认证

除`/`、`/health`、`/live`、`/ready`之外的所有端点都需要API密钥认证。

### API密钥认证

**请求头：** `X-API-Key: <your-api-key>`

**模式：**

1. **生产模式** - 设置`EPISTEME_API_KEYS`环境变量
   - 逗号分隔的有效API密钥列表
   - 所有受保护端点都需要有效密钥
   - 缺少或无效时返回401 Unauthorized

2. **开发模式** - 将`EPISTEME_API_KEYS`留空或不设置
   - 无需认证

### 生成API密钥

```bash
openssl rand -base64 32
```

### 请求示例

```bash
# 带认证（生产环境）
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"code": "def long_method(): pass", "min_confidence": 0.5}'

# 无认证（开发模式）
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"code": "def long_method(): pass"}'
```

---

## 速率限制

所有端点按IP地址进行基于TTL桶驱逐的速率限制。

| 端点 | 速率限制 | 原因 |
|----------|------------|--------|
| `/analyze` | 20次/分钟 | CPU密集型 |
| `/refactor` | 20次/分钟 | CPU密集型 |
| `/search` | 50次/分钟 | 嵌入计算 |
| `/stats`、`/graph/*` | 100次/分钟 | 标准 |
| `/`、`/health` | 无限制 | 公开 |

超限时返回429，并包含`Retry-After`请求头。

---

## 端点

### 健康检查与信息

#### `GET /`

服务信息。

**响应：**
```json
{
  "name": "episteme",
  "version": "0.1.0",
  "description": "Software engineering knowledge graph",
  "endpoints": ["analyze", "search", "graph", "refactor", "stats"]
}
```

#### `GET /health`

带组件状态的健康检查。

**响应：**
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

存活探测: `{"status": "alive"}`

#### `GET /ready`

就绪探测: `{"status": "ready"}`（未就绪时返回503）

#### `GET /stats`

图谱统计。

**响应：**
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

### 代码分析

#### 支持的代码异味（16个检测器）

| ID | 名称 | 支持语言 |
|---|---|---|
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

检测代码异味。

**请求：**
```json
{
  "code": "def long_method():\n    ...",
  "language": "python",
  "min_confidence": 0.5
}
```

**响应：**
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

获取已检测异味的排名重构建议。

**请求：**
```json
{
  "code": "def long_method():\n    ...",
  "top_k": 3,
  "min_confidence": 0.5
}
```

**响应：**
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

### 搜索

#### `GET /search`

通过查询参数搜索: `/search?q=strategy+pattern&top_k=5`

#### `POST /search`

知识库的语义搜索。

**请求：**
```json
{
  "query": "How to fix Long Method?",
  "top_k": 5,
  "entity_type": "refactoring"
}
```

**响应：**
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

### 知识图谱

#### `GET /graph/{id}`

按ID获取实体详情。

**示例：** `GET /graph/DP-005`

#### `GET /graph/{id}/neighbors`

获取实体的相邻实体: `/graph/SMELL-01/neighbors?relation_type=solved_by`

#### `POST /graph/neighbors`

获取相邻实体（POST）。

**请求：**
```json
{
  "entity_id": "SMELL-01",
  "relation_type": "solved_by"
}
```

#### `GET /graph/path`

最短路径: `/graph/path?from_id=SMELL-01&to_id=LAW-042-S&max_depth=5`

#### `POST /graph/subgraph`

提取子图。

**请求：**
```json
{
  "entity_id": "DP-005",
  "depth": 2
}
```

#### `GET /graph/contradictions`

查找具有冲突关系的实体。

#### `POST /graph/infer-transitive`

推断传递性强制关系。

---

### 监控

#### `GET /metrics`

Prometheus格式的指标，包括:
- `http_requests_total` — 按方法、端点、状态分组
- `episteme_smells_detected_total` — 按smell_id分组
- `episteme_searches_total` — 按entity_type分组
- `episteme_analysis_duration_seconds` — 直方图

---

## 性能

| 端点 | 平均延迟 | 备注 |
|----------|-------------|-------|
| `/analyze` | ~5ms | 正则 + AST解析（OnceLock缓存） |
| `/refactor` | ~10ms | 包含图谱遍历 |
| `/search` | ~20ms | FTS5 + 余弦相似度 |
| `/graph/neighbors` | ~1ms | 内存图谱 |
| `/graph/path` | ~5ms | 深度最多5的BFS |

---

## 错误处理

| 状态码 | 含义 |
|--------|---------|
| 200 | 成功 |
| 400 | 错误请求 |
| 401 | API密钥缺失或无效 |
| 404 | 实体未找到 |
| 429 | 超出速率限制 |
| 500 | 内部错误 |

---

## 环境变量

```bash
# 服务器
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEYS=key1,key2

# 数据
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# 日志
RUST_LOG=info
```

---

## 许可证

APACHE-2.0许可证

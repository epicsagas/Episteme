# Episteme REST API 문서

**버전:** 0.1.0
**기본 URL:** `http://localhost:8000`

---

## 빠른 시작

```bash
# 서버 시작
epis api

# 또는 사용자 지정 호스트/포트
epis api --host 0.0.0.0 --port 8000

# Docker
docker-compose up -d
```

---

## 인증

`/`, `/health`, `/live`, `/ready`를 제외한 모든 엔드포인트는 API 키 인증이 필요합니다.

### API 키 인증

**헤더:** `X-API-Key: <your-api-key>`

**모드:**

1. **프로덕션 모드** - `EPISTEME_API_KEYS` 환경 변수 설정
   - 쉼표로 구분된 유효한 API 키 목록
   - 모든 보호된 엔드포인트에 유효한 키 필요
   - 누락/유효하지 않은 경우 401 Unauthorized 반환

2. **개발 모드** - `EPISTEME_API_KEYS`를 비워두거나 설정하지 않음
   - 인증 불필요

### API 키 생성

```bash
openssl rand -base64 32
```

### 요청 예시

```bash
# 인증 포함 (프로덕션)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"code": "def long_method(): pass", "min_confidence": 0.5}'

# 인증 없이 (개발 모드)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"code": "def long_method(): pass"}'
```

---

## 속도 제한

모든 엔드포인트는 IP 주소별로 TTL 기반 버킷 제거 방식의 속도 제한이 적용됩니다.

| 엔드포인트 | 속도 제한 | 사유 |
|----------|------------|--------|
| `/analyze` | 분당 20회 | CPU 집약적 |
| `/refactor` | 분당 20회 | CPU 집약적 |
| `/search` | 분당 50회 | 임베딩 연산 |
| `/stats`, `/graph/*` | 분당 100회 | 표준 |
| `/`, `/health` | 무제한 | 공개 |

초과 시 `Retry-After` 헤더와 함께 429를 반환합니다.

---

## 엔드포인트

### 상태 및 정보

#### `GET /`

서비스 정보.

**응답:**
```json
{
  "name": "episteme",
  "version": "0.1.0",
  "description": "Software engineering knowledge graph",
  "endpoints": ["analyze", "search", "graph", "refactor", "stats"]
}
```

#### `GET /health`

구성 요소 상태가 포함된 상태 확인.

**응답:**
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

활성 프로브: `{"status": "alive"}`

#### `GET /ready`

준비 프로브: `{"status": "ready"}` (준비되지 않은 경우 503)

#### `GET /stats`

그래프 통계.

**응답:**
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

### 코드 분석

#### 지원되는 코드 스멜 (16개 감지기)

| ID | 이름 | 언어 |
|---|---|---|
| SMELL-01 | Long Method | 전체 |
| SMELL-02 | Long Parameter List | 전체 |
| SMELL-03 | Primitive Obsession | Python |
| SMELL-04 | Large Class | 전체 |
| SMELL-05 | Data Clumps | 전체 (스텁) |
| SMELL-06 | Switch Statements | 전체 |
| SMELL-07 | Data Class | 전체 |
| SMELL-09 | Shotgun Surgery | 전체 (스텁) |
| SMELL-10 | Divergent Change | 전체 |
| SMELL-11 | Lazy Class | 전체 |
| SMELL-12 | Speculative Generality | 전체 |
| SMELL-13 | Duplicate Code | 전체 (부분) |
| SMELL-14 | Middle Man | 전체 |
| SMELL-18 | Feature Envy | 전체 |
| SMELL-20 | Message Chains | 전체 |
| SMELL-21 | God Object | 전체 |

#### `POST /analyze`

코드 스멜 감지.

**요청:**
```json
{
  "code": "def long_method():\n    ...",
  "language": "python",
  "min_confidence": 0.5
}
```

**응답:**
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

감지된 스멜에 대한 순위가 매겨진 리팩토링 제안.

**요청:**
```json
{
  "code": "def long_method():\n    ...",
  "top_k": 3,
  "min_confidence": 0.5
}
```

**응답:**
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

### 검색

#### `GET /search`

쿼리 매개변수를 통한 검색: `/search?q=strategy+pattern&top_k=5`

#### `POST /search`

지식 베이스에 대한 시맨틱 검색.

**요청:**
```json
{
  "query": "How to fix Long Method?",
  "top_k": 5,
  "entity_type": "refactoring"
}
```

**응답:**
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

### 지식 그래프

#### `GET /graph/{id}`

ID로 엔티티 상세 정보 조회.

**예시:** `GET /graph/DP-005`

#### `GET /graph/{id}/neighbors`

엔티티의 이웃 조회: `/graph/SMELL-01/neighbors?relation_type=solved_by`

#### `POST /graph/neighbors`

이웃 조회 (POST).

**요청:**
```json
{
  "entity_id": "SMELL-01",
  "relation_type": "solved_by"
}
```

#### `GET /graph/path`

최단 경로: `/graph/path?from_id=SMELL-01&to_id=LAW-042-S&max_depth=5`

#### `POST /graph/subgraph`

서브그래프 추출.

**요청:**
```json
{
  "entity_id": "DP-005",
  "depth": 2
}
```

#### `GET /graph/contradictions`

충돌하는 관계를 가진 엔티티 찾기.

#### `POST /graph/infer-transitive`

전이적 강제 관계 추론.

---

### 모니터링

#### `GET /metrics`

Prometheus 형식 메트릭 포함:
- `http_requests_total` — 메서드, 엔드포인트, 상태별
- `episteme_smells_detected_total` — smell_id별
- `episteme_searches_total` — entity_type별
- `episteme_analysis_duration_seconds` — 히스토그램

---

## 성능

| 엔드포인트 | 평균 지연 시간 | 비고 |
|----------|-------------|-------|
| `/analyze` | ~5ms | 정규식 + AST 파싱 (OnceLock 캐시) |
| `/refactor` | ~10ms | 그래프 순회 포함 |
| `/search` | ~20ms | FTS5 + 코사인 유사도 |
| `/graph/neighbors` | ~1ms | 인메모리 그래프 |
| `/graph/path` | ~5ms | 최대 깊이 5까지의 BFS |

---

## 오류 처리

| 상태 코드 | 의미 |
|--------|---------|
| 200 | 성공 |
| 400 | 잘못된 요청 |
| 401 | API 키 누락/유효하지 않음 |
| 404 | 엔티티를 찾을 수 없음 |
| 429 | 속도 제한 초과 |
| 500 | 내부 오류 |

---

## 환경 변수

```bash
# 서버
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEYS=key1,key2

# 데이터
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# 로깅
RUST_LOG=info
```

---

## 라이선스

APACHE-2.0 라이선스

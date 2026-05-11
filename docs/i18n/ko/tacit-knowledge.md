# 암묵적 지식 아키텍처

Episteme는 **정식**(불변, 큐레이션됨)과 **암묵적**(가변, 사용자 기여)이라는 두 가지 뚜렷한 지식 계층을 관리합니다. 이 문서는 이중 데이터베이스 아키텍처, 데이터 흐름 및 인사이트 수명 주기를 설명합니다.

## 개요

| | 정식 지식 | 암묵적 지식 (인사이트) |
|---|---|---|
| **저장소** | `~/.episteme/db/episteme.db` | `~/.episteme/user_knowledge.db` |
| **변경 가능성** | 읽기 전용 (`epis build`로 재구축) | 읽기-쓰기 (MCP를 통한 실시간) |
| **ID 접두사** | `DP-NNN`, `RF-NNN`, `LAW-NNN`, `SMELL-NNN` | `TK-NNN` |
| **소스** | `raw/`의 큐레이션된 마크다운 파일 | MCP `add_insight` 도구 / CLI `epis insight` |
| **엔티티** | 22개 패턴, 66개 리팩토링, 56개 법칙, 23개 스멜 | 무제한 사용자 인사이트 |

이 두 데이터베이스는 물리적으로 분리되어 있지만 런타임에 단일 순회 가능한 그래프로 병합됩니다.

## 이중 데이터베이스 설계

```
┌─────────────────────────────────┐     ┌──────────────────────────────┐
│  정식 DB (episteme.db)          │     │  사용자 지식 DB               │
│                                 │     │  (user_knowledge.db)         │
│  ┌───────────┐  ┌────────────┐  │     │  ┌────────────────────────┐  │
│  │  chunks   │  │ embeddings │  │     │  │  user_entities         │  │
│  │  (914)    │  │  (914)     │  │     │  │  (TK-xxx 항목)         │  │
│  └───────────┘  └────────────┘  │     │  ├────────────────────────┤  │
│                                 │     │  │  user_relations        │  │
│  Built by: epis build           │     │  ├────────────────────────┤  │
│  Populated from: raw/*.md       │     │  │  user_embeddings       │  │
│                                 │     │  ├────────────────────────┤  │
│  런타임에 불변                  │     │  │  user_entities_fts     │  │
│                                 │     │  │  (FTS5 검색 인덱스)     │  │
└──────────────┬──────────────────┘     │  ├────────────────────────┤  │
               │                        │  │  insight_seq           │  │
               │                        │  │  (원자적 ID 카운터)     │  │
               │                        │  └────────────────────────┘  │
               │                        │                              │
               │                        │  Written by: MCP add_insight │
               │                        │  Read by: search_insights    │
               │                        └──────────────┬───────────────┘
               │                                       │
               └───────────────┬───────────────────────┘
                               │
                    ┌──────────▼──────────┐
                    │   CompositeGraph    │
                    │   (인메모리 병합)    │
                    │                     │
                    │  - 통합 엔티티       │
                    │    조회              │
                    │  - 교차 계층 BFS     │
                    │  - 교차 계층         │
                    │    이웃 쿼리         │
                    │                     │
                    │  모든 MCP            │
                    │  도구 요청 처리      │
                    └─────────────────────┘
```

### 왜 데이터베이스를 분리하나요?

1. **보호** — 사용자 입력이 큐레이션된 정식 지식을 손상시킬 수 없습니다.
2. **독립적인 수명 주기** — 정식 지식은 빌드 파이프라인을 통해 업데이트; 암묵적 지식은 실시간으로 업데이트됩니다.
3. **이식성** — 정식 계층을 건드리지 않고 `user_knowledge.db`를 여러 시스템이나 팀 간에 공유할 수 있습니다.

## CompositeGraph

`CompositeGraph` 구조체(`src/domain/composite_graph.rs`)는 시작 시 두 계층을 단일 `GraphRepository` 인터페이스로 병합합니다:

- `relations.json`에서 정식 `KnowledgeGraph` 로드
- `UserGraphStore`를 통해 `user_knowledge.db` 열기
- 두 계층에 걸쳐 통합된 `get_entity()`, `get_neighbors()`, `find_path()` 제공
- 사용자 작업은 정식 그래프를 수정하지 않음

### 정상적 폴백

`user_knowledge.db`를 열 수 없는 경우(파일 누락, 권한 오류), 시스템은 정식 전용 모드로 폴백합니다. 6개의 정식 MCP 도구는 계속 작동하며, 3개의 암묵적 지식 도구는 오류를 반환합니다.

## 사용자 지식 스키마

```sql
-- 핵심 엔티티 테이블
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,                    -- 예: "TK-001"
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author TEXT NOT NULL DEFAULT 'user',
    confidence REAL NOT NULL DEFAULT 0.5,   -- 0.0에서 1.0
    evidence_count INTEGER NOT NULL DEFAULT 0,
    last_validated TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',        -- JSON 배열
    relations TEXT NOT NULL DEFAULT '{}',   -- JSON: type -> [target_ids]
    created_at TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT '',
    link_provenance TEXT NOT NULL DEFAULT '{}'  -- JSON: entity_id -> metadata
);

-- 명시적 관계 간선
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    to_id TEXT NOT NULL,
    UNIQUE(from_id, relation_type, to_id)
);

-- 임베딩 벡터 (f32, 리틀 엔디안)
CREATE TABLE user_embeddings (
    entity_id TEXT PRIMARY KEY,
    embedding BLOB NOT NULL
);

-- 전문 검색 인덱스
CREATE VIRTUAL TABLE user_entities_fts USING fts5(
    title, content, tags,
    content=user_entities, content_rowid=rowid
);

-- 원자적 ID 시퀀스
CREATE TABLE insight_seq (key TEXT PRIMARY KEY, val INTEGER NOT NULL);
```

## MCP 도구

### add_insight

자유 텍스트에서 `TK-NNN` 엔티티를 생성합니다. 시스템이 자동으로:

1. **정식 엔티티 링크 감지** — 2단계 키워드 매칭(불용어 필터링 + 복합 점수)으로 관련 패턴, 법칙, 스멜을 찾습니다.
2. **중복 확인** — 기존 인사이트와 비교합니다.
3. **`derives_from` 관계 생성** — 높은 신뢰도 링크(점수 >= 0.5)의 경우 정식 엔티티에 자동 연결합니다.
4. **상관 관계 계산** — Jaccard 유사도를 사용하여 관련 인사이트를 찾습니다.

매개변수:
- `text` (필수) — 자유 텍스트 인사이트 내용
- `project` (선택) — 프로젝트 이름 태그
- `tags` (선택) — 카테고리 태그
- `linked_entities` (선택) — 명시적으로 연결할 엔티티 ID (예: `["DP-005", "SMELL-01"]`)

### search_insights

사용자가 기여한 인사이트에 대한 FTS5 키워드 검색. 내용과 관계가 포함된 일치하는 `TK-*` 엔티티를 반환합니다.

매개변수:
- `query` (필수) — 자연어 검색 쿼리
- `limit` (선택) — 최대 결과 수 (기본값 10, 최대 20)

### confirm_links

인사이트와 정식 엔티티 간의 자동 감지된 링크를 확인하거나 거부합니다. 각 확인은:

- 인사이트의 신뢰도 점수를 높임 (확인된 링크당 +0.05, 최대 1.0)
- 링크 출처 기록 (소스, 점수, 타임스탬프)
- 인사이트 간의 병합/대체 관계 지원

매개변수:
- `insight_id` (필수) — `TK-NNN` ID
- `accepted` (필수) — 유효한 링크로 확인할 엔티티 ID
- `rejected` (선택) — 거부할 엔티티 ID
- `merged_with` (선택) — 병합/대체할 대상 인사이트 ID

## 인사이트 수명 주기

```
1. add_insight("마이크로서비스 분리 시 도메인 경계를 먼저 식별하기로 결정")
       │
       ▼
2. 링크 자동 감지: CONWAY-001 (Conway's Law), DP-026 (Strangler Fig)
       │
       ▼
3. derives_from → LAW-017, DP-026과 함께 TK-001 생성
       │
       ▼
4. confirm_links(insight_id="TK-001", accepted=["LAW-017"])
       │
       ▼
5. 신뢰도 향상: 0.5 → 0.55
       │
       ▼
6. 나중에: search_insights("마이크로서비스 분리") → TK-001 반환
       │
       ▼
7. find_path("TK-001", "SMELL-03") → 교차 계층 그래프 순회
```

## 관계 유형

| 관계 | 방향 | 설명 |
|----------|-----------|-------------|
| `derives_from` | TK → 정식 | 인사이트가 정식 엔티티에 근거함 |
| `applies_to` | TK → 정식 | 인사이트가 패턴/법칙을 특정 컨텍스트에 적용함 |
| `supersedes` | TK → TK | 새로운 인사이트가 이전 인사이트를 대체함 |
| `related_to` | TK → TK/정식 | 일반적인 시맨틱 연결 |

## CLI 사용법

```bash
# 인사이트 추가
epis insight add "팀에서 God Class 리팩토링 시 Extract Class보다 Facade Pattern이 효과적이었음"

# 인사이트 검색
epis insight search "인증 미들웨어"

# 모든 인사이트 나열
epis insight list
```

## 주요 소스 파일

| 파일 | 역할 |
|------|------|
| `src/domain/composite_graph.rs` | 정식 + 사용자 계층의 런타임 병합 |
| `src/adapters/user_graph_store.rs` | SQLite 기반 `MutableGraphRepository` |
| `src/server/mcp_insight.rs` | 3개의 암묵적 지식 도구를 위한 MCP 핸들러 |
| `src/adapters/insight_utils.rs` | ID 생성, 타임스탬프, 텍스트 유틸리티 |
| `src/domain/types.rs` | `UserEntity`, `LinkProvenance`, `EntityType::Insight` |
| `src/ports/graph.rs` | `MutableGraphRepository` 트레이트 (14개 메서드) |

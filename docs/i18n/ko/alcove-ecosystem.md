# Alcove 생태계 — 아키텍처 및 기능 분석

> Episteme의 암묵적 지식 계층(TK-*)과 Alcove 문서화 생태계의 저장소 모델, 검색 기능, 수명 주기 관리 및 사용 사례 가이드를 다루는 상세 비교입니다.

---

## 1. 아키텍처 개요

### Episteme 암묵적 지식 (TK-*)

| 측면 | 세부 사항 |
|--------|--------|
| **저장소** | SQLite 단일 파일 (`~/.episteme/user_knowledge.db`) |
| **스키마** | 5개 테이블: `user_entities`, `user_relations`, `user_embeddings`, `user_entities_fts` (FTS5 가상), `insight_seq` |
| **단위** | 하나의 인사이트 = 하나의 `UserEntity` 행 (TK-xxx ID) |
| **그래프** | 런타임에 `CompositeGraph`를 통해 정식 그래프와 병합 — 교차 계층 경로 순회 가능 (TK-001 → DP-005 → SMELL-01) |
| **동시성** | MCP + CLI 동시 접근을 위한 `Mutex<Connection>` + WAL 모드 |

### Alcove 문서화 시스템

| 측면 | 세부 사항 |
|--------|--------|
| **저장소** | 파일 시스템의 마크다운 파일 + Tantivy BM25 인덱스 + sqlite-vec 임베딩 |
| **구조** | 3단계 분류: 프로젝트당 핵심(7), 보충(19), 공개(15) 파일 |
| **단위** | 하나의 구조화된 마크다운 파일 (PRD, ARCHITECTURE, DECISIONS 등) |
| **그래프** | wikilink + 파일 경로 기반 느슨한 연결 |
| **동시성** | 문서 루트당 파일 기반 잠금 (`.index_lock`), 볼트별 인덱스 격리 |
| **볼트** | Obsidian PARA 폴더에 대한 3개의 심볼릭 링크: areas(8개 문서), resources(71개), zettelkasten(17개) |

---

## 2. 저장소 모델 비교

### Episteme TK-* 스키마

```sql
-- 핵심 테이블
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,           -- TK-001, TK-002, ...
    title TEXT,                    -- 자동: 첫 번째 줄, 최대 80자
    content TEXT,                  -- 자유 텍스트 (최대 길이 제한 없음)
    author TEXT DEFAULT 'user',
    confidence REAL DEFAULT 0.5,   -- 확인된 링크당 +0.05, 최대 1.0
    evidence_count INTEGER DEFAULT 0,
    last_validated TEXT DEFAULT '',
    tags TEXT DEFAULT '[]',        -- JSON 배열
    relations TEXT DEFAULT '{}',   -- JSON HashMap<relation_type, Vec<entity_id>>
    link_provenance TEXT DEFAULT '{}',
    created_at TEXT,
    updated_at TEXT
);

-- 정규화된 관계 (derives_from, applies_to, supersedes)
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT,
    relation_type TEXT,
    to_id TEXT,
    UNIQUE(from_id, relation_type, to_id)
);

-- FTS5 전문 검색
CREATE VIRTUAL TABLE user_entities_fts USING fts5(title, content, tags, content=user_entities);
```

### Alcove 파일 구조

```
~/.alcove/
  config.toml                    # 전역 설정 (docs_root, core/team/public 파일 목록, 임베딩 모델)
  docs -> symlink                # → Obsidian/SecondBrain/99-Archives/projects
  vaults/
    areas -> symlink             # → Obsidian/02-Areas (8개 문서)
    resources -> symlink         # → Obsidian/03-Resources (71개 문서)
    zettelkasten -> symlink      # → Obsidian/10-Zettelkasten (17개 문서)
  models/                        # 캐시된 ONNX 임베딩 모델
  logs/

<docs_root>/<project>/
  .alcove/
    index/                       # Tantivy BM25 인덱스 파일
    index_meta.json              # 파일 핑거프린트 (mtime + size)
    vectors.db                   # sqlite-vec 임베딩
  PRD.md                         # 제품 요구사항
  ARCHITECTURE.md                # 시스템 설계
  PROGRESS.md                    # 마일스톤 및 상태
  DECISIONS.md                   # 아키텍처 결정 기록
  CONVENTIONS.md                 # 코딩 표준
  SECRETS_MAP.md                 # 환경 변수 및 시크릿
  DEBT.md                        # 기술 부채 등록부
```

---

## 3. 지식 특성

| 차원 | Episteme TK-* | Alcove |
|-----------|---------------|--------|
| **유형** | 순간적 인사이트, 교훈, 팀 결정 | 구조화된 프로젝트 문서 (요구사항, 아키텍처, 결정) |
| **변경 가능성** | 가변 (SQLite CRUD) | 가변 (파일 편집 + 인덱스 재구축) |
| **소스** | 사용자 기여 자유 텍스트 | 사용자 작성 + 템플릿에서 에이전트 생성 |
| **권위** | 개인/팀 관찰 | 팀 위임 / 조직 정책 |
| **세분성** | 원자적 (항목당 하나의 인사이트) | 섹션화 (DECISIONS.md당 여러 ADR) |
| **연결** | 정식 엔티티에 자동 감지 (키워드 점수) | 수동 wikilink + 마크다운 링크 |
| **버전 관리** | 없음 (SQLite 전용) | Git 기반 (파일 = 진실의 원천) |

### 인사이트 수명 주기 (Episteme TK-*)

```
add_insight(text, tags?, project?, linked_entities?)
  │
  ├── TK-xxx ID 생성 (원자적 시퀀스)
  ├── detect_canonical_links() — 키워드 매칭 → 상위 5개 정식 엔티티
  │     score >= 0.5 → 자동 링크 (derives_from)
  │     score < 0.5 → 제안된 링크
  ├── FTS5 중복 감지 → DuplicateCandidate[]
  ├── SQLite + 인메모리 캐시에 저장
  └── 반환: { id, auto_links, suggested_links, duplicates, confidence }

confirm_links(id, accepted[], rejected[])
  │
  ├── derives_from/applies_to 관계 추가
  ├── link_provenance 소스를 "manual"로 업그레이드
  ├── 신뢰도 향상 (링크당 +0.05, 최대 1.0)
  └── 업데이트 저장

search_insights(query, limit?)
  │
  └── FTS5 MATCH 쿼리 → 순위가 매겨진 결과
```

### 문서 수명 주기 (Alcove)

```
init_project(project_name, project_path?)
  │
  ├── 템플릿에서 7개 핵심 문서 생성 (PRD, ARCHITECTURE, ...)
  ├── 선택적으로 공개 문서 생성 (README, CHANGELOG, ...)
  └── 검색 인덱스 재구축

validate_docs()
  │
  ├── 필수 파일 존재 확인
  ├── 템플릿 플레이스홀더 확인 (TODO, FIXME)
  ├── 필수 섹션 제목 확인
  ├── 최소 목록 항목 수 확인
  └── 반환: 파일별 pass/warn/fail

lint_project()
  │
  ├── 끊어진 [[wikilinks]] 및 마크다운 링크 감지
  ├── 고아 파일 찾기 (어떤 문서에서도 링크되지 않은 파일)
  ├── 오래된 마커 찾기 (WIP, TODO, FIXME, DRAFT, DEPRECATED)
  └── 오래된 연도 참조 찾기 (2년 이상)

audit_project()
  │
  ├── 개인 문서 저장소에서 누락된 필수 문서 스캔
  ├── 공개 프로젝트 저장소에서 노출된 내부 문서 스캔
  ├── 파일을 계층으로 분류
  └── 반환: suggested_actions[]
```

---

## 4. 검색 기능

| 기능 | Episteme TK-* | Alcove |
|------------|---------------|--------|
| **엔진** | FTS5 (키워드 매치) | Tantivy BM25 + sqlite-vec 코사인 유사도 |
| **퓨전** | 없음 | RRF (Reciprocal Rank Fusion, k=60) |
| **CJK** | 특별한 지원 없음 | NgramTokenizer (min=2, max=3) |
| **청킹** | 해당 없음 (한 행 = 하나의 인사이트) | 200-500자 청크 |
| **증분** | 해당 없음 (단일 테이블) | mtime + size 핑거프린트 비교 |
| **벡터 검색** | 스키마 존재(`user_embeddings`)하지만 **연결되지 않음** | 완전 작동 (MultilingualE5Small, 384d) |
| **범위** | 단일 데이터베이스 | 프로젝트별 또는 전역 (교차 프로젝트) |
| **폴백** | 없음 | 인덱스 없을 때 grep 부분 문자열 매치 |

---

## 5. 기능 완성도

| 기능 | Episteme TK-* | Alcove |
|---------|---------------|--------|
| 생성 | `add_insight` | `init_project`, 파일 편집 |
| 읽기 | `search_insights` (검색 전용, ID로 가져오기 없음) | `get_doc_file`, `search_project_docs` |
| 업데이트 | MCP를 통해 노출되지 않음 | 직접 파일 편집 + `rebuild_index` |
| 삭제 | MCP를 통해 노출되지 않음 | 파일 삭제 + `rebuild_index` |
| 검증 | 없음 | `validate_docs`, `lint_project` |
| 감사 | 없음 | `audit_project` (공개/비공개 분리) |
| 백업 | 없음 | `backup_vault` (git 커밋 스냅샷) |
| 가져오기 | 없음 | `promote_document` (Obsidian → doc-repo) |
| 정책 | 없음 | enforce 수준이 있는 `policy.toml` |
| 템플릿 | 없음 | 7개 핵심 + 19개 보충 + 15개 공개 |

---

## 6. Alcove 볼트 시스템

Obsidian PARA 구조에 심볼릭 링크된 세 개의 볼트:

| 볼트 | 대상 | 문서 수 | 목적 |
|-------|--------|------|---------|
| `areas` | `02-Areas` | 8 | 도메인 영역: MCP 에이전트, DevOps, Rust, LLM/RAG, 오픈 소스 |
| `resources` | `03-Resources` | 71 | 참조: AWS, 소프트웨어 공학 법칙, 기술 문서 |
| `zettelkasten` | `10-Zettelkasten` | 17 | 원자적 노트: AI 아키텍처, BM25, 지식 그래프, Rust 패턴 |

각 볼트는 독립적인 다음을 가집니다:
- BM25 인덱스 (Tantivy)
- 벡터 데이터베이스 (sqlite-vec)
- 파일 핑거프린트 추적 (`index_meta.json`)
- 캐시 격리 (별도의 `OnceLock<Mutex<HashMap>>`)

---

## 7. Alcove 설정 시스템

### 전역: `~/.alcove/config.toml`

```toml
docs_root = "/path/to/Obsidian/SecondBrain/99-Archives/projects"

[core]
files = ["PRD.md", "ARCHITECTURE.md", "PROGRESS.md", "DECISIONS.md",
         "CONVENTIONS.md", "SECRETS_MAP.md", "DEBT.md"]

[team]
files = ["ENV_SETUP.md", "ONBOARDING.md", "DATA_MODEL.md", "SCHEMA.md",
         "DEPLOYMENT.md", "RUNBOOK.md", "PLAYBOOK.md", "MONITORING.md", ...]  # 19개 파일

[public]
files = ["README.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md", ...]  # 15개 파일

[embedding]
model = "MultilingualE5Small"
auto_download = true
enabled = true
```

### 프로젝트별: `alcove.toml`

`diagram_format`, `core_files`, `team_files`, `public_files`에 대한 전역 기본값을 재정의합니다.

### 정책: `policy.toml`

다음을 정의합니다:
- `enforce` 수준: `strict` | `warn` | `off`
- 섹션 제목과 최소 항목 수가 있는 필수 문서
- 명명 규칙 (`UPPER_SNAKE`, `lower_snake`, `kebab`, `free`)
- 우선순위: 프로젝트 > 팀 > 기본 제공 기본값

---

## 8. 사용 사례 결정 매트릭스

| 상황 | 추천 도구 | 근거 |
|-----------|-----------------|-----------|
| "프로덕션 사고로부터 얻은 교훈을 기록하고 싶다" | **Episteme TK-*** | 향후 교차 참조를 위해 관련 스멜/법칙에 자동 연결 |
| "새 프로젝트를 위한 문서를 시작하고 싶다" | **Alcove** `init_project` | 7개 핵심 템플릿 자동 생성 |
| "오래된 문서가 있는지 확인하고 싶다" | **Alcove** `lint_project` | WIP/TODO/DEPRECATED/오래된 날짜 자동 감지 |
| "팀이 인증 미들웨어에 대해 결정한 것을 찾고 싶다" | **Alcove** `search_project_docs` | BM25 + 벡터로 구조화된 DECISIONS.md 검색 |
| "모듈에서 코드 스멜을 감지하고 싶다" | **Episteme** `analyze_code` | 패턴/정규식 기반 스멜 감지 |
| "PRD에 모든 필수 섹션이 있는지 확인하고 싶다" | **Alcove** `validate_docs` | 정책 기반 섹션 및 항목 수 검증 |
| "인사이트를 Strategy 패턴에 연결하고 싶다" | **Episteme** `confirm_links` | 정식 엔티티에 `derives_from` 간선 생성 |
| "에이전트 접근을 위해 Obsidian 노트를 가져오고 싶다" | **Alcove** `promote_document` | 자동 프로젝트 감지로 doc-repo에 가져오기 |
| "SRP와 Extract Class 간의 관계를 찾고 싶다" | **Episteme** `find_path` | 엔티티 유형에 걸친 다중 홉 그래프 순회 |
| "프로젝트 문서 상태를 백업하고 싶다" | **Alcove** `backup_vault` | 타임스탬프가 있는 git 커밋 스냅샷 |
| "공개 저장소에 노출된 내부 문서를 감사하고 싶다" | **Alcove** `audit_project` | 개인 및 공개 위치 모두 스캔 |
| "코드에 대한 순위가 매겨진 리팩토링 제안을 받고 싶다" | **Episteme** `suggest_refactorings` | 복합 점수: 심각도 x 노력 x 원칙 정렬 |

---

## 9. 상호 보완적 역할

```
Episteme TK-*                     Alcove
"어떤 보편적 원칙이              "우리 팀은 이것에 대해
 여기에 적용되나요?"              무엇을 결정했나요?"

 순간적 인사이트 ←────────────→ 구조화된 결정 기록
 키워드 자동 연결                  템플릿 기반 스캐폴딩
 교차 계층 그래프 순회             교차 프로젝트 문서 검색
 코드 분석 → 스멜 감지            문서 분석 → 오래됨 감지
```

**둘 다 활성화된 경우**: Episteme는 보편적인 "왜"(법칙, 패턴)를 제공하고, Alcove는 프로젝트별 "우리가 결정한 것"(ADR, 관례)을 제공합니다. 에이전트는 두 소스를 인용해야 하며, 팀 규칙이 일반적인 지침과 충돌할 때는 Alcove가 우선합니다.

---

## 10. 규모 및 성능

| 메트릭 | Episteme TK-* | Alcove |
|--------|---------------|--------|
| **설계 용량** | 수백 개의 인사이트 | ~10,000개 파일 |
| **검색 지연 시간** | FTS5 즉시 (인메모리) | 개요의 경우 BM25 < 500ms |
| **토큰 효율성** | 결과당 단일 인사이트 | 상위 5개 청크 ~1.5k 토큰 (grep의 경우 ~8k에 비해) |
| **인덱스 재구축** | 불필요 (FTS5 트리거) | 증분: 변경된 파일만 |
| **모델 크기** | 해당 없음 (연결되지 않음) | 15MB (ArcticEmbedXS) ~ 2.3GB (BGE-M3) |

---

*참고: [Alcove 통합 가이드](./alcove-integration.md)에서 사용 패턴 및 워크플로우 예시를 확인하세요.*

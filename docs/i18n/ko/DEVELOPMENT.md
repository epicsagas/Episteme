# Episteme 개발 가이드

**프로젝트:** Episteme v0.1.0
**언어:** Rust (edition 2024)
**최종 업데이트:** 2026-05-03

---

## 현재 상태

| 구성 요소 | 상태 | 세부 사항 |
|-----------|------|-----------|
| **지식 베이스** | 완료 | 22개 패턴, 66개 리팩토링, 56개 법칙, 23개 스멜, 201개 관계 |
| **코드 스멜 감지** | 프로덕션 | 16개 감지 함수, 10개 언어 |
| **REST API** | 프로덕션 | 17개 엔드포인트 (axum), 속도 제한, 인증 |
| **MCP 서버** | 프로덕션 | 6개 도구, stdio + HTTP 전송 |
| **RAG 파이프라인** | 프로덕션 | SQLite + FTS5 + fastembed (ONNX) |
| **그래프 시각화** | 프로덕션 | D3-force를 사용한 대화형 웹 UI |

---

## 아키텍처

헥사고날 (포트 & 어댑터) 아키텍처:

```
src/
├── commands/          # CLI 하위 명령 핸들러 (clap)
│   ├── analysis.rs    # analyze, infer
│   ├── build.rs       # build (RAG 파이프라인)
│   ├── explore.rs     # explore (search/REPL)
│   ├── graph.rs       # graph 쿼리
│   ├── install.rs     # 설치 마법사 (TUI)
│   ├── service.rs     # MCP HTTP 데몬 관리
│   └── other.rs       # api, mcp, web, telemetry, hooks
├── adapters/          # 인프라 계층
│   ├── regex_parsers.rs   # GenericParser (10개 언어, OnceLock 정규식 캐시)
│   ├── python_ast_parser.rs  # Python AST (rustpython-parser)
│   ├── search_engines.rs  # FTS5 키워드 + 코사인 유사도
│   ├── service.rs         # MCP HTTP 데몬
│   ├── sqlite_db.rs       # SQLite 연결 풀
│   ├── cache.rs           # Redis 캐싱 (선택 사항)
│   └── ...
├── domain/            # 비즈니스 로직 (외부 의존성 없음)
│   ├── graph.rs       # KnowledgeGraph (BFS, subgraph, contradictions, Jaccard)
│   ├── detectors.rs   # TieredAccum이 있는 16개 스멜 감지기
│   ├── engine.rs      # RefactoringInferenceEngine + RefactoringRanker
│   ├── summarizer.rs  # 세부 수준 응답 최적화
│   └── types.rs       # EntityType, RelationType, 핵심 타입
├── server/            # HTTP 계층 (axum)
│   ├── api_routes.rs  # 17개 REST 엔드포인트
│   ├── mcp_handler.rs # MCP 씬 파사드
│   ├── mcp_search.rs  # 검색 서비스
│   ├── mcp_graph.rs   # 그래프 서비스
│   └── mcp_analysis.rs # 코드 분석 서비스
└── ports/             # 트레이트 (헥사고날 경계)
    ├── parser.rs      # CodeParser 트레이트
    ├── search.rs      # SearchEngine 트레이트
    ├── graph.rs       # GraphStore 트레이트
    └── embeddings.rs  # EmbeddingProvider 트레이트
```

---

## 기술 스택

| 구성 요소 | 기술 | 목적 |
|-----------|------|------|
| **언어** | Rust (edition 2024) | 안전성, 성능, 단일 바이너리 |
| **웹 프레임워크** | axum | REST API + MCP HTTP 전송 |
| **데이터베이스** | rusqlite (번들 SQLite) | 지식 그래프 + 벡터 저장소 |
| **검색** | FTS5 + 코사인 유사도 | 키워드 + 시맨틱 하이브리드 검색 |
| **임베딩** | fastembed (ONNX Runtime) | 로컬, 제로 구성 임베딩 생성 |
| **CLI** | clap (derive) | 15개 하위 명령 |
| **Python AST** | rustpython-parser | AST 기반 Python 스멜 감지 |
| **기타 언어** | regex (OnceLock 캐시) | GenericParser 프레임워크 |

---

## 코드 스멜 감지기 (16개)

| ID | 스멜 | 감지 방식 |
|----|------|-----------|
| SMELL-01 | Long Method | LOC 임계값 |
| SMELL-02 | Long Parameter List | 매개변수 수 |
| SMELL-03 | Primitive Obsession | 원시 매개변수 비율 |
| SMELL-04 | Large Class | 메서드 + 필드 수 |
| SMELL-05 | Data Clumps | 반복 매개변수 그룹 (스텁) |
| SMELL-06 | Switch Statements | Switch/match 수 |
| SMELL-07 | Data Class | 메서드 대 필드 비율 |
| SMELL-08 | Temporary Field | 조건부 필드 사용 (스텁) |
| SMELL-09 | Shotgun Surgery | 변경 결합 (스텁) |
| SMELL-10 | Divergent Change | 메서드 응집도 메트릭 |
| SMELL-11 | Lazy Class | 낮은 LOC + 메서드 수 |
| SMELL-12 | Speculative Generality | 구체적 구현 없는 추상화 |
| SMELL-13 | Duplicate Code | 해시 기반 유사도 (부분) |
| SMELL-14 | Middle Man | 위임 비율 |
| SMELL-15 | Parallel Inheritance Hierarchies | 계층 미러링 (스텁) |
| SMELL-16 | Comments | 주석 대 코드 비율 (스텁) |
| SMELL-17 | Dead Code | 도달 불가능/미사용 감지 (스텁) |
| SMELL-18 | Feature Envy | 외부 호출 비율 |
| SMELL-19 | Inappropriate Intimacy | 크로스 클래스 비공개 접근 (스텁) |
| SMELL-20 | Message Chains | 호출 체인 깊이 |
| SMELL-21 | God Object | 복합: LOC + 메서드 + 결합 |
| SMELL-22 | Refused Bequest | 오버라이드 대 없음 비율 (스텁) |
| SMELL-23 | Alternative Classes with Different Interfaces | 인터페이스 편차 (스텁) |

---

## 개발 환경 설정

```bash
# 클론 및 빌드 (Rust 1.95 이상 필요)
git clone https://github.com/epicsagas/Episteme.git
cd Episteme
cargo build

# 테스트 실행
cargo test

# 린트
cargo clippy -- -D warnings

# 로컬 설치 (데이터 시딩 및 DB 빌드 자동 실행)
cargo install --path .
epis install --local
```

---

## API 엔드포인트 (17개)

| 메서드 | 경로 | 설명 |
|--------|------|------|
| GET | `/` | 서비스 정보 |
| GET | `/health` | 상태 확인 |
| GET | `/live` | 활성 프로브 |
| GET | `/ready` | 준비 프로브 |
| GET | `/stats` | 그래프 통계 |
| POST | `/analyze` | 코드 스멜 감지 |
| POST | `/refactor` | 리팩토링 제안 |
| GET | `/search` | 지식 검색 |
| POST | `/search` | 지식 검색 (POST) |
| GET | `/graph/{id}` | 엔티티 조회 |
| GET | `/graph/{id}/neighbors` | 이웃 조회 |
| POST | `/graph/neighbors` | 이웃 조회 (POST) |
| POST | `/graph/subgraph` | 서브그래프 추출 |
| GET | `/graph/path` | 최단 경로 |
| GET | `/graph/contradictions` | 모순 찾기 |
| POST | `/graph/infer-transitive` | 전이적 관계 추론 |
| GET | `/metrics` | Prometheus 메트릭 |

---

## 향후 로드맵

- **IDE 플러그인** — VSCode, IntelliJ 네이티브 통합
- **사용자 정의 엔티티** — 팀별 패턴/스멜 추가
- **팀 메트릭** — 조직 전체의 패턴 사용량 집계
- **다국어 문서** — 한국어, 일본어, 중국어 지식 베이스
- **대화형 튜토리얼** - MCP 도구를 위한 인앱 가이드 투어

---

*최종 업데이트: 2026-05-03*

# SPEC: Phase 1 — Episteme → llm-kernel 마이그레이션

**Status**: draft
**Created**: 2026-06-06
**Goal**: 임베딩, SQLite, 검색 모듈을 llm-kernel v0.1.0으로 교체하여 중복 코드 ~2,400줄 제거

## Context

Episteme는 llm-kernel이 제공하는 기능(임베딩, SQLite 스키마, 하이브리드 검색)을
직접 구현하고 있음. 동일 오너의 4개 프로젝트(velith-engine, research-agent,
knowledge-forge, fmemory)는 이미 llm-kernel에 의존 중. Episteme만 유일한 이탈 상태.

## 임베딩 모델 3티어 추천

fastembed-rs v5.13 기준. **다국어(multilingual)를 기본으로** 선택.
Episteme 지식 그래프 내용은 영어지만, AI 에이전트가 한국어/일본어 쿼리를
직접 전달할 수 있어야 함. 모두 ONNX 백엔드.

### 하 (Low) — 8GB RAM, GPU 없음

| 항목 | 값 |
|------|-----|
| **모델** | `MultilingualE5Small` |
| 차원 | 384 |
| ONNX 크기 | ~130MB |
| 파라미터 | ~33M |
| 컨텍스트 | 512 |
| 다국어 | ✅ 100+ 언어 |
| RAM 피크 | ~300MB |
| query_prefix | `"query: "` |
| doc_prefix | `"passage: "` |
| 기준 | 현재 Episteme AllMiniLML66V2 (영어만, MTEB 42.0) 대비 다국어 + 품질 향상 |

### 중 (Mid) — 16-32GB RAM

| 항목 | 값 |
|------|-----|
| **모델** | `MultilingualE5Base` |
| 차원 | 768 |
| ONNX 크기 | ~420MB |
| 파라미터 | ~110M |
| 컨텍스트 | 512 |
| 다국어 | ✅ 100+ 언어 |
| RAM 피크 | ~1.2GB |
| query_prefix | `"query: "` |
| doc_prefix | `"passage: "` |
| 기준 | 384→768 차원으로 검색 품질 대폭 향상. 표준 워크스테이션 실용 최적점 |

### 상 (High) — 32GB+ RAM

| 항목 | 값 |
|------|-----|
| **모델** | `BGEM3` |
| 차원 | 1024 |
| ONNX 크기 | ~2.27GB |
| 파라미터 | 569M |
| 컨텍스트 | **8192** |
| 다국어 | ✅ 100+ 언어 |
| RAM 피크 | ~4GB |
| 인코딩 | **Dense + Sparse + ColBERT** 삼중 지원 |
| 기준 | 하이브리드 검색(Dense+Sparse) 활용으로 검색 품질 근본적 차이 |

### 티어 선택 이유

```
다국어 MTEB vs 모델 크기

63.0 ┤                                        ● BGEM3 (2.27GB)  Dense+Sparse+ColBERT
     │                                          ╱ 8192 컨텍스트
     │                                        ╱
~56  ┤                          ○ E5-Base (420MB)
     │                        ╱
     │                      ╱
~50  ┤          ● E5-Small (130MB)
     │        ╱
42.0 ┤ △ MiniLM-L6 (80MB) ← 현재 Episteme (영어만)
     └────────────────────────────────────────────────
       0    200    400   600   800  1.2G  1.6G  2.0G  2.4G
```

- **다국어 기본**: AI 에이전트가 한국어 쿼리를 직접 전달해도 정상 임베딩
- **E5 계열**: Microsoft 연구, query/passage prefix로 검색 품질 안정적
- **BGEM3 (상급)**: Dense+Sparse+ColBERT 삼중 인코딩 → llm-kernel `search` 모듈의
  하이브리드 검색과 시너지. Sparse 임베딩이 키워드 매칭을 보완
- **8192 컨텍스트**: 긴 엔티티 설명(패턴 정의, 리팩토링 절차)을 통째로 임베딩 가능
- llm-kernel `FastembedProvider`가 `query_prefix()`/`doc_prefix()` 자동 처리

## Requirements

### R1: llm-kernel 의존성 추가

`Cargo.toml`에 llm-kernel v0.1.0 추가, 필요 피처 게이트 활성화:

```toml
[dependencies]
llm-kernel = { version = "0.1.0", features = [
    "embedding",
    "embedding-fastembed",
    "embedding-openai",
    "store",
    "search",
] }
```

기존 직접 의존 중인 `fastembed`, `ndarray` 제거 (llm-kernel이 트랜짓으로 끌어옴).

### R2: 임베딩 어댑터 교체

**삭제** (총 657줄):
- `src/adapters/embeddings/local_embeddings.rs` (191줄) — `FastembedProvider`로 대체
- `src/adapters/embeddings/openai_embeddings.rs` (230줄) — `OpenAIEmbeddingClient`로 대체
- `src/adapters/embeddings/noop_embeddings.rs` (25줄) — llm-kernel에 해당 기능 없으면 유지 또는 thin wrapper
- `src/ports/embeddings.rs` (12줄) — llm-kernel `EmbeddingProvider` trait 사용

**신규**:
- `src/adapters/embeddings/mod.rs` — llm-kernel `EmbeddingProvider` → Episteme `ports::EmbeddingProvider` 어댑터 (thin wrapper, ~30줄)

**아키텍처**:
```
Episteme ports::EmbeddingProvider (trait 유지)
    ↑ impl
thin adapter (~30줄)
    ↑ delegates to
llm-kernel embedding::EmbeddingProvider
    ↑ impl
FastembedProvider | OpenAIEmbeddingClient
```

hexagonal 아키텍처를 유지: Episteme의 `ports/` trait은 그대로 두고,
`adapters/`에서 llm-kernel 구현체를 래핑.

### R3: 모델 선택 설정

`EpistemeConfig`에 임베딩 모델 설정 추가:

```rust
// config.rs에 추가
pub embedding_model: String,  // "xsq" | "mq" | "lq" | "openai" | custom
```

기본값: `"xsq"` (SnowflakeArcticEmbedXSQ)

env var: `EPISTEME_EMBEDDING_MODEL`

기본값 변경: `"e5-small"` (기존 `"local"` → `"e5-small"`)

모델 매핑:
| 값 | llm-kernel EmbeddingModel | 티어 | 차원 |
|----|--------------------------|------|------|
| `e5-small` (기본) | `MultilingualE5Small` | 하 | 384 |
| `e5-base` | `MultilingualE5Base` | 중 | 768 |
| `bge-m3` | `BGEM3` | 상 | 1024 |
| `openai` | OpenAI API (기존 동작) | — | 1536 |

### R4: SQLite 어댑터 교체

**삭제** (174줄):
- `src/adapters/infra/sqlite_db.rs` 전체

**대체**:
- llm-kernel `store` 모듈의 `init_schema()`, `PRAGMA` 설정 사용
- `Chunk`, `EmbeddingRow` 타입 → llm-kernel store 타입으로 마이그레이션
- `insert_chunks()`, `get_all_embeddings()`, `get_chunk_count()`, `get_embedding_count()`
  → llm-kernel store API로 교체

**스키마 호환성**: 기존 DB 파일과 호환되도록 llm-kernel store의
`init_schema()`가 `CREATE TABLE IF NOT EXISTS` 사용.

### R5: 검색 엔진 교체

**삭제** (909줄):
- `src/adapters/infra/search_engines.rs` 전체

**대체**:
- FTS5 키워드 검색 → llm-kernel `search` 모듈의 BM25
- 코사인 유사도 벡터 검색 → llm-kernel `search` 모듈의 vector search
- RRF 하이브리드 퓨전 → llm-kernel `search` 모듈의 hybrid

`ports/search.rs`의 `SearchIndex` trait은 유지,
`adapters/` 구현체를 llm-kernel 기반으로 교체.

### R6: DirectML 게이트 유지

기존 `#[cfg(all(feature = "directml", target_os = "windows"))]` 로직은
llm-kernel `FastembedProvider` 생성 시 실행 프로바이더 설정으로 이관.

Episteme 측 코드에서는 `llm-kernel`에 DirectML feature를 전달:

```toml
[target.'cfg(windows)'.dependencies]
llm-kernel = { version = "0.1.0", features = ["embedding-fastembed"] }

[features]
directml = ["llm-kernel/embedding-fastembed"]  # Windows DirectML은 fastembed 내부 처리
```

### R7: 기존 임베딩 DB 마이그레이션

차원 변경 시(384 → 768, 1024) 기존 DB를 재빌드해야 함:

1. `epis build --reindex` 커맨드로 전체 재색인
2. 기존 DB를 백업(`embeddings.db.bak`) 후 새 스키마로 재생성
3. 마이그레이션 감지: `PRAGMA user_version` 또는 `embedding_dim` 메타 테이블 비교

## 제외 항목 (Phase 2+)

- MCP 서버 프레임워크 (llm-kernel `mcp` 모듈) — Episteme 도메인 특화 로직 많음
- Config 로딩 (YAML → TOML 전환) — 별도 작업
- 설치 마법사 — Phase 2
- Telemetry — Phase 2
- user_graph_store.rs — llm-kernel `graph` 모듈과 구조가 달라 추가 분석 필요

## Acceptance Criteria

- AC1: `cargo check --all-features` 통과
- AC2: `cargo clippy --all-features -- -D warnings` 통과
- AC3: `cargo test --all-features` 통과 — 기존 테스트 전부 동일 결과
- AC4: `epis build` 가 기존 DB와 동일하게 동작
- AC5: `epis search_knowledge` MCP 툴이 기존과 동일한 결과 품질
- AC6: `MultilingualE5Small` 모델로 embed → 384-dim 벡터 반환
- AC7: `MultilingualE5Base` 모델로 embed → 768-dim 벡터 반환
- AC8: `BGEM3` 모델로 embed → 1024-dim 벡터 반환
- AC9: 한국어 쿼리("팩토리 메서드 패턴") 임베딩 → 영어 쿼리와 의미적 유사도 높게 측정
- AC10: `directml` feature가 Windows에서 정상 작동
- AC11: `--no-default-features --features embedding-openai` 시 OpenAI API 임베딩만 사용 가능
- AC12: 삭제된 코드 줄 수 ≥ 1,500 (임베딩 + SQLite + 검색)

## 구현 순서

```
Step 1: Cargo.toml 업데이트 (의존성 추가/제거)
   ↓
Step 2: ports/embeddings.rs → llm-kernel trait 어댑터
   ↓
Step 3: adapters/embeddings/ → llm-kernel FastembedProvider/OpenAI 래핑
   ↓
Step 4: adapters/infra/sqlite_db.rs → llm-kernel store
   ↓
Step 5: adapters/infra/search_engines.rs → llm-kernel search
   ↓
Step 6: config.rs에 embedding_model 설정 추가
   ↓
Step 7: 기존 테스트 통과 확인 + 신규 테스트 추가
   ↓
Step 8: 삭제된 파일 정리, unused import 제거
```

## Risks

| 리스크 | 완화 |
|--------|------|
| llm-kernel store 스키마가 Episteme와 다름 | `init_schema()` 후 마이그레이션 쿼리 실행 |
| 임베딩 차원 변경 시 기존 DB 무효 | `--reindex` 커맨드로 재빌드 |
| llm-kernel `EmbeddingProvider`가 `anyhow::Result` 반환 | Episteme trait은 `Result<Vec<f32>, String>` → 어댑터에서 변환 |
| `FastembedProvider`가 `query_prefix()` 자동 삽입 | 기존 Episteme 코드에 prefix 중복 방지 필요 |
| fastembed 버전 불일치 (Episteme v5.13 vs llm-kernel의 버전) | llm-kernel에 fastembed 의존을 맞추고 Episteme은 간접 참조 |

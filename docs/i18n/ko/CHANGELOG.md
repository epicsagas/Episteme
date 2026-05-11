# 변경 내역

Episteme의 모든 주요 변경 사항은 이 파일에 기록됩니다.

형식은 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)를 기반으로 하며,
이 프로젝트는 [시맨틱 버전 관리](https://semver.org/spec/v2.0.0.html)를 따릅니다.

## [Unreleased]

### 변경 사항

- CLI: `explore` 명령이 `search`로 이름 변경 (기존 이름은 사용 중단된 별칭으로 유지)
- CLI: `mcp`와 `api`가 전체 서비스 수명 주기를 직접 관리 (`start`, `stop`, `restart`, `status`, `enable [--now]`, `disable [--now]`)
- CLI: `service` 최상위 명령 사용 중단 — 대신 `mcp start/stop/restart/status/enable/disable` 사용
- CLI: `mcp --http` 사용 중단 — HTTP 데몬 모드를 위해 `mcp start` 사용
- CLI: `launchd-install/uninstall/status` 사용 중단 — 대신 `mcp enable/disable/status` 사용
- `enable/disable` 이제 크로스 플랫폼 지원: macOS (launchd) 및 Linux (systemd 사용자 유닛)

### 추가 사항

- `api start/stop/restart/status/enable/disable` — REST API 데몬 수명 주기 관리
- `mcp enable`을 위한 Linux systemd 사용자 유닛 생성

- **Claude Code용 MCP HTTP 전송** — 전송 선택기 TUI, HTTP 기본값, launchd 자동 활성화
- **에이전트 프롬프트 자동 설치** — `epis install`이 Episteme 에이전트 프롬프트를 `~/.claude/agents/`에 복사
- **엔티티 설명** — 마크다운 소스 파일에서 설명 필드 자동 추출, 웹 뷰어 상세 패널에 표시
- **벤치마크 시각화 SPA** — 트렌드 분석, 쿼리 분석 대시보드
- **웹 뷰어 리디자인** — Sankey 다이어그램 레이아웃, 사이드바 트리, 상세 패널, 서브그래프 가독성 개선
- **MCP 설정 업서트** — `epis install` 재실행 시 설정이 다르면 전송 방식 업데이트 (stdio ↔ HTTP)
- **MCP yaml 설정** — `config.yaml`의 `mcp.host` / `mcp.port` (yaml → env 폴백)
- **모니터링** — 환경 변수를 통한 네이티브 및 원격 Prometheus 스크랩 타겟 지원
- **CI 강화** — cargo audit, gitleaks, SBOM 생성, 고정된 액션 SHA
- **릴리스 파이프라인** — Windows 타겟, crates.io 퍼블리싱, Homebrew tap
- `examples/`의 **God module 아키텍처 진단 예제**

### 변경 사항

- **설치 마법사** — 모든 단계(전송, Redis, 원격 측정)가 전체 화면 TUI로 마이그레이션
- **설치 흐름** — 시딩 후 RAG 인덱스 자동 빌드, DB가 이미 존재하면 건너뜀
- **지식 그래프** — 교차 엔티티 시맨틱 관계로 보강
- **라이선스** — MIT → Apache-2.0

### 수정 사항

- 원격 측정을 위한 동기식 `main()`에서 Tokio 런타임 패닉
- 검색 품질 — NDCG 측정 버그 해결, hit@1 정확도 100%로 개선
- 검색 재현율 — 교차 유형 부스팅, 희소 엔티티 처리, 의도 동의어
- fastembed 모델 캐시가 `~/.episteme/models`에 고정
- launchd 부트스트랩 UID 치환 및 포트 사용 중 처리
- CORS 출처를 이제 `EPISTEME_CORS_ORIGINS`로 구성 가능

## [0.1.0] - 2026-05-03

### 추가 사항

- **완전한 Rust 재작성** — Python 코드베이스를 관용적인 Rust로 완전 교체
- **헥사고날 아키텍처** — `ports/` (트레이트), `domain/` (비즈니스 로직), `adapters/` (인프라), `server/` (HTTP)
- **GenericParser 프레임워크** — 8개 중괄호 기반 파서를 `ParserConfig`를 사용하는 `GenericParser`로 통합; 정규식 패턴은 `OnceLock`과 `Box::leak`로 캐시
- **Python AST 파싱** — 정확한 Python 스멜 감지를 위한 `rustpython-parser` (Long Method, Large Class, God Object)
- **TieredAccum + build_detection()** — `detectors.rs`에서 14개의 동일한 스멜 감지 구성 중복 제거 (1,253 → 591줄)
- **MCP 모듈 분해** — `EpistemeMCP`(675줄)를 `mcp_search`, `mcp_graph`, `mcp_analysis` 서비스로 분할
- **CLI 명령 분해** — `main.rs`(1,741줄)를 `commands/` 모듈과 `cli.rs`(clap 정의용)로 분할
- **API 핸들러 중복 제거** — 중복된 `search`/`search_post`를 공유 `do_search()`로 병합
- **16개 스멜 감지 함수** — 기존 14개에서 증가, 모든 GoF 스멜 카테고리 포함
- **17개 REST API 엔드포인트** — 상태 프로브, Prometheus 메트릭, CORS, 속도 제한
- **속도 제한기 TTL 제거** — MAX_BUCKETS=10,000, 1시간 TTL로 무제한 메모리 증가 방지
- **ReDoS 완화** — 삼항 연산자 정규식을 `[^:]+`에서 `[^:\n]{1,50}`으로 제한
- **로컬 임베딩** — 제로 구성 시맨틱 검색을 위한 fastembed (ONNX Runtime)
- **대화형 설치 마법사** — crossterm, vim 키 바인딩, 대체 화면이 있는 TUI
- **배포 패키징** — 자동 DB 부트스트랩이 포함된 릴리스 아카이브 생성을 위한 `episteme dist` 명령
- **크로스 플랫폼 CI** — linux/macOS (x86_64 + aarch64)용 GitHub Actions 릴리스 워크플로우
- **다단계 Dockerfile** — Rust 빌더 + slim Debian 런타임

### 변경 사항

- **언어**: Python 3.11+ → Rust (edition 2024)
- **웹 프레임워크**: FastAPI → axum
- **데이터베이스**: Python sqlite3 → rusqlite (번들)
- **임베딩**: sentence-transformers/PyTorch → fastembed/ONNX Runtime
- **CLI**: argparse → clap (derive)
- **모든 정규식 패턴 캐시** — 전역 `REGEX_CACHE`를 통해 핫 경로에서 재컴파일 제로

### 제거 사항

- Python 런타임 의존성
- ChromaDB 의존성
- tree-sitter 의존성
- PyPI 퍼블리싱 워크플로우
- `episteme-hook` 독립 실행형 바이너리 (Python 전용 PyPI 진입점이었음) — 대신 `episteme hooks ground|sniff|audit` 사용

## [0.0.5] - 2026-04-30

### 추가 사항

- D3-force를 사용한 그래프 시각화 웹 UI (`episteme web`)
- 릴리스 아카이브에 사전 빌드된 벡터 DB
- 개발 워크플로우를 위한 `epis install --local` 플래그
- 모든 161개 엔티티를 포함하는 650개 이상의 시맨틱 관계
- 릴리스 중 CI 자동 벡터 DB 생성

## [0.0.4] - 2026-04-29

### 추가 사항

- 6개 도구가 포함된 MCP 서버
- 4개의 전문화된 에이전트
- `epis install` 명령
- `epis service` 데몬 관리
- 하이브리드 검색 (FTS5 + 벡터)
- Redis 캐싱, GPU 가속
- 10개 언어 코드 스멜 감지
- Prometheus + Grafana 모니터링

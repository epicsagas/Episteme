<p align="center">
<img src="../assets/icon.png" alt="Episteme" width="60%" />
</p>

<p align="center"><sub>Episteme (σύνταγμα) — 그리스어로 "조직화된 체계" 또는 "분별력"</sub></p>

<p align="center">오프라인 우선, 단일 바이너리 지식 그래프로 설계 패턴, 리팩토링 기법, 소프트웨어 법칙을 의미론적 관계로 연결합니다.<br><b>AI 에이전트를 최우선으로 설계</b> — 소프트웨어 엔지니어링 전문 지식을 Claude Code, Cursor 및 기타 MCP 호환 도구에 직접 통합하세요.</p>

<p align="center">Rust로 작성됨 · 단일 바이너리 · 완전 오프라인</p>

<p align="center">
    <a href="https://github.com/epicsagas/Episteme/actions"><img src="https://img.shields.io/github/actions/workflow/status/epicsagas/Episteme/ci.yml?branch=main&label=CI" alt="CI" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.95+-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/episteme"><img src="https://img.shields.io/badge/crates.io-v0.1.0-orange.svg" alt="crates.io" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

<p align="center">
  <a href="../../README.md">English</a> |
  <a href="README.ja.md">日本語</a> |
  한국어 |
  <a href="README.de.md">Deutsch</a> |
  <a href="README.fr.md">Français</a> |
  <a href="README.zh-CN.md">简体中文</a> |
  <a href="README.zh-TW.md">繁體中文</a> |
  <a href="README.pt.md">Português</a> |
  <a href="README.es.md">Español</a> |
  <a href="README.hi.md">हिन्दी</a>
</p>

---

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/features.png">
  <img src="../assets/features.png" align="center" width="100%" alt="Episteme 기능 개요" />
</picture>

---

## 빠른 시작

> **필수 조건:** [rustup](https://rustup.rs)을 통한 Rust 1.95+ · **Rust가 없으신가요?** [Docker](#옵션-3-docker-rust-불필요) 또는 [사전 빌드된 바이너리](#옵션-4-사전-빌드된-바이너리rust-불필요)를 참조하세요.

**1. Rust 설치 (아직 설치하지 않은 경우)**

| OS | 명령어 |
|----|---------|
| **macOS / Linux** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Windows** | [`rustup-init.exe`](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe) 다운로드 후 실행 |

설치 후 **새 터미널**을 열거나(macOS/Linux에서 `source "$HOME/.cargo/env"` 실행) 하세요.

**2. Episteme 설치 (첫 빌드는 3-5분 소요)**

```bash
# Install cargo-binstall if missing
cargo install cargo-binstall

# Fast path (downloads prebuilt binary when available)
cargo binstall episteme

# Fallback (build from source)
cargo install --git https://github.com/epicsagas/Episteme
```

**3. 데이터 시딩 + AI 도구 연결**

```bash
epis install claude    # 또는: cursor, codex, gemini
```

**4. 확인**

```bash
epis --version
epis stats
```

완료되었습니다. Claude Code를 재시작하면 Episteme 도구를 사용할 수 있습니다.

### 30초 만에 체험하기

**옵션 A — CLI:** 프로젝트 내의 아무 파일에나 대해 실행합니다.

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

**옵션 B — Claude Code:** 프로젝트 내의 아무 파일을 열고 자연스럽게 질문합니다.

```
Find code smells in this project and suggest refactorings.
```

Episteme가 자동으로 트리거됩니다 — 특별한 문법이 필요 없습니다. 사용자의 설명을 지식 그래프에 매핑하고 순위가 매겨진 인용 가능한 결과를 반환합니다.

---

## 왜 Episteme인가?

LLM은 이미 Strategy 패턴이 무엇인지 알고 있습니다. SOLID 원칙을 암기하고, GoF 패턴을 나열하며, 코드 스멜을 설명할 수 있습니다. 그렇다면 이 프로젝트는 왜 존재할까요?

**필요한 것은 지식이 아니라 구조화되고 연결된 추론입니다.**

LLM에게 "God Object를 어떻게 고치나요?"라고 물으면 합리적인 답변을 얻을 수 있습니다. 하지만 그 답변은 대화마다 달라지고, 추적 가능성이 없으며, 문제의 근본 원인이나 다운스트림 결과와 연결되지 않습니다. Episteme는 분산된 사실을 탐색 가능한 그래프로 변환하여, 모든 권장 사항이 근거가 있고 인용 가능하며 더 넓은 설계 환경과 연결되도록 합니다.

### 잘 작성된 LLM 프롬프트와는 어떤 차이가 있나요?

| | 잘 작성된 LLM 프롬프트 | Episteme + LLM |
|---|---|---|
| 사전 감지 | 사용자가 올바른 질문을 해야만 가능 | 문제 설명에 자동으로 트리거 |
| 토큰 효율성 | 긴 설명 + 여러 후속 턴 필요 | 하나의 도구 호출로 구조화된 결과 반환 |
| 관계 탐색 | 기껏해야 1홉, 자주 환각 발생 | 다중 홉 그래프 탐색, 검증됨 |
| 교차 참조 | 수동, 오류 발생 가능성 | 201개 의미론적 관계를 통한 자동화 |
| 일관성 | 대화마다 달라짐 | 매번 동일한 구조화된 답변 |
| 인용 가능성 | "Extract Class를 사용해야 할 것 같습니다" | "Extract Class (RF-018), 우선순위 0.89" |
| 오프라인 / 폐쇄망 | 최상의 결과를 위해 인터넷 필요 | 완전 로컬, 단일 바이너리 |

### 언제 유용한가요?

<details>
<summary><b>1. AI 에이전트가 질문을 기다리지 않고 사전에 문제를 감지해야 할 때</b></summary>

MCP 통합은 문제 설명에 자동으로 트리거됩니다. 사용자가 "이 클래스는 너무 많은 일을 한다"고 말하면, 에이전트가 God Object에 대해 질문할 필요가 없습니다 — Episteme가 불만을 `SMELL-03`에 매핑하고, 순위가 매겨진 리팩토링을 제시하며, 위반 사항을 근본 원칙까지 추적합니다. 이는 모호한 불만을 구조화된 개선 계획으로 변환합니다.
</details>

<details>
<summary><b>2. 토큰 소비를 줄이고 싶을 때 — 설명에 낭비하지 않고</b></summary>

Episteme가 없으면 LLM은 "God Object를 어떻게 고치나요?"라는 질문에 스멜을 설명하고, 리팩토링을 나열하며, SOLID 원칙을 설명하고, 각 옵션을 안내합니다 — 응답당 수백 개의 토큰이 소모됩니다. Episteme를 사용하면 하나의 MCP 도구 호출로 `SMELL-03 → RF-018 (0.89) → LAW-001`을 반환합니다. 동일한 전문 지식을 토큰 예산의 일부로 제공합니다.
</details>

<details>
<summary><b>3. 코드 분석이 개선과 연결되어야 할 때 — 단순한 감지만이 아닌</b></summary>

SonarQube 같은 도구는 스멜을 감지합니다. LLM은 패턴을 제안할 수 있습니다. Episteme는 둘 다 수행하고 그것들을 연결합니다: Long Method 감지 → 위반하는 법칙 추적 → 이를 해결하는 리팩토링의 순위 결정 → 그 리팩토링을 강제하는 패턴 표시.
</details>

<details>
<summary><b>4. 분산된 패턴 지식만으로는 부족할 때 — 관계가 필요할 때</b></summary>

Extract Method가 무엇을 하는지 아는 것은 기본입니다. 그것이 Long Method (SMELL-01)를 *해결*하고, 이는 Single Responsibility (LAW-001)를 *위반*하며, 이는 Facade 패턴 (DP-012)에 의해 *강제된다는* 것을 아는 것 — 이것은 LLM이 스스로 안정적으로 구성할 수 없는 추론 체인입니다. Episteme의 201개 의미론적 관계를 통해 AI 에이전트는 이러한 경로를 결정론적으로 탐색할 수 있습니다.
</details>

<details>
<summary><b>5. 아키텍처 결정을 내릴 때 의견이 아닌 근거가 필요할 때</b></summary>

"마이크로서비스를 사용해야 할까요?" — Episteme는 질문을 Conway의 법칙 (LAW-017), SRP (LAW-001), Strangler Fig 패턴 (DP-026)에 연결하고 그 관계를 보여줍니다. 결정이 블로그 게시물이 아닌 엔지니어링 법칙으로 추적 가능해집니다.
</details>

<details>
<summary><b>6. 일관되고 인용 가능한 엔지니어링 조언이 필요할 때 — 환각된 권장 사항이 아닌</b></summary>

모든 발견 사항은 명시적인 엔티티 ID(`DP-005`, `RF-001`, `LAW-021`)를 참조합니다. 권장 사항에는 우선순위 점수와 노력 추정치가 포함됩니다. 동일한 쿼리는 항상 동일한 구조화된 답변을 반환합니다.
</details>

<details>
<summary><b>7. 폐쇄망이나 제한된 네트워크 환경에서 작업할 때</b></summary>

Episteme는 완전히 오프라인으로 실행됩니다: 단일 바이너리, 로컬 SQLite 데이터베이스, fastembed(ONNX Runtime)를 통한 로컬 임베딩. 원격 측정, 폰 홈, 외부 API 호출이 없습니다. 코드와 분석 결과는 사용자의 기기를 떠나지 않습니다.
</details>

---

## 설치

### 옵션 1: 원커맨드 설치 (권장)

```bash
# 첫 빌드는 3-5분 소요 — 정상적인 현상입니다
cargo install --git https://github.com/epicsagas/Episteme
epis install claude    # 데이터 시딩 + MCP 설정 + 에이전트 설치
```

> `epis install claude` 실행 후, MCP 도구와 에이전트가 나타나도록 **Claude Code를 재시작**하세요.

### 옵션 2: 소스에서 빌드

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme && cargo build --release
```

그런 다음 플랫폼에 맞는 바이너리를 실행합니다:

| 플랫폼 | 명령어 |
|----------|---------|
| **macOS / Linux** | `./target/release/epis install --local claude` |
| **Windows** | `.\target\release\episteme.exe install --local claude` |

### 옵션 3: Docker (Rust 불필요)

```bash
docker-compose up -d
```

MCP 설정 파일에 추가합니다:

| 도구 | 설정 파일 경로 |
|------|----------------|
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

### 옵션 4: 사전 빌드된 바이너리 (Rust 불필요)

[GitHub Releases](https://github.com/epicsagas/Episteme/releases)에서 플랫폼에 맞는 최신 바이너리를 다운로드합니다:

| 플랫폼 | 파일 |
|----------|------|
| **macOS** (Apple Silicon) | `episteme-aarch64-apple-darwin.tar.gz` |
| **macOS** (Intel) | `episteme-x86_64-apple-darwin.tar.gz` |
| **Linux** (x86_64) | `episteme-x86_64-unknown-linux-gnu.tar.gz` |
| **Linux** (ARM64) | `episteme-aarch64-unknown-linux-gnu.tar.gz` |
| **Windows** (x86_64) | `episteme-x86_64-pc-windows-msvc.zip` |

```bash
# macOS / Linux
tar xzf episteme-*.tar.gz
sudo mv episteme /usr/local/bin/

# Windows — zip 파일을 압축 해제하고 episteme.exe를 PATH에 추가
```

그런 다음 설치합니다:
```bash
epis install claude    # 또는: cursor, codex, gemini
```

### 확인

```bash
epis --version
epis stats
epis explore "strategy pattern"    # 지식 그래프 탐색
```

---

## MCP 도구 및 에이전트

> **MCP란?** [Model Context Protocol](https://modelcontextprotocol.io)은 AI 도구가 외부 서비스를 호출할 수 있게 하는 개방형 표준입니다. Episteme는 지식 그래프를 Claude Code, Cursor 및 기타 호환 편집기가 자동으로 호출할 수 있는 MCP 도구로 제공합니다.

### 9개 MCP 도구

#### 정규 지식 (6개 도구)

| 도구 | 용도 | 사용 예시 |
|------|------|-----------|
| **`search_knowledge`** | 모든 엔티티에 대한 의미론적 검색 | "재시도 로직에 대한 패턴 찾기" |
| **`get_entity`** | ID로 특정 엔티티의 세부 정보 조회 | "Strategy 패턴 (DP-023) 설명" |
| **`get_neighbors`** | 관련 엔티티 탐색 | "Long Method를 해결하는 리팩토링은?" |
| **`find_path`** | 두 엔티티 간의 연결 경로 탐색 | "SRP와 Extract Class의 관계는?" |
| **`analyze_code`** | 정규식/AST 분석으로 코드 스멜 감지 | "이 결제 검증 코드 리뷰해 줘" |
| **`suggest_refactorings`** | 순위가 매겨진 리팩토링 제안 | "이 클래스에서 리팩토링해야 할 것은?" |

#### 암묵지 (3개 도구)

| 도구 | 용도 | 사용 예시 |
|------|------|-----------|
| **`add_insight`** | 팀 결정, 교훈 기록 | "이벤트 기반 아키텍처를 폴링 대신 선택한 이유" |
| **`search_insights`** | 과거 팀 지식 검색 | "인증 미들웨어에 대해 무슨 결정을 했나요?" |
| **`confirm_links`** | 자동 감지된 정규 엔티티 링크 검증 | TK-001이 SMELL-03과 연관됨을 확인 |

Episteme는 암묵지를 별도의 데이터베이스(`~/.episteme/user_knowledge.db`)에 저장하고, 런타임에 컴포지트 레이어를 통해 정규 그래프와 병합합니다. 팀 인사이트는 패턴, 법칙, 스멜에 자동으로 연결되어 경험을 탐색 가능한 지식으로 변환합니다.

자세한 설계 내용은 [암묵지 아키텍처](../../docs/tacit-knowledge.md)를 참조하세요.

### 4개 전문 에이전트 (연결된 네트워크)

에이전트는 함께 작동합니다 — 각 분석은 다른 에이전트로 전달하는 **다음 단계** 옵션으로 끝납니다.

| 에이전트 | 사용 시기 | 핵심 기능 | 전달 대상 |
|-------|-----------|-----------|-----------|
| **`code-reviewer`** | 코드 스멜, SOLID 위반 | 인과 관계 분석 (근본 원인 → 다운스트림 증상) | advisor, architecture-analyst, refactoring-expert |
| **`episteme-advisor`** | 엔지니어링 결정, 트레이드오프 | 실행 계획이 포함된 다중 엔티티 트레이드오프 체인 | code-reviewer, architecture-analyst, researcher |
| **`episteme-researcher`** | 지식 그래프 탐색 | 패턴, 법칙, 스멜 간의 연결 맵 | advisor, code-reviewer |
| **`architecture-analyst`** | 법칙에 대한 아키텍처 평가 | 위험 가중 평가가 포함된 준수 점수 책정 | advisor, code-reviewer, researcher |

**워크플로우 예시**: `code-reviewer`가 God Object를 감지 → 3개의 다운스트림 스멜까지 인과 관계 추적 → "RF-018 적용"(→ refactoring-expert) 또는 "근본 원인 심층 분석"(→ episteme-advisor) 또는 "아키텍처 검사"(→ architecture-analyst) 옵션 제공.

[전체 MCP 통합 가이드](../../docs/mcp-integration-guide.md)

---

## CLI 사용법

```bash
# 코드 스멜 분석
epis analyze my_code.py --language python --json
episteme infer my_code.py

# 지식 그래프 탐색
epis explore "strategy pattern"
epis graph path DP-005 RF-001   # 예: Factory Method → Extract Method

# RAG 인덱스 빌드
epis build

# 서버 시작
epis api              # :8000에서 REST API
episteme mcp --http       # :43175에서 MCP 서버
episteme web --port 8080  # 웹 UI (대화형 그래프 탐색기)

# 배포 패키징
episteme dist --out-dir release/
```

---

## 기능

| | 기능 | 왜 중요한가 |
|--|------|------------|
| 🧠 | **22개 GoF 설계 패턴** | 실제 예제가 포함된 완전한 카탈로그 |
| 🔧 | **66개 리팩토링 기법** | 코드 샘플이 포함된 Fowler의 카탈로그 |
| ⚖️ | **56개 소프트웨어 법칙 및 원칙** | SOLID, Conway의 법칙, CAP 정리 등 |
| 👃 | **17개 코드 스멜 유형** | Long Method, God Object, Feature Envy 등 ¹ |
| 🔗 | **201개 의미론적 관계** | "해결한다", "강제한다", "위반한다", "관련 있다" |
| 🤖 | **9개 MCP 도구 + 4개 에이전트** | 고품질 AI 에이전트 상호작용 및 에이전트 간 핸드오프 |
| 🌍 | **10개 언어 지원** | Python(AST), Java, TypeScript, Go, Rust, C++, C#, PHP, Ruby, Kotlin |
| 📊 | **결정론적 분석** | AST 기반 Python + 정규식 다중 언어, 매번 동일한 결과 |
| 🏷️ | **인용 가능한 지식** | 모든 발견 사항이 명시적 엔티티 ID(`RF-001`, `LAW-021`)에 연결 |
| 🌐 | **REST API (17개 엔드포인트)** | 인증, 속도 제한, 헬스 프로브, Prometheus 메트릭 |
| 📦 | **단일 바이너리** | 런타임 없음, 크로스 플랫폼 (macOS, Linux, Windows) |
| 🔌 | **로컬 임베딩** | fastembed(ONNX Runtime), 제로 구성 의미론적 검색 |
| 🐳 | **Docker 지원** | 헬스 체크가 포함된 다단계 빌드 |

> ¹ Duplicate Code(SMELL-13)과 Shotgun Surgery(SMELL-09)는 다중 파일 컨텍스트가 필요하며 단일 파일 모드에서는 건너뜁니다.

---

## 문서

| 문서 | 설명 |
|------|------|
| [빠른 시작](../../QUICKSTART.md) | 단계별 설정, 첫 실행, 문제 해결 |
| [MCP 통합 가이드](../../docs/mcp-integration-guide.md) | 도구 참조, 에이전트 예시, 대화 흐름 |
| [API 참조](../../docs/api.md) | REST 엔드포인트, 인증, 예시 |
| [배포](../../docs/distribution.md) | 릴리스 패키징 및 배포 |
| [개발 및 기여](../../DEVELOPMENT.md) | 아키텍처, 기여 방법 |
| [변경 이력](../../CHANGELOG.md) | 릴리스 이력 및 버전 정보 |

---

## 설정

### 환경 변수

```bash
# 데이터 위치
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# API 서버
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEY=your-secret-key

# MCP 서버
EPISTEME_MCP_HOST=127.0.0.1
EPISTEME_MCP_PORT=43175
```

---

## 문제 해결

**설치 후 `episteme` 명령을 찾을 수 없는 경우**

| 플랫폼 | 해결 방법 |
|----------|-----------|
| **macOS / Linux** | `export PATH="$HOME/.cargo/bin:$PATH"` — `~/.bashrc` 또는 `~/.zshrc`에 추가하여 영구 적용 |
| **Windows** | `%USERPROFILE%\.cargo\bin`을 시스템 PATH에 추가하거나 새 터미널 열기 |

**Claude Code / Cursor에서 MCP 도구가 나타나지 않는 경우**

`epis install` 실행 후 편집기를 재시작합니다. 그래도 나타나지 않으면 설정이 올바르게 작성되었는지 확인합니다:
```bash
cat ~/.claude.json   # Claude Code
```

**포트가 이미 사용 중인 경우**
```bash
episteme mcp --http --port 43176   # 다른 포트 사용
```

**첫 시작이 느린 경우**

Episteme는 첫 실행 시 로컬 임베딩 인덱스를 빌드합니다. 30-60초가 소요되며 일회성 비용입니다. 이후 시작은 즉시 이루어집니다.

**`cargo install` 중 컴파일 오류가 발생하는 경우**

Rust 1.95+가 설치되어 있는지 확인합니다:
```bash
rustup update stable
rustup show   # 활성 툴체인 확인
```

> 추가 도움: [QUICKSTART.md 문제 해결 섹션](../../QUICKSTART.md#troubleshooting) · [이슈 열기](https://github.com/epicsagas/Episteme/issues)

---

## 로드맵

- [ ] **사용자 정의 엔티티** — 팀별 패턴/스멜 추가
- [ ] **대화형 튜토리얼** — MCP 도구에 대한 앱 내 가이드 투어
- [ ] **다국어 메타데이터** — 엔티티 제목 및 요약의 한국어, 일본어, 중국어 지원 (README 번역은 완료)
- [ ] **MCP 도구 설명** — IDE 전용 플러그인을 대체하는 향상된 도구 설명
- [ ] **팀 메트릭** — 조직 전체의 패턴 사용 현황 집계

---

## 기여

기여를 환영합니다! 아키텍처 개요와 기여 가이드는 [DEVELOPMENT.md](../../DEVELOPMENT.md)를 참조하세요.

```bash
# 테스트 실행
cargo test

# 린트
cargo clippy -- -D warnings

# 포맷
cargo fmt
```

질문이 있으신가요? [디스커션 열기](https://github.com/epicsagas/Episteme/discussions) 또는 [이슈 등록](https://github.com/epicsagas/Episteme/issues).

---

## 라이선스

Apache 2.0 — 자세한 내용은 [LICENSE](../../LICENSE)를 참조하세요.

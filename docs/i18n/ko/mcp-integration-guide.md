# MCP 통합 가이드

> Episteme의 지식 그래프를 Claude Code, Cursor 및 기타 MCP 호환 AI 도구에 통합하기

## Rust MCP HTTP 모드 (현재)
독립 실행형 HTTP 전송을 직접 사용합니다:

```bash
# HTTP를 통한 MCP 시작
episteme mcp --http --host 127.0.0.1 --port 43175
```

인증 동작:
- `EPISTEME_API_KEYS`가 설정된 경우, 요청에 다음이 포함되어야 합니다:
```http
Authorization: Bearer <api-key>
```
- 키가 설정되지 않은 경우 인증이 생략됩니다 (개발 모드).
- `GET /health`는 상태 확인을 위해 항상 공개됩니다.

참고:
- `epis service`는 동일한 MCP HTTP 모드를 백그라운드에서 관리합니다 (`start|stop|status|enable|disable`).
- 이전 `--proxy` 예제는 사용 중단되었습니다; 대신 `mcp --http`/`service`를 직접 사용하세요.

## MCP란 무엇인가요?

[Model Context Protocol (MCP)](https://modelcontextprotocol.io)은 AI 어시스턴트가 외부 도구와 데이터 소스에 접근할 수 있도록 하는 개방형 표준입니다. Episteme은 AI 에이전트에게 소프트웨어 공학 지식에 직접 접근할 수 있는 6개의 MCP 도구를 제공합니다.

---

## 빠른 시작 (Claude Code)

### 1. Episteme 설치

```bash
# 설치 (Rust 1.95 이상 필요)
cargo install --git https://github.com/epicsagas/Episteme

# Claude Code에 에이전트 및 MCP 서버 설치
# (데이터 시딩 및 MCP 자동 설정)
epis install claude
```

> 데이터 다운로드에 실패하는 경우 소스 설치 사용: `git clone` → `cargo build --release` → `epis install --local`

### 2. 설치 확인

`~/.claude/claude_desktop_config.json` 확인:

```json
{
  "mcpServers": {
    "episteme": {
      "command": "episteme",
      "args": ["mcp"]
    }
  }
}
```

### 3. 사용 시작

Claude Code를 재시작합니다. 이제 6개의 Episteme 도구에 접근할 수 있습니다:

```
사용자: "God Object 스멜을 수정하는 가장 좋은 방법은 무엇인가요?"

Claude (search_knowledge 도구 사용):
  → "God Object" 리팩토링 검색
  → 반환: RF-018 (Extract Class), RF-023 (Move Method)

Claude: "God Object 안티패턴(SMELL-03)은 단일 책임 원칙(LAW-001)을 위반합니다.
추천 리팩토링:

1. Extract Class (RF-018) - 관련 메서드/필드를 새 클래스로 이동
2. Move Method (RF-023) - 메서드를 적절한 클래스로 재배치

둘 다 SOLID 원칙을 강화하고 테스트 가능성을 향상시킵니다."
```

---

## MCP 도구 참조

### 1. `search_knowledge`

**목적**: 모든 엔티티(패턴, 법칙, 리팩토링, 스멜)에 대한 시맨틱 검색

**매개변수**:
```typescript
{
  query: string          // 자연어 쿼리
  top_k?: number         // 반환할 결과 수 (기본값: 5)
  filter_type?: string   // "pattern", "law", "refactoring", "smell"
}
```

**반환값**:
```typescript
{
  results: [{
    entity_id: string     // 예: "DP-023"
    title: string
    entity_type: string
    similarity: number    // 0.0-1.0
    summary: string
  }]
}
```

**대화 예시**:
```
사용자: "코드를 더 테스트 가능하게 만들려면 어떻게 하나요?"

Claude 호출: search_knowledge({
  query: "improve testability",
  top_k: 3
})

반환:
- LAW-001: Single Responsibility Principle
- DP-018: Dependency Injection
- RF-042: Extract Interface

Claude: "테스트 가능성을 향상시키는 세 가지 핵심 접근법:
1. SRP 적용 (LAW-001) - 하나의 클래스, 하나의 변경 이유
2. 의존성 주입 사용 (DP-023) - 의존성 주입
3. 인터페이스 추출 (RF-042) - 외부 의존성 모킹"
```

---

### 2. `get_entity`

**목적**: ID로 특정 엔티티의 전체 세부 정보 조회

**매개변수**:
```typescript
{
  entity_id: string   // 예: "DP-023", "RF-001", "SMELL-01"
}
```

**반환값**:
```typescript
{
  entity_id: string
  title: string
  type: string
  description: string
  implementation: string    // 코드 예시
  when_to_use: string
  benefits: string[]
  trade_offs: string[]
  related_entities: {
    relation_type: string
    target_id: string
    description: string
  }[]
}
```

**대화 예시**:
```
사용자: "Strategy Pattern에 대해 자세히 설명해 주세요"

Claude 호출: get_entity({ entity_id: "DP-023" })

반환:
- Strategy Pattern의 전체 설명
- 사용 시기 (여러 알고리즘, 런타임 전환)
- 구현 (Context, Strategy 인터페이스, ConcreteStrategies)
- 장점: 개방/폐쇄 원칙, 런타임 유연성
- 단점: 클래스 수 증가
- 관련: LAW-002 (개방/폐쇄), RF-034 (조건문을 다형성으로 교체)

Claude: "Strategy Pattern (DP-023)은 알고리즘을 별도의 클래스로 캡슐화합니다...
[코드 예시와 함께 상세 설명 제공]

관련 개념:
- 개방/폐쇄 원칙 강제 (LAW-002)
- 종종 조건문을 다형성으로 교체(RF-034)를 통해 적용됨"
```

---

### 3. `get_neighbors`

**목적**: 주어진 엔티티에 연결된 엔티티 탐색

**매개변수**:
```typescript
{
  entity_id: string
  relation_type?: string   // "solves", "enforces", "violates", "relates_to"
  max_depth?: number       // 기본값: 1
}
```

**반환값**:
```typescript
{
  neighbors: [{
    entity_id: string
    title: string
    relation_type: string
    distance: number
  }]
}
```

**대화 예시**:
```
사용자: "Long Method 스멜을 해결하는 리팩토링은 무엇인가요?"

Claude 호출: get_neighbors({
  entity_id: "SMELL-01",
  relation_type: "solved_by"
})

반환:
- RF-001: Extract Method
- RF-034: Decompose Conditional
- RF-056: Replace Temp with Query

Claude: "Long Method (SMELL-01)은 다음으로 해결할 수 있습니다:
1. Extract Method (RF-001) - 더 작은 메서드로 분리
2. Decompose Conditional (RF-034) - 복잡한 조건 단순화
3. Replace Temp with Query (RF-056) - 임시 변수 제거"
```

---

### 4. `find_path`

**목적**: 두 엔티티 간의 연결 찾기 (예: 법칙이 패턴과 어떻게 관련되는지)

**매개변수**:
```typescript
{
  from_id: string
  to_id: string
  max_depth?: number   // 기본값: 5
}
```

**반환값**:
```typescript
{
  path: [{
    entity_id: string
    title: string
    relation_to_next: string
  }],
  path_found: boolean
  hops: number
}
```

**대화 예시**:
```
사용자: "단일 책임 원칙은 Extract Class 리팩토링과 어떤 관계가 있나요?"

Claude 호출: find_path({
  from_id: "LAW-001",
  to_id: "RF-018"
})

반환 경로:
LAW-001 (SRP) --enforces--> DP-007 (Facade) --achieved_via--> RF-018 (Extract Class)

Claude: "SRP (LAW-001)은 2홉으로 Extract Class (RF-018)에 연결됩니다:
1. SRP는 Facade 패턴(DP-007)을 강제함
2. Facade는 종종 Extract Class(RF-018)를 통해 달성됨

통찰: 클래스에 여러 책임이 있을 때 클래스를 추출하는 것(RF-018)이
SRP(LAW-001)를 만족시키는 데 도움이 됩니다."
```

---

### 5. `analyze_code`

**목적**: AST 분석을 통한 코드 스멜 감지

**매개변수**:
```typescript
{
  code: string
  language: string   // "python", "java", "typescript" 등
  min_confidence?: number   // 기본값: 0.5
}
```

**반환값**:
```typescript
{
  smells_detected: number,
  detections: [{
    smell_id: string
    smell_name: string
    confidence: number
    location: string
    metrics: {
      loc: number
      cyclomatic_complexity: number
      parameter_count: number
    }
  }]
}
```

**대화 예시**:
```
사용자: "이 결제 검증 코드를 리뷰해 주세요"
[코드 붙여넣기]

Claude 호출: analyze_code({
  code: "...",
  language: "python"
})

반환:
- SMELL-01: Long Method (신뢰도 0.87, LOC=45)
- SMELL-08: Long Parameter List (신뢰도 0.92, params=9)

Claude: "2개의 코드 스멜을 발견했습니다:

1. Long Method (SMELL-01, 신뢰도 87%)
   - 45줄의 코드 (임계값: 20)
   - 높은 순환 복잡도 (12)
   - 권장: Extract Method (RF-001)

2. Long Parameter List (SMELL-08, 신뢰도 92%)
   - 9개의 매개변수 (임계값: 4)
   - 권장: Introduce Parameter Object (RF-029)"
```

---

### 6. `suggest_refactorings`

**목적**: 감지된 스멜에 대한 순위가 매겨진 리팩토링 제안

**매개변수**:
```typescript
{
  code: string
  language: string
  top_k?: number   // 제안 수 (기본값: 3)
  min_confidence?: number
}
```

**반환값**:
```typescript
{
  smells_analyzed: number,
  results: [{
    smell: {
      smell_id: string
      smell_name: string
      confidence: number
    },
    suggestions: [{
      refactoring_id: string
      title: string
      priority_score: number
      severity: string
      effort: string
      principles: string[]
    }]
  }]
}
```

**대화 예시**:
```
사용자: "이 코드에서 무엇을 리팩토링해야 하나요?"
[큰 클래스 붙여넣기]

Claude 호출: suggest_refactorings({
  code: "...",
  language: "java",
  top_k: 3
})

God Object 스멜에 대한 순위가 매겨진 제안 반환:
1. RF-018: Extract Class (우선순위 0.89, 심각도 높음, 노력 중간)
2. RF-023: Move Method (우선순위 0.76, 심각도 중간, 노력 낮음)
3. RF-042: Extract Interface (우선순위 0.65, 심각도 낮음, 노력 낮음)

Claude: "이 코드는 God Object 특성(SMELL-03)을 보입니다.

상위 리팩토링 우선순위:

1. Extract Class (RF-018) - 우선순위 0.89
   - 심각도: 높음
   - 노력: 중간
   - 결제 관련 메서드를 PaymentProcessor 클래스로 이동
   - 강제: SRP (LAW-001), SoC (LAW-012)

2. Move Method (RF-023) - 우선순위 0.76
   - 심각도: 중간
   - 노력: 낮음
   - 검증 메서드를 Validator 클래스로 재배치
   - 응집도 향상을 위한 빠른 승리

먼저 #2(Move Method)로 즉각적인 개선을 시작한 다음,
#1(Extract Class)으로 장기적인 설계를 해결하세요."
```

---

## AI 에이전트

Episteme에는 연결된 시스템으로 함께 작동하는 4개의 전문화된 에이전트가 포함되어 있습니다. 각 에이전트는 분석을 위해 지식 그래프 도구를 사용하고 다른 에이전트에 전달할 수 있는 대화형 후속 옵션을 제시합니다.

### 에이전트 네트워크

```
code-reviewer ←→ episteme-advisor
      ↕                ↕
architecture-analyst ←→ episteme-researcher
      ↕
refactoring-expert (외부)
```

각 에이전트는 보고서를 **다음 단계** — 다른 에이전트나 도구에 연결하는 대화형 옵션 — 와 함께 종료합니다. 이를 통해 감지에서 해결까지 종단 간 워크플로우를 만듭니다.

---

### 1. `code-reviewer`

**사용 시기**: 코드에서 스멜, SOLID 위반, 리팩토링 우선순위를 리뷰할 때

**차별화 점**: 감지를 넘어 — 스멜 간의 인과 관계 추적(어떤 스멜이 근본 원인이고 어떤 것이 하위 증상인지), 언어 생태계 관례(Rust, Go 등)에 대한 검증, 상황에 맞는 심각도 보정을 수행합니다.

**사용 도구**: `analyze_code` → `suggest_refactorings` → `get_entity` → `get_neighbors` → `find_path`

**예시 프롬프트**:
```
"이 인증 모듈을 리뷰해 주세요"
[코드 붙여넣기]
```

**에이전트 응답**:
```
# Episteme 코드 리뷰

## 요약
AuthenticationModule에 근본 원인인 God Object 스멜이 있어 3개의
하위 문제를 발생시키고 있습니다. 근본 원인을 수정하면 개선이
연쇄적으로 이어집니다.

## 스멜 감지
| 스멜 | 위치 | 심각도 | 신뢰도 | 관련 원칙 |
|-------|----------|----------|------------|-------------------|
| God Object (SMELL-03) | auth.rs:1-420 | 심각 | 확인됨 | LAW-001 (SRP) |
| Long Method (SMELL-01) | auth.rs:88-142 | 높음 | 확인됨 | LAW-001 (SRP) |
| Divergent Change (SMELL-09) | auth.rs:200-380 | 중간 | 확인됨 | LAW-012 (SoC) |
| Feature Envy (SMELL-15) | auth.rs:290-333 | 중간 | 확인됨 | LAW-003 (LSP) |

## 인과 분석
근본 원인:
- SMELL-03 → 원인 → SMELL-01, SMELL-09
  (God Object 수정 시 연쇄적 개선 발생)

독립적:
- SMELL-15 — 독립적, 하위 영향 없음

## 순위가 매겨진 리팩토링
1. **Extract Class** (RF-018) — 우선순위: 심각
   - 해결: SMELL-03 (근본 원인)
   - 연쇄 효과: SMELL-01, SMELL-09도 해결
   - 수행할 작업: SessionManager, EmailVerifier 클래스 추출

## 다음 단계
1. 리팩토링 — RF-018 (Extract Class) 적용 → refactoring-expert 에이전트 사용
2. 심층 분석 — SMELL-03의 근본 원인 체인 탐색 → episteme-advisor 사용
3. 아키텍처 확인 — 구조적 영향 평가 → architecture-analyst 사용
4. 더 알아보기 — SMELL-03의 전체 설명 → get_entity(detail_level="full")
```

**검증**: 스멜을 확인하기 전에 언어 관용구(예: Rust에서 큰 구조체는 정상), 도메인 컨텍스트(테스트 파일, 생성된 코드), 실제 영향(외형적 vs 개발 차단)을 교차 확인합니다.

---

### 2. `episteme-advisor`

**사용 시기**: 엔지니어링 결정(패턴 선택, 아키텍처 트레이드오프, 근본 원인 심층 분석)을 내릴 때

**사용 도구**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**예시 프롬프트**:
```
"ConfigManager 클래스에 Singleton을 사용해야 하나요?"
```

**에이전트 응답**:
```
# 자문: ConfigManager를 위한 Singleton vs DI

## 적용 가능한 엔티티
### 1. Singleton Pattern (DP-006) - 주요
- **설명**: 클래스에 인스턴스가 하나만 있도록 보장
- **의미**: 전역 설정 접근이지만 숨겨진 결합
- **권장**: 사용 자제 — 테스트 가능성 위반

### 2. Dependency Injection (DP-018) - 대안
- **설명**: 생성자를 통해 의존성 주입
- **의미**: 명시적 의존성, 쉬운 모킹
- **권장**: ConfigManager에 권장

## 트레이드오프
- Singleton (DP-006)은 편의성을 제공하지만 DI (DP-018)를 위반
- DI는 보일러플레이트를 추가하지만 LAW-005 (DIP) 준수 가능
- 해결책: 설정 제공자 인터페이스와 함께 DI 사용

## 실행 계획
1. 즉시: 생성자를 통해 ConfigManager 주입
2. 단기: ConfigProvider 인터페이스 추출
3. 장기: 환경별 설정 전략 고려

## 다음 단계
1. 구현 — DI 패턴 적용 → code-reviewer로 검증
2. 대안 탐색 — 접근법 비교 → episteme-researcher 사용
3. 아키텍처 영향 — 하위 효과 평가 → architecture-analyst 사용
```

---

### 3. `episteme-researcher`

**사용 시기**: 지식 그래프의 관계를 탐색하고 대안을 찾을 때

**사용 도구**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**예시 프롬프트**:
```
"Observer Pattern이 강제하는 법칙은 무엇인가요?"
```

**에이전트 응답**:
```
# 연구 보고서: Observer Pattern 강제 법칙

## 주요 엔티티
1. **Observer Pattern** (DP-012) - 행동
   - 정의: 객체 간의 일대다 의존성 정의
   - 한 객체가 변경될 때 다른 객체가 자동으로 변경됨

## 연결 맵
DP-012 --enforces--> LAW-002 (개방/폐쇄)
DP-012 --enforces--> LAW-005 (의존성 역전)
DP-012 --enforces--> LAW-001 (단일 책임)

## 모순 및 트레이드오프
- Observer (DP-012)는 느슨한 결합을 가능하게 하지만 예기치 않은
  업데이트 체인을 유발할 수 있음 — 업데이트를 중앙화하는
  Mediator (DP-014)와 대조적

## 다음 단계
1. 조언 받기 — 발견 사항을 지침으로 전환 → episteme-advisor 사용
2. 코드 확인 — 코드베이스에서 Observer 감지 → code-reviewer 사용
3. 아키텍처 평가 — 구조적 적합성 평가 → architecture-analyst 사용
```

---

### 4. `architecture-analyst`

**사용 시기**: 시스템 아키텍처를 법칙, 패턴, 구조적 위험에 대해 평가할 때

**사용 도구**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**예시 프롬프트**:
```
"이 마이크로서비스 아키텍처가 Conway's Law를 따르고 있나요?"
[아키텍처 설명]
```

**에이전트 응답**:
```
# 아키텍처 분석: 마이크로서비스

## 법칙 및 원칙 준수
| 원칙 | 상태 | 증거 | 영향 |
|-----------|--------|----------|--------|
| Conway's Law (LAW-042) | 위반 | 배송이 2개 팀에 걸쳐 있음 | 조정 오버헤드 |
| SRP (LAW-001) | 위험 | Analytics가 모든 것에 의존 | 강한 결합 |
| Bounded Context (LAW-031) | 위반 | 명확한 도메인 경계 없음 | 공유 데이터 혼란 |

## 주요 긴장
- Conway's Law (LAW-042)는 팀↔서비스 정렬을 요구하지만
  Shipping 서비스가 Commerce + Platform 팀에 걸쳐 있음
- 추적: LAW-042 → related_to → LAW-001 → enforced_by → DP-026 (Strangler Fig)

## 아키텍처 권장 사항
1. **심각**: Shipping을 Commerce 팀으로 이동 — LAW-042가 조정 실패 예측
2. **높음**: Analytics를 위한 Event Bus 도입 — 비동기 이벤트로 분리
3. **중간**: Bounded Context 정의 — 서비스 경계를 도메인에 정렬

## 준수 점수
- 전체: 5/10 | 구조: 4/10 | 확장성: 6/10 | 유지보수성: 5/10

## 다음 단계
1. 조언 받기 — 주요 긴장 해결 → episteme-advisor 사용
2. 코드 확인 — 구조적 스멜 감지 → code-reviewer 사용
3. 대안 연구 — 더 나은 패턴 찾기 → episteme-researcher 사용
```

---

## 워크플로우 체인

에이전트와 도구는 종단 간 파이프라인으로 연결됩니다. 각 체인은 대화형 후속 옵션이 뒤따르는 보고서를 생성합니다.

### 체인 1: 코드 리뷰 파이프라인
```
analyze_code → suggest_refactorings → get_neighbors("solved_by")
  → find_path(smell_A, smell_B) → 인과 그래프가 포함된 보고서
  → 사용자 선택: 수정 적용 / 심층 분석 / 아키텍처 확인 / 더 알아보기
```

### 체인 2: 아키텍처 리뷰 파이프라인
```
search_knowledge → get_entity → get_neighbors("enforces")
  → get_neighbors("violates") → find_path → 준수 보고서
  → 사용자 선택: 리팩토링 계획 / 자문 / 대안 연구
```

### 체인 3: 문제 진단 파이프라인
```
search_knowledge(증상) → get_entity → get_neighbors("solved_by")
  → 근본 원인 보고서 → 사용자 선택: 수정 적용 / 자문 / 확인
```

### 체인 4: 학습 파이프라인
```
search_knowledge(주제) → get_entity → get_neighbors("related_to")
  → 개념 맵 → 사용자 선택: 코드 예시 / 코드에 적용 / 비교
```

### 크로스 도구 체이닝 규칙

모든 도구 호출은 자연스럽게 다음 단계로 이어집니다:

| 호출 후... | 항상 후속 수행... |
|-------------------|--------------------------|
| `analyze_code` | 감지된 스멜에 대한 `suggest_refactorings` |
| `suggest_refactorings` | 대안을 위해 `get_neighbors(smell_id, "solved_by")` |
| `search_knowledge` | 상위 1-2개 결과에 대한 `get_entity` |
| `get_entity` (스멜) | 영향받는 원칙을 위해 `get_neighbors(id, "violates")` |
| `get_entity` (패턴) | 강제되는 법칙을 위해 `get_neighbors(id, "enforces")` |
| 여러 스멜 감지됨 | 인과 관계 매핑을 위해 `find_path(smell_A, smell_B)` |

---

## 기타 도구 설치

### Cursor

```bash
epis install cursor
```

`~/.cursor/mcp.json`에 MCP 설정 추가:
```json
{
  "mcpServers": {
    "episteme": {
      "command": "episteme",
      "args": ["mcp"]
    }
  }
}
```

### Codex (OpenAI)

```bash
epis install codex
```

프로젝트 루트에 에이전트 정의가 포함된 `AGENTS.md`를 생성합니다.

### 사용자 정의 MCP 통합

도구가 MCP를 지원하는 경우 수동으로 설정:

```json
{
  "mcpServers": {
    "episteme": {
      "command": "/path/to/episteme",
      "args": ["mcp"],
      "env": {
        "EPISTEME_DATA_DIR": "~/.episteme/data",
        "EPISTEME_DB_PATH": "~/.episteme/db/episteme.db"
      }
    }
  }
}
```

---

## 백그라운드 서비스로 실행

성능 향상을 위해 Episteme MCP를 영구적인 HTTP 프록시로 실행:

```bash
# 백그라운드 서비스 시작
epis service start

# 상태 확인
epis service status
# 출력: Running on http://localhost:43175 (PID 12345)

# 부팅 시 자동 시작 활성화 (macOS)
epis service enable

# 서비스 중지
epis service stop
```

HTTP 프록시를 사용하도록 MCP 설정 업데이트:

```json
{
  "mcpServers": {
    "episteme": {
      "command": "episteme",
      "args": ["mcp", "--proxy", "http://localhost:43175"]
    }
  }
}
```

로그: `~/.episteme/logs/mcp.out.log`

---

## 문제 해결

### Claude에 도구가 표시되지 않음

1. 설정 파일 존재 확인: `cat ~/.claude/claude_desktop_config.json`
2. episteme이 PATH에 있는지 확인: `which episteme`
3. MCP 직접 테스트: `episteme mcp`
4. 로그 확인: `tail -f ~/.episteme/logs/mcp.err.log`

### "Database not found" 오류

```bash
# 지식 데이터베이스 재구축
epis build --rebuild
```

### 검색 응답이 느림

```bash
# GPU 가속 사용
epis build --gpu

# 또는 백그라운드 서비스로 실행 (빠른 웜업)
epis service start
```

### 에이전트가 도구를 사용하지 않음

에이전트에 도구 호출 기능이 있는지 확인하세요. Claude Code에서:
```
사용자: "Episteme를 사용하여 재시도 로직 패턴을 찾아줘"
      ^^^^ 도구 사용을 명시적으로 언급
```

---

## 고급: 사용자 정의 지식 통합

Episteme(일반 지식)을 Alcove(팀 지식)과 결합:

```json
{
  "mcpServers": {
    "episteme": {
      "command": "episteme",
      "args": ["mcp"]
    },
    "alcove": {
      "command": "npx",
      "args": ["-y", "@joshuarileydev/alcove-mcp"]
    }
  }
}
```

이중 소스 패턴은 [Alcove 통합 가이드](./alcove-integration.md)를 참조하세요.

---

## API 대안

AI 도구가 MCP를 지원하지 않는 경우 REST API를 사용하세요:

```bash
# API 서버 시작
docker-compose up -d

# 모든 도구에서 사용
curl http://localhost:8000/search?q=strategy+pattern
```

엔드포인트는 [API 문서](./api.md)를 참조하세요.

---

## 자동 트리거링 (Claude Code)

자연어로 문제를 설명하면 Claude Code가 자동으로 의도를 감지하고 적절한 Episteme 도구를 호출합니다 — **Episteme을 명시적으로 언급할 필요가 없습니다**. 아래는 정확한 트리거 패턴과 예시입니다.

### 작동 방식

```
자연어 입력
    ↓ Claude가 키워드/패턴 감지
    ↓ Episteme 도구 자동 호출
    ↓ 지식 그래프가 검증된 데이터 반환
    ↓ (디자인 패턴 · 코드 스멜 · 리팩토링 기법 · 엔지니어링 법칙)
    ↓ Claude의 응답이 증거에 기반함
```

> **참고:** 이것은 프롬프트 기반 자동 트리거링이며, 하드 훅이 아닙니다. 호출을 보장하려면 `/episteme` 스킬을 직접 사용하세요.

### 코드 구조 문제

| 말하는 내용 (예시) | Episteme 감지 내용 | 자동 도구 호출 |
|-------------------------|-----------------------|---------------------|
| "이 클래스가 너무 많은 일을 해요", "이 파일이 300줄이 넘어요" | God Class, Large Class, Single Responsibility | `search_knowledge("god class large class single responsibility")` |
| "이 함수가 너무 길어요", "이 메서드에 줄이 너무 많아요" | Long Method | `search_knowledge("long method extract method")` |
| "코드가 너무 복잡해요", "따라가기 어려워요" | Complexity, Cognitive Overload | `search_knowledge("complexity smell cognitive overload")` |
| "이걸 어디에나 복사 붙여넣기 했어요", "중복 로직이 있어요" | Duplicated Code, Clone | `search_knowledge("duplicated code clone smell")` |

### 결합 및 의존성 문제

| 말하는 내용 (예시) | Episteme 감지 내용 | 자동 도구 호출 |
|-------------------------|-----------------------|---------------------|
| "비즈니스 로직이 DB를 직접 호출해요" | Coupling, Persistence, Repository | `search_knowledge("coupling persistence repository data access layer")` |
| "X를 변경하면 Y가 망가져요", "변경이 모든 곳에 영향을 미쳐요" | Brittle Coupling, Change Propagation | `search_knowledge("brittle coupling change propagation rigidity")` |
| "새 타입을 추가하면 모든 곳을 수정해야 해요", "switch-case가 계속 커져요" | Open/Closed, Strategy, Polymorphism | `search_knowledge("open closed principle strategy polymorphism")` |

### 테스트 및 품질 문제

| 말하는 내용 (예시) | Episteme 감지 내용 | 자동 도구 호출 |
|-------------------------|-----------------------|---------------------|
| "이건 테스트하기 어려워요", "단위 테스트를 작성할 수 없어요" | Testability, Dependency Injection | `search_knowledge("testability dependency injection mockability")` |

### 성능 및 동시성 문제

| 말하는 내용 (예시) | Episteme 감지 내용 | 자동 도구 호출 |
|-------------------------|-----------------------|---------------------|
| "API가 느려요", "응답 시간이 너무 길어요" | N+1 Query, Lazy Loading, Caching | `search_knowledge("N+1 query lazy loading caching performance")` |
| "이게 스레드 안전한가요?", "동시성 문제가 있나요?" | Thread Safety, Race Condition | `search_knowledge("thread safety race condition shared mutable state")` |

### 코드 리뷰 및 분석

| 말하는 내용 (예시) | 자동 도구 호출 |
|-------------------------|---------------------|
| 코드 조각을 공유하고 리뷰 요청 | `analyze_code(code)` → `suggest_refactorings(code)` |
| "이 코드를 개선해 주세요", "이걸 리팩토링해 주세요" | `suggest_refactorings(code)` |

### 아키텍처 논의

| 말하는 내용 (예시) | 자동 도구 호출 |
|-------------------------|---------------------|
| "마이크로서비스 vs 모놀리스", "이걸 어떻게 나누어야 하나요?" | `search_knowledge("monolith microservice decomposition bounded context")` |
| "이 아키텍처가 괜찮은가요?", "이 설계를 리뷰해 주세요" | `search_knowledge("layered architecture coupling responsibility")` |

### 실제 대화 예시

**예시 1 — God Object 감지:**

```
사용자: "UserService가 통제 불능이에요. 인증, 세션 관리,
        이메일 확인, 비밀번호 재설정을 모두 처리하고 있어요."

Claude (자동 호출 search_knowledge):
  → 감지: God Object / Single Responsibility 위반
  → 반환: SMELL-03 (God Object), LAW-001 (SRP), RF-018 (Extract Class)

Claude: "UserService에 God Object 특성(SMELL-03)이 보입니다.
  단일 책임 원칙(LAW-001)을 위반하고 있습니다.

  추천 리팩토링:
  1. Extract Class (RF-018) — 세션 로직을 위한 SessionManager 생성
  2. Move Method (RF-023) — 이메일 확인을 EmailService로 이동
  3. Extract Interface (RF-042) — AuthService 인터페이스 정의

  빠른 승리를 위해 #2로 시작한 다음, 구조적 개선을 위해 #1을 해결하세요."
```

**예시 2 — 성능 조사:**

```
사용자: "상품 목록 API가 정말 느려요. 개별로는 잘 로드되는데
        목록 페이지는 5초 이상 걸려요."

Claude (자동 호출 search_knowledge):
  → 감지: N+1 쿼리 문제
  → 반환: 성능 패턴, Lazy Loading, 캐싱 전략

Claude: "이것은 N+1 쿼리 문제인 것 같습니다 — 각 상품이
  개별 DB 호출을 트리거하는 대신 배치 쿼리를 사용해야 합니다.

  지식 그래프의 해결책:
  1. 배치 로딩 / Eager 로딩 — 한 쿼리로 관련 데이터 가져오기
  2. Data Mapper (DP-019) — 쿼리 로직을 도메인 객체에서 분리
  3. Lazy Loading 적용 (RF-050) — 필요할 때만 관계 로드

  코드에서 N+1 패턴을 확인해 드릴까요?"
```

**예시 3 — 코드 조각으로 리뷰:**

```
사용자: [Python 코드 붙여넣기]
       "이걸 리뷰해 주세요. 뭔가 이상한 것 같아요."

Claude (자동 호출 analyze_code → suggest_refactorings):
  → analyze_code 감지: Long Method (신뢰도 0.87), Long Parameter List (0.92)
  → suggest_refactorings 순위가 매겨진 제안 반환

Claude: "2개의 코드 스멜을 발견했습니다:

  1. Long Method (SMELL-01) — 45줄, 순환 복잡도 12
     → Extract Method (RF-001), 우선순위 0.89

  2. Long Parameter List (SMELL-08) — 9개 매개변수
     → Introduce Parameter Object (RF-029), 우선순위 0.92

  RF-029 (Parameter Object)부터 시작하세요 — 가장 높은 우선순위이며
  이후의 Extract Method를 더 쉽게 만듭니다."
```

---

## 다음 단계

1. **에이전트 체험**: episteme-advisor에게 "Singleton을 사용해야 하나요?"라고 물어보세요
2. **코드 분석**: 함수를 붙여넣고 code-reviewer에게 스멜 확인을 요청하세요
3. **그래프 탐색**: episteme-researcher를 사용하여 패턴 관계를 찾으세요
4. **사용자 정의 워크플로우**: 도구 결합 (분석 → 제안 → 검색)

더 많은 예시는 다음을 참조하세요:
- [Alcove 통합](./alcove-integration.md) — 팀 지식 + Episteme
- [모니터링 설정](../../monitoring/README.md) — 패턴 사용량 추적
- [API 참조](./api.md) — REST 엔드포인트

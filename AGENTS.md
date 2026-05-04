# AGENTS.md — Syntagma

이 프로젝트는 Syntagma MCP 서버를 통해 AI 에이전트와 통합됩니다.

## MCP 서버 연결

MCP 서버는 stdio 모드로 실행됩니다. 각 도구별 설정 방법은 아래를 참고하세요.

### Codex (OpenAI)

`AGENTS.md`가 자동으로 컨텍스트로 포함됩니다. `syntagma install --tool codex`로 설치하세요.

### Gemini CLI

`~/.gemini/mcp.json`에 다음을 추가하세요:

```json
{
  "mcpServers": {
    "syntagma": {
      "command": "syntagma-mcp",
      "args": []
    }
  }
}
```

또는 `syntagma install --tool gemini`를 실행하세요.

### OpenCode

프로젝트 루트의 `.opencode/mcp.json`에 다음을 추가하세요:

```json
{
  "mcpServers": {
    "syntagma": {
      "command": "syntagma-mcp",
      "args": []
    }
  }
}
```

또는 `syntagma install --tool opencode`를 실행하세요.

## 사용 가능한 에이전트

| 에이전트 | 역할 | 언제 사용 |
|---------|------|----------|
| `syntagma-advisor` | 엔지니어링 결정 조언 | 패턴 선택, 리팩토링 우선순위, 아키텍처 트레이드오프 결정 시 |
| `syntagma-researcher` | 지식 그래프 탐색 | 특정 패턴·법칙·스멜 검색 및 개념 간 관계 조사 시 |
| `code-reviewer` | 코드 품질 리뷰 | 코드 스멜 감지, 리팩토링 제안, PR 리뷰 시 |
| `architecture-analyst` | 아키텍처 평가 | 시스템 설계 검토, 확장성 위험 분석, 구조적 결함 식별 시 |

## 사용 가능한 MCP 도구

| 도구 | 설명 |
|------|------|
| `search_knowledge` | 지식 그래프에서 패턴, 법칙, 리팩토링, 스멜 시맨틱 검색 |
| `get_entity` | 특정 엔티티의 상세 정보 조회 (ID 기준: DP-xxx, LAW-xxx, RF-xxx, SMELL-xxx) |
| `get_neighbors` | 엔티티의 관련 연결 노드 탐색 |
| `find_path` | 두 엔티티 사이의 연결 경로 탐색 |
| `analyze_code` | 코드 스멜 자동 감지 분석 |
| `suggest_refactorings` | 감지된 스멜에 대한 리팩토링 제안 |

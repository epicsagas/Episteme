# Episteme — 빠른 시작 가이드

2분 안에 Episteme를 시작하세요.

---

## 필수 조건

- **Rust 1.95 이상** (edition 2024 필요) — [rustup으로 설치](https://rustup.rs)
- 인터넷 연결 (초기 데이터 다운로드용)

---

## 옵션 1: AI 도구 통합 (권장)

**적합한 대상:** Claude Code, Cursor, Codex, Gemini 사용자

```bash
# 1. Episteme 설치
cargo install --git https://github.com/epicsagas/Episteme

# 2. AI 도구에 설치 (데이터 다운로드, MCP 설정, 에이전트 복사)
epis install claude      # Claude Code
epis install cursor      # Cursor
epis install codex       # OpenAI Codex
epis install gemini      # Antigravity
epis install all         # 모든 도구에 한 번에 설치
```

> `epis install claude`에서 데이터 다운로드에 실패하는 경우, 아래의 소스 설치 방식을 대신 사용하세요.

**이것으로 완료됩니다.** AI 도구를 재시작하면 Episteme이 활성화됩니다.

---

## 옵션 2: Docker (Rust 불필요)

```bash
docker-compose up -d

# 접속
# API:       http://localhost:8000
# Health:    http://localhost:8000/health
```

Docker를 통한 MCP 통합의 경우, MCP 설정에 다음을 추가하세요:
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

---

## 옵션 3: 소스에서 설치

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme

# 빌드
cargo build --release

# 데이터 시딩 및 벡터 DB 빌드 (빌드는 자동으로 실행됨)
./target/release/epis install --local
```

---

## 그래프 시각화

Episteme에는 대화형 D3-force 그래프 뷰어가 포함되어 있습니다:

```bash
episteme web               # 기본: http://localhost:8080
episteme web --port 9001   # 사용자 지정 포트
episteme web --host 0.0.0.0 --port 8080  # 네트워크에 공개
```

---

## 자주 사용하는 명령어

```bash
# 코드에서 스멜 분석
epis analyze my_code.py --language python
epis analyze my_code.py --json

# 리팩토링 제안 받기
episteme infer my_code.py --top-k 5

# 지식 그래프 탐색
epis explore "strategy pattern"
epis graph path DP-005 RF-001

# 서버 시작
epis api              # REST API: :8000
episteme mcp --http       # MCP 서버: :43175
episteme web --port 8080  # 웹 UI

# 백그라운드 MCP 데몬 (HTTP 프록시)
epis service start
epis service status
epis service stop

# 릴리스 아카이브 생성
episteme dist --out-dir release
```

---

## 문제 해결

### "Database not found"
```bash
epis install claude   # 데이터 아카이브 재다운로드
# 또는
epis install --local
```

### "Port already in use"
```bash
episteme web --port 9001
epis api --port 9000
```

---

## 다음 단계

- **[README](../../README.md)** — 전체 기능 개요 및 아키텍처
- **[MCP 통합 가이드](./mcp-integration-guide.md)** — 도구 참조 및 에이전트 예시
- **[API 참조](./api.md)** — REST 엔드포인트
- **[기여하기](../../CONTRIBUTING.md)** — 개발 워크플로우

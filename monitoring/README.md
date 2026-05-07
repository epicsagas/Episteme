# Syntagma Monitoring

Prometheus + Grafana 독립 모니터링 스택.
`syntagma-api`는 **네이티브, 원격 머신, 별도 docker** 어디서 실행해도 scrape 가능.

---

## 아키텍처

```
┌─────────────────────────────────────┐     ┌──────────────────────────────┐
│  monitoring docker (독립 네트워크)    │     │  syntagma-api (어디서든 OK)  │
│                                     │     │                              │
│  prometheus ──scrape──────────────────────▶  :8000/metrics              │
│      │                              │     │                              │
│  grafana ◀── query ─── prometheus   │     │  로컬 네이티브               │
│  alertmanager ◀── alert ─ prometheus│     │  원격 머신                   │
│  node-exporter (호스트 시스템 메트릭)│     │  별도 docker host            │
└─────────────────────────────────────┘     └──────────────────────────────┘
```

`SYNTAGMA_API_HOST` 환경변수 하나로 모든 시나리오 대응.

---

## 빠른 시작

### 1. 환경 변수 설정

```bash
cd monitoring
cp .env.example .env
```

`.env` 파일에서 상황에 맞게 `SYNTAGMA_API_HOST` 설정:

| 실행 환경 | SYNTAGMA_API_HOST 값 |
|-----------|----------------------|
| 로컬 네이티브 (Mac/Windows) | `host.docker.internal` |
| 로컬 네이티브 (Linux) | `172.17.0.1` 또는 호스트 IP |
| 원격 머신 | `192.168.1.100` (실제 IP) |

**필수**: `GF_SECURITY_ADMIN_PASSWORD` 반드시 변경

```bash
# .env 최소 설정 예시
SYNTAGMA_API_HOST=host.docker.internal
SYNTAGMA_API_PORT=8000
GF_SECURITY_ADMIN_USER=admin
GF_SECURITY_ADMIN_PASSWORD=your-strong-password
```

### 2. 모니터링 스택 실행

```bash
cd monitoring
docker compose up -d
```

### 3. 접속

| 서비스 | URL | 계정 |
|--------|-----|------|
| Grafana | http://localhost:3000 | .env 설정값 |
| Prometheus | http://localhost:9090 | - |
| Alertmanager | http://localhost:9093 | - |

### 4. Prometheus 타겟 확인

http://localhost:9090/targets 에서 `syntagma-api` 타겟이 **UP** 상태인지 확인.

---

## 시나리오별 설정

### 로컬 네이티브 (Mac / Windows)

```bash
# .env
SYNTAGMA_API_HOST=host.docker.internal
SYNTAGMA_API_PORT=8000
```

`host.docker.internal`은 Docker Desktop이 자동으로 호스트 IP로 resolve.

### 로컬 네이티브 (Linux)

`host.docker.internal`은 Linux Docker에서 기본 미지원.
`docker-compose.yml`의 `extra_hosts: host.docker.internal:host-gateway`로 자동 대응.

```bash
# .env
SYNTAGMA_API_HOST=host.docker.internal
SYNTAGMA_API_PORT=8000
```

또는 호스트 IP를 직접 지정:

```bash
SYNTAGMA_API_HOST=172.17.0.1   # docker0 브릿지 기본 게이트웨이
```

### 원격 머신

```bash
# .env
SYNTAGMA_API_HOST=192.168.1.100   # 원격 머신 IP
SYNTAGMA_API_PORT=8000
```

원격 머신에서 8000 포트가 방화벽에서 열려 있어야 함.

### 여러 인스턴스 동시 모니터링

`prometheus/prometheus.yml.tmpl`에 job 추가:

```yaml
scrape_configs:
  - job_name: 'syntagma-api-prod'
    static_configs:
      - targets: ['prod.example.com:8000']
        labels:
          env: prod

  - job_name: 'syntagma-api-staging'
    static_configs:
      - targets: ['staging.example.com:8000']
        labels:
          env: staging
```

재시작 없이 설정 반영:

```bash
curl -X POST http://localhost:9090/-/reload
```

---

## prometheus.yml 설정 방식

`prometheus/prometheus.yml.tmpl` 템플릿을 컨테이너 시작 시 `envsubst`로 렌더링.

```
prometheus.yml.tmpl  →  envsubst  →  prometheus.yml (컨테이너 내부)
```

렌더링 결과 확인:

```bash
docker exec syntagma-prometheus cat /etc/prometheus/prometheus.yml
```

---

## Grafana 대시보드

```
Grafana → Dashboards → Import
```

- `grafana/syntagma-dashboard.json` — 패턴 사용량 / 코드 품질
- `grafana/syntagma-api-dashboard.json` — API 레이턴시 / 에러율

---

## 운영 명령어

```bash
# 상태 확인
docker compose ps

# 설정 변경 후 Prometheus 무중단 리로드
curl -X POST http://localhost:9090/-/reload

# 로그 확인
docker compose logs -f prometheus

# 중지 (데이터 보존)
docker compose down

# 완전 초기화 (데이터 삭제)
docker compose down -v
```

---

## 트러블슈팅

### syntagma-api 타겟이 DOWN

```bash
# 1. 컨테이너에서 직접 연결 확인
docker exec syntagma-prometheus wget -qO- http://${SYNTAGMA_API_HOST}:${SYNTAGMA_API_PORT}/health

# 2. 렌더링된 prometheus.yml 확인
docker exec syntagma-prometheus cat /etc/prometheus/prometheus.yml

# 3. Linux에서 host.docker.internal 미동작 시
ip route | grep docker   # docker0 게이트웨이 IP 확인 → .env에 직접 지정
```

### Grafana 대시보드 데이터 없음

1. Prometheus targets 페이지 확인: http://localhost:9090/targets
2. 시간 범위 확인 (우상단)
3. Prometheus에서 직접 쿼리 테스트: http://localhost:9090/graph

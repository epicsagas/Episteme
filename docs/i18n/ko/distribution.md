# 배포 패키징 (Rust CLI)

이 가이드에서는 Rust CLI를 사용하여 다른 사용자를 위한 릴리스 데이터 아카이브를 생성하는 방법을 설명합니다.

## 명령어

```bash
episteme dist
```

## `episteme dist`에 포함되는 항목
- `raw/`
- `meta/`
- `data/` (존재하는 경우)
- `db/episteme.db` (임베딩 DB)

출력 아카이브:
- `dist/episteme-data-<version>.tar.gz`

## 자동 빌드 동작
- `~/.episteme/db/episteme.db`가 없으면, `episteme dist`가 먼저 `epis build`를 자동으로 실행합니다.
- 빌드된 DB는 프로젝트 로컬 `db/` 디렉토리에도 복사되어 아카이브에 포함됩니다.
- `epis install --local`은 아카이브(또는 소스 트리 폴백)에서 데이터를 시딩하고 RAG 인덱스를 `~/.episteme/`에 자동으로 빌드합니다.

## 옵션
- `--out-dir <DIR>`: 출력 디렉토리 (기본값: `dist`)
- `--no-db`: DB 포함 건너뛰기
- `--skip-build`: DB가 없어도 자동 빌드하지 않음

예시:

```bash
# dist/에 기본 패키징
episteme dist

# 사용자 지정 출력 디렉토리
episteme dist --out-dir release

# 메타데이터만 패키징 (DB 제외)
episteme dist --no-db

# 엄격 모드: DB가 없으면 실패
episteme dist --skip-build
```

## 검증
아카이브를 생성한 후 구조를 확인합니다:

```bash
tar -tzf dist/episteme-data-*.tar.gz | head -n 30
```

다음 항목이 보여야 합니다:
- `episteme-data-<version>/raw/...`
- `episteme-data-<version>/meta/...`
- `episteme-data-<version>/db/episteme.db` (`--no-db`가 아닌 경우)

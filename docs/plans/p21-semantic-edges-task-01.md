---
type: plan-task
plan: p21-semantic-edges
task: 01
title: Haiku API 클라이언트 + Config + ingest 통합 + CLI 플래그
status: todo
updated_at: 2026-04-13
---

# Task 01 — Haiku API 클라이언트 + Config + ingest 통합 + CLI 플래그

## Changed files

- `crates/secall-core/src/graph/semantic.rs` — 신규 생성
- `crates/secall-core/src/graph/mod.rs:3` — `pub mod semantic;` 추가
- `crates/secall-core/src/vault/config.rs` — `GraphConfig` 추가
- `crates/secall/src/commands/ingest.rs:290-313` — semantic phase 추가
- `crates/secall/src/main.rs` — Ingest 커맨드에 `--no-semantic` 플래그

## Change description

### Step 1: `semantic.rs` 신규 생성

Haiku API 클라이언트 + 폴백 + 저장 로직을 담는 모듈.

```rust
/// Haiku API로 시맨틱 엣지 추출 (보강용)
/// - 입력: frontmatter + 본문 앞 2000자
/// - 출력: Vec<GraphEdge> (confidence: "LLM")
/// - ANTHROPIC_API_KEY 환경변수 필요
pub async fn extract_with_haiku(
    fm: &SessionFrontmatter,
    body: &str,
) -> Result<Vec<GraphEdge>>
```

프롬프트 설계:
- 시스템: "Extract semantic relationships from this agent session log. Return JSON only."
- 입력: frontmatter YAML + summary + body 앞 2000자
- 출력 스키마:
  ```json
  { "edges": [{ "relation": "fixes_bug|modifies_file|introduces_tech|discusses_topic",
                "target_type": "issue|file|tech|topic",
                "target_label": "seCall#15" }] }
  ```
- 모델: `claude-haiku-4-5-20251001`
- max_tokens: 512

API 호출:
- `reqwest::Client` + `https://api.anthropic.com/v1/messages`
- 헤더: `x-api-key`, `anthropic-version: 2023-06-01`
- `ANTHROPIC_API_KEY` 환경변수에서 로드

```rust
/// 규칙 기반 + (옵션) Haiku API로 시맨틱 엣지 추출 후 DB 저장
/// - API key 없거나 호출 실패 시 규칙 기반만 사용
/// - 노드 자동 생성 (issue:N, file:path, tech:X, topic:Y)
pub async fn extract_and_store(
    db: &Database,
    fm: &SessionFrontmatter,
    body: &str,
) -> Result<usize>  // 저장된 엣지 수 반환
```

폴백 순서:
1. 항상 규칙 기반 실행 (`extract_semantic_edges()` 호출)
2. `ANTHROPIC_API_KEY` 존재 시 Haiku 호출
3. Haiku 실패 → `tracing::warn!` + 규칙 결과만 저장
4. 결과 병합: 중복은 `UNIQUE(source, target, relation)` DB 제약으로 자동 방어

### Step 2: `graph/mod.rs` 수정

```rust
pub mod build;
pub mod export;
pub mod extract;
pub mod semantic;  // 추가
```

### Step 3: `config.rs` — GraphConfig 추가

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct GraphConfig {
    /// 시맨틱 엣지 추출 활성화 (기본: true)
    pub semantic: bool,
}
```

`Config` 구조체에 `pub graph: GraphConfig` 필드 추가.
`Default` impl에 `graph: GraphConfig::default()` 추가.

### Step 4: `ingest.rs` — semantic phase 추가

vector_tasks 이후 (line 313 부근):

```rust
// 시맨틱 엣지 추출 (graph build 경유 아닌 ingest 직접 연동)
if config.graph.semantic && !no_semantic && !new_session_ids.is_empty() {
    eprintln!("Extracting semantic edges for {} session(s)...", new_session_ids.len());
    // vault에서 frontmatter + body 읽어서 extract_and_store 호출
    for session_id in &new_session_ids {
        if let Err(e) = /* semantic::extract_and_store(...) */ {
            tracing::warn!(session = %session_id, "semantic extraction skipped: {}", e);
        }
    }
}
```

`no_semantic` 파라미터를 `ingest_sessions()` 함수에 추가.
`run()` 함수에서 CLI 플래그 전달.

### Step 5: `main.rs` — CLI 플래그

Ingest 커맨드에 추가:
```rust
/// Skip semantic edge extraction during ingest
#[arg(long)]
no_semantic: bool,
```

match arm에서 `commands::ingest::run(...)` 호출 시 전달.

## Dependencies

- 없음 (reqwest, serde_json 이미 workspace 의존성)

## Verification

```bash
# 유닛 테스트
cargo test -p secall-core graph::semantic
# 기대: Haiku mock/skip 테스트 통과

# 기존 테스트 회귀 없음
cargo test -p secall-core
cargo test -p secall

# 타입 체크
cargo check --workspace
```

## Risks

- Haiku API 호출이 ingest 핫패스에 추가되어 latency 증가 → `--no-semantic` 으로 회피
- `secall ingest` + `secall graph build` 양쪽에서 시맨틱 엣지 생성 가능 → `UNIQUE` 제약으로 중복 방어 (`INSERT OR IGNORE`)
- `ANTHROPIC_API_KEY` 없는 환경에서 규칙 기반만 동작해야 함 → 폴백 로직 필수

## Scope boundary

수정 금지:
- `crates/secall-core/src/graph/extract.rs` — 기존 규칙 추출 로직 유지
- `crates/secall-core/src/graph/build.rs` — 기존 graph build 내 시맨틱 호출 유지
- `crates/secall-core/src/store/schema.rs` — 스키마 변경 없음
- `crates/secall-core/src/store/graph_repo.rs` — 기존 upsert 함수 그대로 사용

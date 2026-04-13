---
type: plan-task
plan: p21-haiku-api-ingest
task: 01
title: Haiku API 클라이언트 + Config + ingest 통합 + CLI 플래그
status: todo
updated_at: 2026-04-13
---

# Task 01 — Haiku API 클라이언트 + Config + ingest 통합 + CLI 플래그

## Changed files

- `crates/secall-core/src/graph/semantic.rs` — 신규 생성
- `crates/secall-core/src/graph/mod.rs:3` — `pub mod semantic;` 추가
- `crates/secall-core/src/vault/config.rs:8` — `Config` 구조체에 `pub graph: GraphConfig` 필드 추가
- `crates/secall-core/src/store/db.rs:246` — `get_session_vault_path()` 헬퍼 추가
- `crates/secall/src/commands/ingest.rs:290-323` — vector_tasks 이후 semantic phase 추가
- `crates/secall/src/commands/sync.rs:282` — `ingest_sessions` 호출에 `no_semantic` 파라미터 추가
- `crates/secall/src/main.rs:50` — Ingest 커맨드에 `--no-semantic` 플래그 추가

## Change description

### Step 1: `graph/mod.rs` — 모듈 등록

```rust
pub mod build;
pub mod export;
pub mod extract;
pub mod semantic;  // 추가
```

### Step 2: `graph/semantic.rs` — 신규 생성

Haiku API 클라이언트 + 통합 저장 로직.

**2-1. Haiku API 호출 함수**

```rust
/// Haiku API로 시맨틱 엣지 추출 (보강용)
/// - 입력: frontmatter + 본문 앞 2000자
/// - 출력: Vec<GraphEdge> (confidence: "LLM")
/// - ANTHROPIC_API_KEY 환경변수 필요
pub async fn extract_with_haiku(
    fm: &SessionFrontmatter,
    body: &str,
) -> anyhow::Result<Vec<GraphEdge>>
```

- reqwest::Client로 `https://api.anthropic.com/v1/messages` POST
- 헤더: `x-api-key: {ANTHROPIC_API_KEY}`, `anthropic-version: 2023-06-01`, `content-type: application/json`
- 모델: `claude-haiku-4-5-20251001`, max_tokens: 512
- 프롬프트:
  - system: "Extract semantic relationships from this agent session log. Return JSON only."
  - user: frontmatter YAML + summary + body 앞 2000자
  - 출력 스키마:
    ```json
    { "edges": [{ "relation": "fixes_bug|modifies_file|introduces_tech|discusses_topic",
                  "target_type": "issue|file|tech|topic",
                  "target_label": "seCall#15" }] }
    ```
- 응답 파싱: `content[0].text` → JSON → `Vec<GraphEdge>` 변환 (confidence: `"LLM"`, weight: relation별 차등)

**2-2. 통합 저장 함수**

```rust
/// 규칙 기반 + (옵션) Haiku API로 시맨틱 엣지 추출 후 DB 저장
/// - API key 없거나 호출 실패 시 규칙 기반만 사용
/// - 노드 자동 생성 (issue:N, file:path, tech:X, topic:Y)
pub async fn extract_and_store(
    db: &Database,
    fm: &SessionFrontmatter,
    body: &str,
) -> anyhow::Result<usize>  // 저장된 엣지 수 반환
```

폴백 순서:
1. 항상 규칙 기반 실행 (`extract::extract_semantic_edges()` 호출)
2. `ANTHROPIC_API_KEY` 존재 시 Haiku 호출
3. Haiku 실패 → `tracing::warn!` + 규칙 결과만 저장
4. 결과 병합: 중복은 `UNIQUE(source, target, relation)` DB 제약으로 자동 방어 (`INSERT OR IGNORE`)
5. 각 엣지의 target 노드 자동 생성 (`upsert_graph_node`)

**2-3. 테스트**

- `test_extract_and_store_rules_only` — API key 없이 규칙 기반만 동작
- `test_haiku_response_parsing` — JSON 응답 → GraphEdge 변환
- `test_haiku_invalid_json_fallback` — 잘못된 JSON → 규칙 기반 폴백

### Step 3: `vault/config.rs` — GraphConfig 추가

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct GraphConfig {
    /// 시맨틱 엣지 추출 활성화 (기본: true)
    pub semantic: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        GraphConfig { semantic: true }
    }
}
```

`Config` 구조체(line 8)에 `pub graph: GraphConfig` 추가.
`Default` impl(line 173)에 `graph: GraphConfig::default()` 추가.

### Step 4: `commands/ingest.rs` — semantic phase 추가

vector_tasks 처리 이후 (line 313 부근), `Ok(IngestStats {...})` 반환 전:

```rust
// 시맨틱 엣지 추출 (graph build 경유 아닌 ingest 직접 연동)
if config.graph.semantic && !no_semantic && !new_session_ids.is_empty() {
    let vault = secall_core::vault::Vault::new(&config.vault.path);
    eprintln!("Extracting semantic edges for {} session(s)...", new_session_ids.len());
    for session_id in &new_session_ids {
        let short = &session_id[..8.min(session_id.len())];
        // vault에서 세션 마크다운 읽기 → frontmatter + body 파싱
        // semantic::extract_and_store(db, &fm, &body) 호출
        match secall_core::graph::semantic::extract_and_store(db, /* fm, body */).await {
            Ok(n) => tracing::debug!(session = short, edges = n, "semantic edges extracted"),
            Err(e) => tracing::warn!(session = short, "semantic extraction skipped: {}", e),
        }
    }
}
```

`ingest_sessions()` 함수 시그니처에 `no_semantic: bool` 파라미터 추가.
`run()` 함수에서 CLI 플래그를 `ingest_sessions()`에 전달.

### Step 5: `main.rs` — CLI 플래그

Ingest 커맨드(line 33-52)에 추가:

```rust
/// Skip semantic edge extraction during ingest
#[arg(long)]
no_semantic: bool,
```

match arm(line 307-315)에서 `commands::ingest::run(...)` 호출 시 `no_semantic` 전달.

## Dependencies

- 없음 (reqwest 0.12 + serde_json 1 이미 workspace 의존성)

## Verification

```bash
# 신규 테스트
cargo test -p secall-core graph::semantic

# 기존 테스트 회귀 없음
cargo test -p secall-core
cargo test -p secall

# 타입 체크
cargo check --workspace
```

## Risks

- Haiku API 호출이 ingest 핫패스에 추가되어 latency 증가 → `--no-semantic`으로 회피
- `secall ingest` + `secall graph build` 양쪽에서 시맨틱 엣지 생성 가능 → `UNIQUE` 제약으로 중복 방어 (`INSERT OR IGNORE`)
- `ANTHROPIC_API_KEY` 없는 환경에서 규칙 기반만 동작해야 함 → 폴백 로직 필수
- reqwest async 호출을 ingest 내에서 await → 이미 async fn이므로 문제 없음

## Scope boundary

수정 금지:
- `crates/secall-core/src/graph/extract.rs` — 기존 규칙 추출 로직 유지
- `crates/secall-core/src/graph/build.rs` — 기존 graph build 내 시맨틱 호출 유지
- `crates/secall-core/src/store/schema.rs` — 스키마 변경 없음
- `crates/secall-core/src/store/graph_repo.rs` — 기존 upsert 함수 그대로 사용
- `crates/secall-core/src/mcp/server.rs` — MCP 도구 변경 없음

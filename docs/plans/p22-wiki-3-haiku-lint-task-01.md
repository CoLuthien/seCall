---
type: task
plan: p22-wiki-3-haiku-lint
task_number: 1
status: draft
updated_at: 2026-04-13
---

# Task 01 — HaikuBackend + 세션 데이터 전처리

## Changed files

| 파일 | 변경 |
|------|------|
| `crates/secall-core/src/wiki/haiku.rs` | **신규** — Anthropic Messages API 백엔드 |
| `crates/secall-core/src/wiki/mod.rs:1-17` | 수정 — `pub mod haiku` + `pub use haiku::HaikuBackend` 추가 |
| `crates/secall/src/commands/wiki.rs:46-77` | 수정 — 백엔드 선택 match에 `"haiku"` 분기 추가 |
| `crates/secall/src/commands/wiki.rs:119-158` | 수정 — `load_batch_prompt()`, `load_incremental_prompt()` 리팩토링: 세션 데이터 DB 주입 |
| `crates/secall-core/src/store/db.rs` | 수정 — `get_session_with_turns()` 메서드 추가 (세션 메타 + 턴 내용 일괄 조회) |
| `crates/secall-core/src/vault/config.rs:115-135` | 수정 — `WikiConfig` 주석에 `"haiku"` 추가 |
| `docs/prompts/wiki-haiku-system.md` | **신규** — Haiku용 system prompt |

## Change description

### 1. HaikuBackend 구현 (`haiku.rs`)

```rust
pub struct HaikuBackend {
    pub api_key: String,       // ANTHROPIC_API_KEY
    pub model: String,         // claude-haiku-4-5-20251001
    pub max_tokens: u32,       // 4096
    pub system_prompt: String, // wiki-haiku-system.md 내용
}
```

- Anthropic Messages API (`https://api.anthropic.com/v1/messages`) 직접 호출
- `reqwest` 사용 (secall-core에 이미 workspace dependency)
- 헤더: `x-api-key`, `anthropic-version: 2023-06-01`, `content-type: application/json`
- 요청 구조: `{ model, max_tokens, system, messages: [{ role: "user", content }] }`
- 응답에서 `content[0].text` 추출
- `WikiBackend` trait 구현 (`generate(&self, prompt: &str)`)

### 2. 세션 데이터 전처리

현재 `load_batch_prompt()`과 `load_incremental_prompt()`는 프롬프트 텍스트만 반환하고,
실제 세션 데이터 조회는 Claude Code가 MCP 도구로 자율 수행.

Haiku 백엔드에서는 CLI가 직접 데이터를 주입해야 함:

**배치 모드 (`--since`):**
1. `db.get_sessions_for_date()` 또는 since 기준 세션 목록 조회
2. 프로젝트별 그룹핑
3. 각 세션의 요약 + 턴 내용(앞 3KB)을 프롬프트에 주입
4. 프로젝트 단위로 Haiku 호출 (한 프로젝트의 세션들을 하나의 프롬프트에)

**인크리멘탈 모드 (`--session`):**
1. `db.get_session_with_turns(session_id)` — 단일 세션 전문 조회
2. 턴 내용 전체를 프롬프트에 주입 (Haiku 200K 컨텍스트에서 충분)
3. 기존 위키 페이지 목록도 함께 주입 (병합 힌트용)

### 3. DB 메서드 추가

```rust
/// 세션 메타데이터 + 턴 내용을 한번에 조회 (위키 생성용)
pub fn get_session_with_turns(&self, session_id: &str)
    -> Result<(SessionMeta, Vec<TurnRow>)>
```

기존 `get_session_for_embedding()`과 유사하지만 턴 content를 포함.

### 4. system prompt (`wiki-haiku-system.md`)

기존 `wiki-update.md`의 "작성 규칙" 섹션을 Haiku용으로 축약:
- frontmatter 형식 명시 (type, status, updated_at, sources)
- 출력 형식: 마크다운 코드블록 하나 (후처리 파싱용)
- 한국어, 코드/경로 원문 유지
- 세션 데이터가 프롬프트에 이미 포함되어 있으므로 도구 사용 지시 없음

## Dependencies

- 없음 (첫 번째 태스크)
- `reqwest` — secall-core Cargo.toml에 이미 있음 (line 19)
- `ANTHROPIC_API_KEY` 환경변수 — 없으면 명확한 에러 메시지

## Verification

```bash
# 1. 빌드
cargo build -p secall-core -p secall

# 2. 유닛 테스트 (API 호출 없이 프롬프트 조립 로직)
cargo test -p secall-core wiki
cargo test -p secall wiki

# 3. dry-run 테스트 (실제 API 호출 없이 프롬프트 확인)
ANTHROPIC_API_KEY=test secall wiki update --backend haiku --session 86b9d1fa --dry-run

# 4. 통합 테스트 (실제 Haiku 호출, --ignored)
# ANTHROPIC_API_KEY 필요
cargo test -p secall-core wiki::haiku -- --ignored
```

## Risks

- **API 키 노출**: `ANTHROPIC_API_KEY`를 로그에 출력하지 않도록 주의
- **Haiku 모델 ID 변경**: `claude-haiku-4-5-20251001`이 deprecated되면 config로 오버라이드 가능하도록 설계
- **긴 세션 비용**: 342턴 세션(ba6d76f3) 같은 경우 입력 토큰이 많음. 턴 내용을 앞 N KB로 제한하는 옵션 고려

## Scope boundary

**수정 금지:**
- `crates/secall-core/src/wiki/claude.rs` — Task 03 영역
- `crates/secall-core/src/wiki/ollama.rs` — 기존 백엔드 유지
- `crates/secall-core/src/wiki/lmstudio.rs` — 기존 백엔드 유지
- `crates/secall/src/main.rs` — Task 03에서 CLI 플래그 추가
- `docs/prompts/wiki-update.md` — 기존 Claude 백엔드 프롬프트 유지
- `docs/prompts/wiki-incremental.md` — 기존 Claude 백엔드 프롬프트 유지

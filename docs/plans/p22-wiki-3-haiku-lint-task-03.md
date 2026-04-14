---
type: task
plan: p22-wiki-3-haiku-lint
task_number: 3
status: draft
updated_at: 2026-04-13
---

# Task 03 — 검수 단계 + CLI 통합

## Changed files

| 파일 | 변경 |
|------|------|
| `crates/secall/src/main.rs:243-268` | 수정 — `WikiAction::Update`에 `--review` 플래그 추가 |
| `crates/secall/src/commands/wiki.rs:6-93` | 수정 — `run_update()`에 review 분기 추가 |
| `crates/secall-core/src/wiki/review.rs` | **신규** — 검수 로직 (Claude API 호출) |
| `crates/secall-core/src/wiki/mod.rs` | 수정 — `pub mod review` 추가 |
| `crates/secall-core/src/vault/config.rs:115-135` | 수정 — `WikiConfig` 주석 갱신, `review_model` 옵션 추가 |
| `docs/prompts/wiki-review.md` | **신규** — 검수 프롬프트 |

## Change description

### 1. CLI 플래그 추가 (`main.rs`)

```rust
WikiAction::Update {
    // ... 기존 필드 ...

    /// Review generated pages with Sonnet/Opus after generation
    #[arg(long)]
    review: bool,

    /// Review model: sonnet or opus (default: sonnet)
    #[arg(long, default_value = "sonnet")]
    review_model: String,
}
```

### 2. 검수 로직 (`review.rs`)

```rust
pub struct ReviewResult {
    pub page_path: String,
    pub issues: Vec<ReviewIssue>,
    pub approved: bool,
}

pub struct ReviewIssue {
    pub severity: String,    // "error" | "warning" | "info"
    pub description: String,
    pub suggestion: Option<String>,
}

/// 위키 페이지를 Sonnet/Opus에 보내 검수
pub async fn review_page(
    api_key: &str,
    model: &str,          // "claude-sonnet-4-6" | "claude-opus-4-6"
    page_content: &str,
    source_sessions: &str, // 원본 세션 요약 (대조용)
) -> Result<ReviewResult>
```

- Anthropic Messages API 직접 호출 (Task 01의 HaikuBackend와 같은 패턴)
- system prompt: `wiki-review.md`
- user prompt: 위키 페이지 내용 + 원본 세션 요약
- 응답 파싱: JSON 형식으로 issues 배열 반환 요청
- issues가 있으면 자동 수정 시도 또는 목록 출력

### 3. 검수 프롬프트 (`wiki-review.md`)

```markdown
당신은 개발 위키 품질 검수 에이전트입니다.

## 검수 기준
1. **사실 정확성**: 원본 세션 데이터와 위키 내용이 일치하는지
2. **기술 정보 누락**: 코드 스니펫, 설정값, 에러 메시지 등 중요 정보가 빠졌는지
3. **구조 문제**: frontmatter 규격, 마크다운 구조, Obsidian 링크 형식
4. **중복/모순**: 같은 내용 반복, 서로 모순되는 서술

## 출력 형식
JSON: { "issues": [...], "approved": true/false }
```

### 4. `run_update()` 통합

```
기존 파이프라인 (Task 01 + 02)
  → 초안 생성 → 후처리 → 파일 쓰기
  │
  └─ if --review:
       각 생성된 페이지에 대해:
       → review_page() 호출
       → issues 출력
       → severity=error인 것만 자동 수정 시도 (재호출)
       → 최종 결과 리포트
```

### 5. Config 확장

```toml
[wiki]
default_backend = "haiku"
# review_model = "sonnet"  # --review 시 사용할 모델 (기본: sonnet)
```

`WikiConfig`에 `review_model: Option<String>` 추가.
CLI `--review-model`이 있으면 우선, 없으면 config, 없으면 "sonnet" 기본값.

## Dependencies

- **Task 01** — HaikuBackend (API 호출 패턴 재사용)
- **Task 02** — 후처리 파이프라인 (검수 대상인 최종 위키 파일이 있어야 함)
- `ANTHROPIC_API_KEY` — Haiku + 검수 모두 동일 키 사용

## Verification

```bash
# 1. 빌드
cargo build -p secall-core -p secall

# 2. CLI 플래그 확인
cargo run -- wiki update --help 2>&1 | grep -E "review|review-model"

# 3. 유닛 테스트 — 검수 응답 파싱
cargo test -p secall-core wiki::review

# 4. dry-run (검수 프롬프트 확인)
ANTHROPIC_API_KEY=test secall wiki update --backend haiku --session 86b9d1fa --review --dry-run

# 5. 통합 테스트 (실제 API 호출, --ignored)
cargo test -p secall-core wiki::review -- --ignored

# 6. 전체 파이프라인 E2E
# Manual: secall wiki update --backend haiku --session <id> --review
# 확인: wiki/ 파일 생성 + 검수 리포트 출력
```

## Risks

- **검수 비용**: Sonnet 검수 시 페이지당 ~$0.01. 100페이지 = ~$1. Opus면 ~$10
- **검수 루프**: 검수→수정→재검수 무한 루프 방지 — 재시도 1회로 제한
- **API 호출 실패**: 검수 실패 시 초안 그대로 유지 (검수는 optional이므로 실패해도 파이프라인 중단 안 함)

## Scope boundary

**수정 금지:**
- `crates/secall-core/src/wiki/haiku.rs` — Task 01 완성 후 변경 없음
- `crates/secall-core/src/wiki/lint.rs` — Task 02 완성 후 변경 없음
- `crates/secall-core/src/wiki/claude.rs` — 기존 백엔드 유지
- `crates/secall-core/src/wiki/ollama.rs` — 기존 백엔드 유지
- `docs/prompts/wiki-update.md` — 기존 프롬프트 유지
- `raw/sessions/` — immutable

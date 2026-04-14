---
type: task
plan: p22-wiki-3-haiku-lint
task_number: 2
status: draft
updated_at: 2026-04-13
---

# Task 02 — Lint + 후처리 파이프라인

## Changed files

| 파일 | 변경 |
|------|------|
| `crates/secall-core/src/wiki/lint.rs` | **신규** — frontmatter 검증, 기존 위키 병합, Obsidian 링크 생성 |
| `crates/secall-core/src/wiki/mod.rs` | 수정 — `pub mod lint` 추가 |
| `crates/secall/src/commands/wiki.rs` | 수정 — `run_update()` 후반부에 lint 파이프라인 호출 추가 |

## Change description

### 1. `lint.rs` 모듈 구현

Haiku가 생성한 초안 마크다운을 파싱하여 seCall 위키 규격에 맞게 후처리.

#### 1a. frontmatter 검증/교정

```rust
pub struct WikiFrontmatter {
    pub page_type: String,      // "project" | "topic" | "decision"
    pub status: String,         // "draft" | "in_progress" | "done"
    pub updated_at: String,     // "YYYY-MM-DD"
    pub sources: Vec<String>,   // 세션 ID 목록
}

/// Haiku 출력에서 frontmatter를 파싱하고, 누락 필드를 보정
pub fn validate_frontmatter(content: &str, session_ids: &[String]) -> Result<String>
```

- frontmatter가 없으면 생성
- `type`, `status`, `updated_at`, `sources` 필수 필드 누락 시 기본값 삽입
- `sources`에 현재 세션 ID가 없으면 추가
- Haiku가 다른 필드명 사용 시(title→type 등) 매핑

#### 1b. 기존 위키 병합 (append)

```rust
/// 같은 주제의 기존 위키 페이지가 있으면 내용을 append
pub fn merge_with_existing(
    wiki_dir: &Path,
    page_path: &str,       // "projects/seCall.md"
    new_content: &str,
) -> Result<String>
```

- 기존 페이지가 있으면:
  - frontmatter의 `sources` 배열 합치기 (중복 제거)
  - `updated_at` 갱신
  - 본문은 기존 내용 뒤에 `---` 구분선 + 새 내용 append
- 기존 페이지가 없으면 새 파일 생성

#### 1c. Obsidian 링크 생성

```rust
/// 본문에서 세션 ID 참조를 Obsidian 링크로 변환
pub fn insert_obsidian_links(content: &str, known_pages: &[String]) -> String
```

- 세션 ID 패턴(`[0-9a-f]{8}`) → `[[sessions/날짜/세션파일|세션ID]]`
- 이미 `[[...]]` 안에 있는 건 건너뜀
- 알려진 위키 페이지 제목이 본문에 나오면 `[[페이지명]]`으로 변환

#### 1d. markdownlint 연동 (optional)

```rust
/// markdownlint-cli2가 설치되어 있으면 실행, 없으면 skip
pub fn run_markdownlint(file_path: &Path) -> Result<Option<String>>
```

- `which markdownlint-cli2`로 존재 여부 확인
- 있으면 `markdownlint-cli2 --fix <file>` 실행
- 없으면 경고 로그 출력 후 skip
- `.markdownlint.yaml` 이 vault에 있으면 사용

### 2. 파이프라인 통합 (`wiki.rs`)

`run_update()` 흐름:
```
프롬프트 조립 → Haiku 호출 → [초안 텍스트]
  → validate_frontmatter()
  → merge_with_existing()
  → insert_obsidian_links()
  → 파일 쓰기
  → run_markdownlint() (optional)
```

## Dependencies

- **Task 01** 완료 필요 (HaikuBackend + 전처리 → 초안 텍스트가 있어야 후처리 가능)
- `regex` — 세션 ID 패턴 매칭 (secall-core에 이미 있음)
- `markdownlint-cli2` — optional. `npm install -g markdownlint-cli2`로 설치

## Verification

```bash
# 1. 빌드
cargo build -p secall-core -p secall

# 2. 유닛 테스트
cargo test -p secall-core wiki::lint

# 3. frontmatter 검증 테스트 케이스 (유닛)
# - frontmatter 없는 입력 → 생성
# - 필드 누락 → 보정
# - sources 중복 제거
# - Haiku가 title/date/tags 사용 → type/status/updated_at 매핑

# 4. 병합 테스트 (유닛, temp dir)
# - 기존 페이지 있을 때 append
# - 기존 페이지 없을 때 새 파일 생성
# - sources 배열 합치기

# 5. Obsidian 링크 테스트 (유닛)
# - 세션 ID 패턴 → [[링크]] 변환
# - 이미 링크 안에 있는 건 skip

# 6. 통합 (dry-run 후 실제 파일 확인)
# Manual: ls wiki/projects/ wiki/topics/ 에 파일이 생성되었는지 확인
```

## Risks

- **병합 시 내용 중복**: 같은 세션으로 두 번 실행하면 같은 내용이 append됨 → sources 체크로 방지 (이미 포함된 세션이면 skip)
- **Obsidian 링크 오탐**: 8자리 hex가 세션 ID가 아닌 경우 (커밋 해시 등) → session ID 목록과 대조
- **markdownlint --fix 부작용**: 자동 수정이 frontmatter를 깨뜨릴 수 있음 → lint 후 frontmatter 재검증

## Scope boundary

**수정 금지:**
- `crates/secall-core/src/wiki/haiku.rs` — Task 01 영역
- `crates/secall-core/src/wiki/claude.rs` — 기존 백엔드
- `crates/secall-core/src/wiki/ollama.rs` — 기존 백엔드
- `crates/secall/src/main.rs` — Task 03에서 CLI 플래그 추가
- `docs/prompts/wiki-update.md` — 기존 프롬프트 유지
- `raw/sessions/` — immutable, 절대 수정 금지

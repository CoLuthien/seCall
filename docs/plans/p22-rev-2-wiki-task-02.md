---
type: task
plan: p22-rev-2-wiki
task_number: 2
status: draft
updated_at: 2026-04-14
---

# Task 02 — `build_haiku_incremental_prompt()` 기존 위키 페이지 목록 주입

## Changed files

| 파일 | 위치 | 변경 종류 |
|------|------|----------|
| `crates/secall/src/commands/wiki.rs:268-280` | `build_haiku_prompt()` | `wiki_dir: &Path` 파라미터 추가 후 하위 함수에 전달 |
| `crates/secall/src/commands/wiki.rs:282-326` | `build_haiku_incremental_prompt()` | `wiki_dir: &Path` 파라미터 추가, 프롬프트 끝에 기존 위키 목록 섹션 추가 |
| `crates/secall/src/commands/wiki.rs:31-32` | `build_haiku_prompt()` 호출부 | `&wiki_dir` 인자 전달 |

## Change description

### 1. `wiki.rs:268` — `build_haiku_prompt()` 시그니처 변경

```rust
// 변경 전
fn build_haiku_prompt(
    config: &Config,
    session: Option<&str>,
    since: Option<&str>,
) -> Result<String>

// 변경 후
fn build_haiku_prompt(
    config: &Config,
    wiki_dir: &std::path::Path,   // 추가
    session: Option<&str>,
    since: Option<&str>,
) -> Result<String>
```

내부에서 `session`이 있으면 `build_haiku_incremental_prompt(&db, sid, wiki_dir)` 호출.

### 2. `wiki.rs:282` — `build_haiku_incremental_prompt()` 시그니처 변경

```rust
// 변경 전
fn build_haiku_incremental_prompt(db: &Database, session_id: &str) -> Result<String>

// 변경 후
fn build_haiku_incremental_prompt(
    db: &Database,
    session_id: &str,
    wiki_dir: &std::path::Path,   // 추가
) -> Result<String>
```

### 3. `wiki.rs:324` — 프롬프트 끝 기존 위키 목록 섹션 추가

현재 마지막 줄:
```rust
prompt.push_str("위 세션을 바탕으로 위키 페이지를 작성하세요.");
```

변경 후 (위 줄 **앞**에 삽입):
```rust
// 기존 위키 페이지 목록 주입 (병합 힌트)
let existing_pages: Vec<String> = walkdir::WalkDir::new(wiki_dir)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
    .filter_map(|e| {
        e.path()
            .strip_prefix(wiki_dir)
            .ok()
            .map(|rel| rel.to_string_lossy().to_string())
    })
    .collect();

if !existing_pages.is_empty() {
    prompt.push_str("\n## 기존 위키 페이지 목록 (병합 참고용)\n\n");
    for page in &existing_pages {
        prompt.push_str(&format!("- {}\n", page));
    }
    prompt.push_str(
        "\n위 페이지가 이 세션과 관련이 있으면 새 페이지를 만들지 말고 \
         기존 페이지에 통합하도록 판단하세요.\n\n"
    );
}

prompt.push_str("위 세션을 바탕으로 위키 페이지를 작성하세요.");
```

### 4. `wiki.rs:31-32` — 호출부 `wiki_dir` 전달

```rust
// 변경 전
let prompt = if backend_name == "haiku" {
    build_haiku_prompt(&config, session, since)?

// 변경 후
let prompt = if backend_name == "haiku" {
    build_haiku_prompt(&config, &wiki_dir, session, since)?
```

## Dependencies

- 없음 (Task 01, Task 03과 독립적)
- `walkdir` — secall 크레이트에 이미 있음

## Verification

```bash
# 1. 빌드
cargo build -p secall -p secall-core

# 2. dry-run으로 프롬프트에 기존 위키 섹션 포함 여부 확인
# (wiki/ 디렉토리에 파일이 1개 이상 있는 환경 필요)
ANTHROPIC_API_KEY=test cargo run -- wiki update \
    --backend haiku --session 86b9d1fa --dry-run 2>/dev/null \
    | grep "기존 위키 페이지 목록"
# 출력: "## 기존 위키 페이지 목록 (병합 참고용)" 줄이 나와야 함

# wiki/ 가 비어있으면 섹션이 없어야 함 — 빈 경우 자동 skip 확인
# (빌드 성공 + grep 결과만으로 verification 충분)

# 3. 기존 테스트 회귀 없음
cargo test -p secall-core wiki
```

## Risks

- **wiki/ 가 비어있을 때**: `existing_pages.is_empty()` 체크로 섹션 자체를 생략 — 문제 없음
- **경로 인코딩**: 한글 프로젝트명이 포함된 경우 `to_string_lossy()`가 유실 없이 처리
- **프롬프트 길이**: 위키 페이지가 수백 개면 프롬프트가 길어짐. 최대 50개까지만 포함 (`existing_pages.iter().take(50)`)하도록 제한 권장

## Scope boundary

**수정 금지:**
- `crates/secall-core/src/wiki/lint.rs` (Task 01 영역)
- `crates/secall/src/commands/wiki.rs:607-639` (Task 03 영역)
- `crates/secall/src/commands/wiki.rs:328-406` (`build_haiku_batch_prompt` — 배치 모드는 프로젝트별 단일 호출이라 별도 위키 목록 주입 불필요)
- `crates/secall-core/src/wiki/haiku.rs`
- `crates/secall-core/src/wiki/review.rs`
- `crates/secall/src/main.rs`
- `crates/secall/src/commands/sync.rs`

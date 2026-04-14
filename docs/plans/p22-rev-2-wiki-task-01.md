---
type: task
plan: p22-rev-2-wiki
task_number: 1
status: draft
updated_at: 2026-04-14
---

# Task 01 — `insert_obsidian_links()` 위키 페이지 간 내부 링크 추가

## Changed files

| 파일 | 위치 | 변경 종류 |
|------|------|----------|
| `crates/secall-core/src/wiki/lint.rs` | 113-167 | 시그니처 변경 + 위키 페이지 링크 로직 추가 |
| `crates/secall/src/commands/wiki.rs` | 134-136, 197-199 | 호출부 2곳에서 wiki_pages 인자 추가 |

## Change description

### 1. `lint.rs:113` — `insert_obsidian_links()` 시그니처 변경

현재:
```rust
pub fn insert_obsidian_links(
    content: &str,
    session_ids: &[String],
    vault_paths: &std::collections::HashMap<String, String>,
) -> String
```

변경 후:
```rust
pub fn insert_obsidian_links(
    content: &str,
    session_ids: &[String],
    vault_paths: &std::collections::HashMap<String, String>,
    wiki_pages: &[String],   // 추가: wiki 페이지 제목 목록 (확장자 제거된 상대경로)
) -> String
```

### 2. `lint.rs` — 위키 페이지 제목 매치 후 링크 변환 로직 추가

기존 세션 ID 링크 처리 루프 **이후**에 추가:

```rust
// 위키 페이지 간 내부 링크 생성
for page_path in wiki_pages {
    // 페이지 제목 = 경로의 마지막 파일명 (확장자 없이)
    // 예: "projects/seCall" → 제목 "seCall"
    let title = page_path
        .rsplit('/')
        .next()
        .unwrap_or(page_path);

    // 이미 [[...]] 안에 있는 건 건너뜀 (세션 ID 루프와 동일한 in_link 판별)
    // 제목이 본문에 평문으로 나타나면 [[page_path|title]] 로 변환
    // 단, 제목 길이가 3자 미만이면 오탐 위험 → skip
    if title.len() < 3 {
        continue;
    }

    let link = format!("[[{}|{}]]", page_path, title);
    let mut new_result = String::new();
    let mut remaining = result.as_str();

    while let Some(pos) = remaining.find(title) {
        let in_link = { /* 세션 ID 루프와 동일한 in_link 판별 로직 */ };
        if in_link {
            new_result.push_str(&remaining[..pos + title.len()]);
        } else {
            new_result.push_str(&remaining[..pos]);
            new_result.push_str(&link);
        }
        remaining = &remaining[pos + title.len()..];
    }
    new_result.push_str(remaining);
    result = new_result;
}
```

in_link 판별: 세션 ID 루프와 완전히 동일한 로직 (`prefix.rfind("[[")` + `!between.contains("]]")`). 중복을 피하려면 `is_in_obsidian_link(result: &str, abs_pos: usize, match_len: usize) -> bool` 내부 헬퍼로 추출 후 두 루프에서 재사용.

### 3. `wiki.rs:134-136` — 배치 모드 호출부 수정

```rust
// 변경 전
let linked = secall_core::wiki::lint::insert_obsidian_links(
    &merged, &session_ids, &vault_paths,
);

// 변경 후
let wiki_pages = collect_wiki_pages(&wiki_dir);
let linked = secall_core::wiki::lint::insert_obsidian_links(
    &merged, &session_ids, &vault_paths, &wiki_pages,
);
```

### 4. `wiki.rs:197-199` — 인크리멘탈 모드 호출부 수정 (동일 패턴)

### 5. `wiki.rs` — `collect_wiki_pages()` 헬퍼 추가

```rust
/// wiki/ 디렉토리를 스캔하여 페이지 제목 목록 반환
/// 반환값: 확장자 제거된 상대경로 (예: ["projects/seCall", "topics/rust"])
fn collect_wiki_pages(wiki_dir: &std::path::Path) -> Vec<String> {
    walkdir::WalkDir::new(wiki_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
        .filter_map(|e| {
            e.path()
                .strip_prefix(wiki_dir)
                .ok()
                .map(|rel| rel.with_extension("").to_string_lossy().to_string())
        })
        .collect()
}
```

## Dependencies

- 없음 (Task 02, Task 03과 독립적)
- `walkdir` — secall 크레이트에 이미 있음

## Verification

```bash
# 1. 빌드
cargo build -p secall-core -p secall

# 2. 신규 유닛 테스트 (반드시 통과해야 함)
cargo test -p secall-core wiki::lint::tests::test_insert_wiki_page_links
cargo test -p secall-core wiki::lint::tests::test_insert_wiki_page_links_skip_existing
cargo test -p secall-core wiki::lint::tests::test_insert_wiki_page_links_short_title_skip

# 3. 기존 테스트 회귀 없음
cargo test -p secall-core wiki::lint
```

**반드시 추가할 테스트:**

```rust
#[test]
fn test_insert_wiki_page_links() {
    let content = "seCall 프로젝트에서 rust를 사용했다.";
    let wiki_pages = vec!["projects/seCall".to_string()];
    let result = insert_obsidian_links(content, &[], &Default::default(), &wiki_pages);
    assert!(result.contains("[[projects/seCall|seCall]]"));
}

#[test]
fn test_insert_wiki_page_links_skip_existing() {
    let content = "이미 링크된 [[projects/seCall|seCall]] 참조";
    let wiki_pages = vec!["projects/seCall".to_string()];
    let result = insert_obsidian_links(content, &[], &Default::default(), &wiki_pages);
    // 이미 [[]] 안에 있으므로 변환 없음 — 원본과 동일
    assert_eq!(result, content);
}

#[test]
fn test_insert_wiki_page_links_short_title_skip() {
    // 2자 이하 제목은 오탐 위험 → skip
    let content = "AI를 사용했다.";
    let wiki_pages = vec!["topics/AI".to_string()];
    let result = insert_obsidian_links(content, &[], &Default::default(), &wiki_pages);
    assert!(!result.contains("[[topics/AI"));
}
```

## Risks

- **오탐**: 위키 페이지 제목이 일반 단어와 겹칠 수 있음 (예: "rust", "go"). 3자 미만 제목 skip으로 완화.
- **호출 순서**: 세션 ID 링크 처리 후 위키 페이지 링크 처리 순서를 유지해야 함. 역순이면 세션 링크가 위키 페이지 링크 타깃 안에 삽입될 수 있음.

## Scope boundary

**수정 금지:**
- `crates/secall-core/src/wiki/haiku.rs`
- `crates/secall-core/src/wiki/review.rs`
- `crates/secall-core/src/wiki/ollama.rs`
- `crates/secall-core/src/wiki/lmstudio.rs`
- `crates/secall-core/src/wiki/claude.rs`
- `crates/secall-core/src/store/db.rs`
- `crates/secall/src/commands/wiki.rs:283-326` (Task 02 영역)
- `crates/secall/src/commands/wiki.rs:607-639` (Task 03 영역)
- `crates/secall/src/main.rs`
- `crates/secall/src/commands/sync.rs`

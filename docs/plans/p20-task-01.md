---
type: plan-task
plan: p20
task: 01
title: vault/index.rs 문자열 삽입 로직 추출 + 테스트
status: todo
updated_at: 2026-04-12
---

# Task 01 — vault/index.rs 문자열 삽입 로직 추출 + 테스트

## Changed files

- `crates/secall-core/src/vault/index.rs` (전체 66줄)
  - 7~66줄: `update_index()` 내부의 엔트리 생성(50~53줄) + 문자열 삽입(56~62줄) 추출

## Change description

### Step 1: 순수 함수 2개 추출 (파일 I/O와 분리)

`update_index()` 앞에 두 함수를 추가한다:

```rust
/// 인덱스 한 줄 엔트리 생성 (I/O 없음)
pub(crate) fn build_entry_line(
    link_path: &str, title: &str, turns: usize, agent: &str, time_str: &str,
) -> String {
    format!("- [[{}|{}]] — {}턴, {}, {}\n", link_path, title, turns, agent, time_str)
}

/// content에 entry를 삽입하거나 append.
/// - "## Sessions\n\n" 헤더 있으면 직후 삽입 (최신 항목이 맨 위)
/// - 헤더 없으면 content 끝에 "\n## Sessions\n\n{entry}" 추가
pub(crate) fn insert_into_content(content: &mut String, entry: &str) {
    if let Some(pos) = content.find("## Sessions\n\n") {
        let insert_at = pos + "## Sessions\n\n".len();
        content.insert_str(insert_at, entry);
    } else {
        content.push_str("\n## Sessions\n\n");
        content.push_str(entry);
    }
}
```

### Step 2: `update_index()` 리팩터

기존 50~62줄의 인라인 코드를 함수 호출로 교체한다:

```rust
// 기존 50-62줄 전체 → 이것으로 교체:
let new_entry = build_entry_line(&link_path, &title, turns, agent, &time_str);
insert_into_content(&mut content, &new_entry);
```

### Step 3: `#[cfg(test)]` 모듈 추가 (파일 하단)

| 테스트명 | 검증 내용 |
|----------|-----------|
| `test_build_entry_line_format` | 출력 포맷이 `- [[link\|title]] — N턴, agent, HH:MM\n` 형식 |
| `test_insert_with_header` | "## Sessions\n\n" 존재 시 직후 삽입 확인 |
| `test_insert_creates_header` | 헤더 부재 시 "\n## Sessions\n\n" + entry append |
| `test_insert_empty_content` | 빈 String에서도 정상 동작 |
| `test_insert_preserves_existing` | 새 엔트리가 기존 항목 앞에 삽입됨 |

## Dependencies

- 없음 (secall-core 내부, 다른 task와 독립)

## Verification

```bash
cargo test -p secall-core vault::index
# 기대: 5 passed, 0 failed

cargo check -p secall-core
# 기대: exit 0
```

## Risks

- `update_index()`는 `vault.write_session()` → `sync.rs` 경로로 호출됨.
  리팩터 후 외부 동작이 동일해야 한다. Step 2에서 인라인 코드와 함수 호출이 1:1 대응하는지 diff로 확인.
- `content.insert_str()`는 바이트 위치 기반. UTF-8 멀티바이트가 헤더 앞에 있어도
  `find()` 반환값이 char 경계임을 Rust가 보장하므로 문제없음.

## Scope boundary

수정 금지:
- `crates/secall-core/src/vault/mod.rs`
- `crates/secall-core/src/vault/git.rs` (Task 02 영역)
- `crates/secall/src/commands/` (Task 03 영역)

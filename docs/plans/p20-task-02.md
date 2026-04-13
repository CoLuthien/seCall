---
type: plan-task
plan: p20
task: 02
title: vault/git.rs diff --stat 파싱 로직 추출 + 테스트
status: todo
updated_at: 2026-04-12
---

# Task 02 — vault/git.rs diff --stat 파싱 로직 추출 + 테스트

## Changed files

- `crates/secall-core/src/vault/git.rs` (전체 208줄)
  - 112~121줄: `pull()` 내부의 diff --stat 파싱 클로저 → 함수 호출로 교체
  - 207줄 이후: `count_new_session_files()` 함수 + `#[cfg(test)]` 모듈 추가

## Change description

### Step 1: 순수 함수 추출 (`impl VaultGit` 바깥, `PushResult` struct 뒤)

```rust
/// git diff --stat 출력에서 raw/sessions/ 경로가 포함된 라인 수를 카운트.
pub(crate) fn count_new_session_files(diff_stat_output: &str) -> usize {
    diff_stat_output
        .lines()
        .filter(|l| l.contains("raw/sessions/"))
        .count()
}
```

`impl VaultGit` 바깥에 배치하는 이유: `self` 불필요한 순수 함수. `pub(crate)` — 테스트 접근 가능, 외부 노출 없음.

### Step 2: `pull()` 내부 교체 (112~121줄)

```rust
// 기존 112-121줄 → 교체:
let new_files = if !already_up_to_date {
    self.run_git(&["diff", "--stat", "HEAD@{1}", "HEAD"])
        .ok()
        .map(|o| count_new_session_files(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or(0)
} else {
    0
};
```

### Step 3: `#[cfg(test)]` 모듈 추가 (파일 하단)

| 테스트명 | 입력 | 기대값 |
|----------|------|--------|
| `test_count_single_session` | `raw/sessions/abc.md \| 45 ++++` | 1 |
| `test_count_multiple_mixed` | session 2줄 + wiki 1줄 + summary | 2 |
| `test_count_no_sessions` | wiki + index만 | 0 |
| `test_count_empty` | 빈 문자열 | 0 |
| `test_count_summary_not_counted` | session 1줄 + `1 file changed` 요약줄 | 1 |

테스트 픽스처 예시:
```rust
#[test]
fn test_count_multiple_mixed() {
    let output = " raw/sessions/2026-04-01/abc.md | 45 ++++\n \
                   raw/sessions/2026-04-01/def.md | 12 ++\n \
                   wiki/projects/foo.md           |  8 +\n \
                   3 files changed, 65 insertions(+)";
    assert_eq!(count_new_session_files(output), 2);
}
```

## Dependencies

- 없음 (secall-core 내부, Task 01과 독립)

## Verification

```bash
cargo test -p secall-core vault::git
# 기대: 5 passed, 0 failed

cargo check -p secall-core
# 기대: exit 0
```

## Risks

- `pull()` 외부 동작 변화 없음 — 로직 이동 + 이름 부여만.
- `diff --stat` 출력 형식이 git 버전에 따라 달라질 수 있으나,
  `raw/sessions/` 문자열 포함 여부만 확인하므로 충분히 안정적.

## Scope boundary

수정 금지:
- `crates/secall-core/src/vault/index.rs` (Task 01 영역)
- `crates/secall-core/src/vault/mod.rs`
- `crates/secall/src/commands/` (Task 03 영역)

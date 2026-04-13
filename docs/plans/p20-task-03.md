---
type: plan-task
plan: p20
task: 03
title: commands/ingest.rs 세션 분류 로직 추출 + 테스트 + classify.rs 중복 제거
status: todo
updated_at: 2026-04-12
---

# Task 03 — commands/ingest.rs 세션 분류 로직 추출 + 테스트

## Changed files

- `crates/secall/src/commands/ingest.rs` (535줄)
  - 327줄 앞: `apply_classification()` 함수 추가
  - 351~372줄: 인라인 분류 블록 → 함수 호출로 교체
  - 파일 하단: `#[cfg(test)] mod tests` 추가
- `crates/secall/src/commands/classify.rs` (64줄)
  - 37~45줄: 중복 분류 로직 → `super::ingest::apply_classification()` 호출로 교체

## Change description

### Step 1: 순수 함수 추출 (`ingest_single_session()` 앞)

```rust
/// 컴파일된 regex 규칙과 첫 번째 user turn 내용으로 session_type 결정.
/// - rules 비어있으면 default 반환
/// - rules 순서대로 매칭, 첫 번째 매칭 적용
/// - 매칭 없으면 default 반환
pub(crate) fn apply_classification(
    compiled_rules: &[(regex::Regex, String)],
    first_user_content: &str,
    default_type: &str,
) -> String {
    if compiled_rules.is_empty() {
        return default_type.to_string();
    }
    compiled_rules
        .iter()
        .find_map(|(re, session_type)| {
            if re.is_match(first_user_content) {
                Some(session_type.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| default_type.to_string())
}
```

### Step 2: `ingest_single_session()` 내부 교체 (351~372줄)

```rust
// 기존 351-372줄 블록 전체 → 교체:
// 세션 분류
{
    let first_user_content = session
        .turns
        .iter()
        .find(|t| t.role == secall_core::ingest::Role::User)
        .map(|t| t.content.as_str())
        .unwrap_or("");
    session.session_type = apply_classification(
        compiled_rules,
        first_user_content,
        &config.ingest.classification.default,
    );
}
```

### Step 3: `classify.rs` 중복 제거 (37~45줄)

```rust
// classify.rs 기존 37-45줄 → 교체:
let new_type = super::ingest::apply_classification(
    &compiled_rules,
    first_content,
    &classification.default,
);
```

`classify.rs`의 `compiled_rules` 변수 타입 `Vec<(regex::Regex, String)>`은 `ingest.rs`와 동일하므로 호환성 문제 없음.

### Step 4: `#[cfg(test)]` 모듈 추가 (ingest.rs 하단)

| 테스트명 | 검증 내용 |
|----------|-----------|
| `test_matches_first_rule` | 첫 번째 규칙 매칭 → 해당 타입 반환 |
| `test_matches_second_rule` | 첫 번째 불일치, 두 번째 매칭 |
| `test_no_match_uses_default` | 모든 규칙 불일치 → default |
| `test_empty_rules_returns_default` | 빈 규칙 → default |
| `test_empty_content` | 빈 content → default |
| `test_first_match_wins` | 두 규칙 모두 매칭 → 첫 번째 우선 |

테스트 픽스처 헬퍼:
```rust
fn rules(patterns: &[(&str, &str)]) -> Vec<(regex::Regex, String)> {
    patterns.iter()
        .map(|(p, t)| (regex::Regex::new(p).unwrap(), t.to_string()))
        .collect()
}
```

## Dependencies

- `regex` crate: `crates/secall/Cargo.toml`에 `regex.workspace = true` 이미 존재 (확인 완료)
- Task 01, 02와 독립

## Verification

```bash
cargo test -p secall ingest
# 기대: 6 passed, 0 failed

cargo test -p secall classify
# 기대: classify.rs 컴파일 성공 (기존 테스트 없음, 컴파일만 확인)

cargo check --workspace
# 기대: exit 0
```

## Risks

- `classify.rs`에서 `super::ingest::apply_classification` 접근:
  `commands/mod.rs`에서 `pub(crate) mod ingest;`로 선언되어 있어야 함.
  현재 `pub(crate)` 또는 `pub` 여부를 확인하고, 필요 시 `mod.rs` 가시성 조정.
- `ingest_single_session()` 13개 인자 함수 자체는 건드리지 않음 — 내부 블록만 교체.

## Scope boundary

수정 금지:
- `crates/secall-core/src/vault/` (Task 01, 02 영역)
- `crates/secall-core/src/store/` (이미 완료)
- `crates/secall/src/commands/sync.rs` (Task 03 범위 아님)

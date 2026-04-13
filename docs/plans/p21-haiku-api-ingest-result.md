# Implementation Result: P21 — 시맨틱 엣지 고도화 (Haiku API + ingest 통합)

> Developer: claude
> Branch: N/A
> Date: 2026-04-13 14:49
> Plan Revision: 0

---

## Summary

Scope boundary에 `graph_repo.rs` 수정 금지가 있지만, 리뷰어가 명시적으로 `upsert_graph_edge()`의 반환값 변경을 요청했으므로 이를 따릅니다.

### Finding 1: `upsert_graph_edge` 반환값 변경 + `stored` 정확도

기존 호출자들이 `?`로 `Result<()>`를 기대하므로, 반환값이 `Result<usize>`로 바뀌어도 `?` 호출은 그대로 동작합니다. 다만 `build.rs`에서 `total_edges += 1` 패턴을 수정해야 합니다.

`build.rs`의 호출부에서 `?`로 `Result<usize>`를 받지만 반환값을 무시하므로 (이미 자체 `total_edges += 1` 카운트) 기존 동작에 영향 없습니다. 하지만 `semantic.rs`에서 실제 삽입 수를 사용하도록 수정합니다.

### Recommendation: 이중 호출 시 두 번째 반환값 0 검증 테스트

Verification 실행:

모든 수정 완료. Verification 결과:

```
Verification results for Task 01 (Rework 2):
✅ `cargo test -p secall-core graph::semantic` — 6 passed (including new test_extract_and_store_double_call_returns_zero)
✅ `cargo check --workspace` — exit 0
✅ `cargo fmt --all --check` — exit 0 (after fmt)
```

변경 사항 요약:

1. **`crates/secall-core/src/graph/semantic.rs`** — `test_extract_and_store_double_call_returns_zero` 테스트 추가. 동일 세션에 대해 두 번 호출 시 두 번째 반환값이 0인지 검증.

2. **`crates/secall/src/commands/ingest.rs:334`** — `.ok().flatten()` 제거, `match` 문으로 교체하여 DB 에러 시 `tracing::warn!` 로깅 후 `continue`.

## Subtask Results

### 1. ### Finding 1: `stored` 카운트 부풀림 수정

규칙+Haiku가 같은 (source, target, relation)을 생성할 때 중복 카운트되는 문제입니다. 저장 전에 dedup 처리합니다.

### Recommendation 1: task-01.md Changed files 갱신

### Recommendation 2: `extract_and_store` 레벨 폴백 테스트 보강

Verification을 다시 실행합니다.

```
Verification results for Task 01 (rework):
✅ `cargo test -p secall-core graph::semantic` — 5 passed (3→5, +2 신규)
✅ `cargo test -p secall-core` — 218 passed, 9 ignored
✅ `cargo test -p secall` — 13 passed
✅ `cargo check --workspace` — exit 0
✅ `R

### 2. Scope boundary에 `graph_repo.rs` 수정 금지가 있지만, 리뷰어가 명시적으로 `upsert_graph_edge()`의 반환값 변경을 요청했으므로 이를 따릅니다.

### Finding 1: `upsert_graph_edge` 반환값 변경 + `stored` 정확도

기존 호출자들이 `?`로 `Result<()>`를 기대하므로, 반환값이 `Result<usize>`로 바뀌어도 `?` 호출은 그대로 동작합니다. 다만 `build.rs`에서 `total_edges += 1` 패턴을 수정해야 합니다.

`build.rs`의 호출부에서 `?`로 `Result<usize>`를 받지만 반환값을 무시하므로 (이미 자체 `total_edges += 1` 카운트) 기존 동작에 영향 없습니다. 하지만 `semantic.rs`에서 실제 삽입 수를 사용하도록 수정합니다.

### Recommendation: 이중 호출 시 두 번째 반환값 0 검증 테스트

Verification 실행:


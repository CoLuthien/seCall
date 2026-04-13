# Review Report: P21 — 시맨틱 엣지 고도화 (Haiku API + ingest 통합) — Round 1

> Verdict: conditional
> Reviewer: 
> Date: 2026-04-13 13:30
> Plan Revision: 0

---

## Verdict

**conditional**

## Findings

1. crates/secall-core/src/graph/semantic.rs:219 — `extract_and_store()`가 `upsert_graph_edge()` 직후 `stored += 1`을 무조건 수행합니다. 그런데 `upsert_graph_edge()`는 `INSERT OR IGNORE`라 중복 엣지는 실제로 저장되지 않습니다. 규칙 기반과 Haiku 결과가 같은 엣지를 만들면 반환값이 “저장된 엣지 수”보다 크게 나오며, ingest 로그/후속 호출자가 잘못된 수치를 받게 됩니다.

## Recommendations

1. `docs/plans/p21-haiku-api-ingest-task-01.md`의 Changed files에 실제로 수정된 `crates/secall-core/src/store/db.rs`, `crates/secall/src/commands/sync.rs`를 반영해 task 계약과 구현 범위를 맞추세요.
2. `test_haiku_invalid_json_fallback`는 현재 parse 오류만 확인합니다. task 설명대로 `extract_and_store()` 레벨의 폴백 동작을 검증하는 테스트로 보강하는 편이 안전합니다.

## Subtask Verification

| # | Subtask | Status |
|---|---------|--------|
| 1 | Haiku API 클라이언트 + Config + ingest 통합 + CLI 플래그 | ✅ done |


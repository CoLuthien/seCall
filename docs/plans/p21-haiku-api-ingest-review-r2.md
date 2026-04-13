# Review Report: P21 — 시맨틱 엣지 고도화 (Haiku API + ingest 통합) — Round 2

> Verdict: conditional
> Reviewer: 
> Date: 2026-04-13 14:42
> Plan Revision: 0

---

## Verdict

**conditional**

## Findings

1. crates/secall-core/src/graph/semantic.rs:225 — `stored += 1`가 `INSERT OR IGNORE` 결과와 무관하게 실행됩니다. 메모리 dedup 이후에도 DB에 이미 같은 `(source, target, relation)` 엣지가 있으면 실제 insert는 0건인데 반환값은 1 증가해, `extract_and_store()`의 “저장된 엣지 수” 계약이 계속 깨집니다.

## Recommendations

1. `upsert_graph_edge()`가 `rusqlite::Connection::execute()`의 변경 행 수를 반환하도록 바꾸고, `extract_and_store()`는 그 값을 누적해 실제 insert 수만 집계하세요.
2. `extract_and_store()`를 동일 세션/동일 DB에 두 번 호출했을 때 두 번째 반환값이 0인지 검증하는 테스트를 추가하세요.
3. `crates/secall/src/commands/ingest.rs:334`의 `db.get_session_vault_path(session_id).ok().flatten()`는 DB 오류를 “경로 없음”으로 삼키므로, 최소한 warning 로깅 또는 에러 집계를 하도록 정리하는 편이 안전합니다.

## Subtask Verification

| # | Subtask | Status |
|---|---------|--------|
| 1 | Haiku API 클라이언트 + Config + ingest 통합 + CLI 플래그 | ✅ done |


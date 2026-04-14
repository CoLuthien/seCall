# Review Report: P22 Rev.2 — Wiki 파이프라인 잔여 결함 수정 — Round 1

> Verdict: fail
> Reviewer: 
> Date: 2026-04-14 10:21
> Plan Revision: 0

---

## Verdict

**fail**

## Findings

1. crates/secall/src/commands/wiki.rs:191 — 배치 모드 재생성 경로에서 `std::fs::write(&full_path, &linked2).ok()`로 저장 실패를 무시합니다. 쓰기 실패 시 파일은 갱신되지 않는데도 이후 재검수는 진행되어, 자동 수정 재시도가 성공한 것처럼 오판할 수 있습니다.
2. crates/secall/src/commands/wiki.rs:299 — 인크리멘탈 모드 재생성 경로에서도 동일하게 저장 실패를 무시합니다. 실제 결과 파일과 재검수 대상 내용이 불일치할 수 있어 논리 결함입니다.

## Recommendations

1. 재생성 경로의 `std::fs::write`는 `?` 또는 명시적 `match`로 처리해서 저장 실패 시 즉시 중단하거나 최소한 경고 후 재검수를 건너뛰도록 바꾸는 것이 맞습니다.

## Subtask Verification

| # | Subtask | Status |
|---|---------|--------|
| 1 | `insert_obsidian_links()` 위키 페이지 간 내부 링크 추가 | ✅ done |
| 2 | `build_haiku_incremental_prompt()` 기존 위키 페이지 목록 주입 | ✅ done |
| 3 | `run_review()` error급 이슈 자동 수정 재시도 | ✅ done |


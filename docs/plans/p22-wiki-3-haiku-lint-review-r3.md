# Review Report: P22 — Wiki 3단계 파이프라인 (Haiku 초안 + lint + 검수) — Round 3

> Verdict: fail
> Reviewer: 
> Date: 2026-04-14 00:05
> Plan Revision: 0

---

## Verdict

**fail**

## Findings

1. crates/secall-core/src/wiki/lint.rs:116 — `insert_obsidian_links()`가 세션 참조를 `[[<full-session-id>|<short-id>]]`로 치환하는데, 프로젝트의 링크 규약은 `[[raw/sessions/YYYY-MM-DD_session-id]]` 계열입니다. 현재 구현은 실제 vault 경로를 계산하지 않아 생성된 세션 링크가 Obsidian에서 해석되지 않는 깨진 링크가 됩니다.
2. crates/secall/src/commands/wiki.rs:595 — `--review` 경로가 `approved == false`이거나 `severity=error` 이슈가 있어도 결과를 출력만 하고 종료합니다. Task 03 계약은 error급 이슈에 대해 자동 수정 1회와 재검수를 요구하는데, 그 재시도 로직이 없어 검수 단계가 단순 리포트로만 동작합니다.

## Recommendations

1. `insert_obsidian_links()`는 DB의 session vault path 또는 날짜/파일명 규칙을 사용해 실제 note target을 만들고, 이미 일부 링크가 있는 경우에도 남은 bare session ID를 계속 치환하도록 테스트를 보강하세요.
2. `run_review()`는 error 이슈가 있을 때 수정 프롬프트를 재호출하고 1회만 재검수하는 흐름을 추가하세요. warning-only일 때 approve 처리도 명시적으로 분기하는 편이 안전합니다.
3. `docs/plans/p22-wiki-3-haiku-lint-result.md`에는 task별 Verification 명령과 결과를 다시 정리해 두는 것이 다음 재검토 때 혼선을 줄입니다.

## Subtask Verification

| # | Subtask | Status |
|---|---------|--------|
| 1 | HaikuBackend + 세션 데이터 전처리 | ✅ done |
| 2 | Lint + 후처리 파이프라인 | ✅ done |
| 3 | 검수 단계 + CLI 통합 | ✅ done |


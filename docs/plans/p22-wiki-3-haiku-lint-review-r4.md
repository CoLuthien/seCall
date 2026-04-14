# Review Report: P22 — Wiki 3단계 파이프라인 (Haiku 초안 + lint + 검수) — Round 4

> Verdict: fail
> Reviewer: 
> Date: 2026-04-14 00:11
> Plan Revision: 0

---

## Verdict

**fail**

## Findings

1. crates/secall-core/src/wiki/lint.rs:116 — 세션 링크를 `[[{full_uuid}|{short_id}]]`로 생성하지만, 실제 생성 경로는 `sessions/{short_id}.md` 또는 `projects/...`라서 링크 대상 노트가 존재하지 않습니다. 현재 구현대로면 세션 참조 링크가 Obsidian에서 깨집니다.
2. crates/secall-core/src/wiki/lint.rs:121 — 같은 세션 ID가 이미 한 번 링크돼 있으면 그 세션의 나머지 평문 참조까지 전부 건너뛰고, 그렇지 않은 경우에도 첫 번째 매치만 치환합니다. 한 문서에 같은 세션 ID가 여러 번 나오면 일부 참조가 평문으로 남습니다.
3. crates/secall/src/commands/wiki.rs:148 — `run_markdownlint()`가 파일을 수정할 수 있는데도 검수는 수정 전 문자열 `linked`를 그대로 `run_review()`에 넘깁니다. 결과적으로 `--review`는 최종 저장본이 아닌 pre-lint 초안을 검수하게 됩니다.

## Recommendations

1. `insert_obsidian_links()`는 실제 파일 경로 규칙과 맞는 링크 타깃을 사용하고, 이미 링크된 구간만 제외한 뒤 본문 전체의 모든 매치를 치환하도록 바꾸는 편이 안전합니다.
2. 검수 단계는 markdownlint 이후 파일을 다시 읽어서 최종 저장본 기준으로 실행하는 편이 계약과 동작이 일치합니다.

## Subtask Verification

| # | Subtask | Status |
|---|---------|--------|
| 1 | HaikuBackend + 세션 데이터 전처리 | ✅ done |
| 2 | Lint + 후처리 파이프라인 | ✅ done |
| 3 | 검수 단계 + CLI 통합 | ✅ done |


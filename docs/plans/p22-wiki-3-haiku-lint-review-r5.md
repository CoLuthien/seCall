# Review Report: P22 — Wiki 3단계 파이프라인 (Haiku 초안 + lint + 검수) — Round 5

> Verdict: fail
> Reviewer: 
> Date: 2026-04-14 06:45
> Plan Revision: 0

---

## Verdict

**fail**

## Findings

1. crates/secall/src/commands/wiki.rs:283 — `build_haiku_incremental_prompt()`는 세션 메타와 턴만 직렬화하고 끝나며, Task 01 계약에 있던 "기존 위키 페이지 목록도 함께 주입" 단계가 없습니다. 그 결과 `--session` 모드에서 기존 페이지 병합 힌트를 모델에 전달하지 못합니다.
2. crates/secall-core/src/wiki/lint.rs:113 — `insert_obsidian_links()`는 `session_ids`/`vault_paths`만 받아 세션 참조만 링크화합니다. Task 02 계약에 있던 "알려진 위키 페이지 제목이 본문에 나오면 `[[페이지명]]`으로 변환" 로직이 없어서 페이지 간 내부 링크 생성이 누락됩니다.

## Recommendations

1. 인크리멘탈 프롬프트 생성 시 `wiki/` 아래 기존 페이지 목록 또는 최소한 관련 프로젝트/토픽 페이지명을 함께 주입해 병합 힌트를 복구하세요.
2. `insert_obsidian_links()`에 `known_pages` 입력을 추가하거나 별도 후처리 단계를 두어, 세션 링크와 페이지 제목 링크를 분리해 처리하는 편이 안전합니다.

## Subtask Verification

| # | Subtask | Status |
|---|---------|--------|
| 1 | HaikuBackend + 세션 데이터 전처리 | ✅ done |
| 2 | Lint + 후처리 파이프라인 | ✅ done |
| 3 | 검수 단계 + CLI 통합 | ✅ done |


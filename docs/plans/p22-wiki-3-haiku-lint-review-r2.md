# Review Report: P22 — Wiki 3단계 파이프라인 (Haiku 초안 + lint + 검수) — Round 2

> Verdict: fail
> Reviewer: 
> Date: 2026-04-13 23:50
> Plan Revision: 0

---

## Verdict

**fail**

## Findings

1. crates/secall/src/commands/wiki.rs:117, crates/secall/src/commands/wiki.rs:415 — 배치 모드에서 생성 결과 전체에 대해 단일 `page_path`만 계산하고, `session == None`이면 항상 `index.md`로 기록합니다. Task 01/02 계약은 프로젝트별 별도 페이지 생성·병합인데, 현재 구현은 배치 출력의 다중 페이지를 분리하지 못해 `wiki/` 개별 파일 생성 요구를 충족하지 못합니다.
2. crates/secall/src/commands/wiki.rs:463 — 검수 단계가 모델에 전달하는 원본 근거가 `session: <id>` 또는 `batch update` 문자열뿐입니다. Task 03은 원본 세션 요약/내용을 대조용으로 전달해야 하는데, 현재 구현으로는 사실 정확성 검수가 실질적으로 동작할 수 없습니다.

## Recommendations

1. `--review-model`은 `Option<String>`로 받아 미지정일 때만 `config.wiki.review_model`을 쓰도록 바꾸면, 명시적 `--review-model sonnet`과 미지정을 구분할 수 있습니다.

## Subtask Verification

| # | Subtask | Status |
|---|---------|--------|
| 1 | HaikuBackend + 세션 데이터 전처리 | ✅ done |
| 2 | Lint + 후처리 파이프라인 | ✅ done |
| 3 | 검수 단계 + CLI 통합 | ✅ done |


# Review Report: P22 — Wiki 3단계 파이프라인 (Haiku 초안 + lint + 검수) — Round 1

> Verdict: fail
> Reviewer: 
> Date: 2026-04-13 23:38
> Plan Revision: 0

---

## Verdict

**fail**

## Findings

1. crates/secall/src/commands/wiki.rs:95 — 생성 결과를 `println!`으로만 출력하고 종료합니다. Task 02 계약상 필수인 `validate_frontmatter()`, `merge_with_existing()`, `insert_obsidian_links()`, 파일 쓰기, `run_markdownlint()` 호출이 전혀 없어 `wiki/` 파일 생성·병합이 수행되지 않습니다.
2. crates/secall/src/commands/wiki.rs:262 — Haiku 배치 프롬프트가 세션별 1줄 요약만 넣고 턴 본문을 전혀 주입하지 않습니다. Task 01은 배치 모드에서 각 세션의 턴 내용 일부(앞 3KB 수준)를 프롬프트에 포함하도록 요구하므로, 현재 구현은 원문 근거가 부족해 위키 초안 정확도가 계약 수준에 미달합니다.
3. crates/secall/src/main.rs:270 — `--review-model`에 기본값 `"sonnet"`을 박아 두고, crates/secall/src/commands/wiki.rs:106 에서 그 값을 그대로 사용합니다. Task 03에서 추가한 `config.wiki.review_model`은 실제로 읽히지 않아, CLI 미지정 시 config 값이 적용되어야 한다는 계약을 만족하지 못합니다.

## Recommendations

1. Task 02 완료 기준에 맞게 `run_update()`에서 생성 결과를 후처리 파이프라인으로 넘기고 실제 `wiki/` 경로에 쓰도록 연결한 뒤 재검토 요청하세요.
2. Task 03 재작업 시 `review_model` 해석 우선순위를 `CLI > config.wiki.review_model > "sonnet"`으로 바꾸고, 결과 문서에 Task 03 Verification 결과도 다시 남기세요.

## Subtask Verification

| # | Subtask | Status |
|---|---------|--------|
| 1 | HaikuBackend + 세션 데이터 전처리 | ✅ done |
| 2 | Lint + 후처리 파이프라인 | ✅ done |
| 3 | 검수 단계 + CLI 통합 | ✅ done |


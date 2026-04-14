---
type: plan
status: in_progress
updated_at: 2026-04-14
slug: p22-rev-2-wiki
---

# P22 Rev.2 — Wiki 파이프라인 잔여 결함 수정

## Background

P22 (Wiki 3단계 파이프라인) 5회 연속 리뷰 실패 후 설계 재검토.
Task 01(HaikuBackend), Task 02(lint 파이프라인), Task 03(CLI 통합)의 핵심 구조는
이미 구현 완료되어 있으며, 아래 3개 잔여 결함만 수정한다.

## 반복 실패 원인 분석

| 결함 | 최초 발견 | 상태 |
|------|----------|------|
| Obsidian 링크 타깃 불일치 | 3차 | ✅ 5차에서 해결 |
| 링크 치환 첫 매치만 | 4차 | ✅ 5차에서 해결 |
| 검수 pre-lint 내용 사용 | 4차 | ✅ 5차에서 해결 |
| 검수 error급 재시도 로직 없음 | 3차 | ❌ **미해결** → Rev.2 Task 03 |
| incremental 프롬프트에 기존 위키 목록 미주입 | 5차 | ❌ **미해결** → Rev.2 Task 02 |
| 위키 페이지 간 내부 링크 생성 누락 | 5차 | ❌ **미해결** → Rev.2 Task 01 |

## Subtasks

| # | 제목 | 파일 | 의존성 |
|---|------|------|--------|
| 01 | `insert_obsidian_links()` 위키 페이지 간 내부 링크 | lint.rs + wiki.rs | 없음 |
| 02 | `build_haiku_incremental_prompt()` 기존 위키 목록 주입 | wiki.rs | 없음 |
| 03 | `run_review()` error급 이슈 재시도 | wiki.rs | 없음 |

세 태스크는 서로 다른 함수를 수정하므로 독립적으로 진행 가능.

## 설계 원칙 (반복 실패 방지)

1. **1 task = 1 함수 변경**: 여러 함수를 동시에 바꾸지 않는다
2. **Verification = cargo test**: "Manual: 확인" 없이 모두 자동 검증
3. **Scope boundary 엄수**: 다른 task 파일 일절 수정 금지

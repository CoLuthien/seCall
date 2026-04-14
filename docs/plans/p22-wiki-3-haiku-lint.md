---
type: plan
status: draft
updated_at: 2026-04-13
---

# P22 — Wiki 3단계 파이프라인 (Haiku 초안 + lint + 검수)

## 배경

현재 위키 생성은 Claude Code subprocess가 MCP 도구(recall, get, wiki_search, Read/Write/Edit)로
자율 탐색하는 에이전트 방식. 비용이 높고($20-50/전체), 로컬 LLM으론 도구 접근이 불가능한 구조.

RT 테스트 결과 E4B(9B)는 내용 빈약 + 4096 컨텍스트 병목으로 위키 품질 부족 확인.

## 목표

CLI 주도 3단계 파이프라인으로 전환:

1. **Haiku 초안** — Rust가 세션 데이터를 DB에서 직접 추출 → 프롬프트에 주입 → Haiku API 호출
2. **Lint + 후처리** — frontmatter 교정, 기존 위키 병합, Obsidian 링크 생성, markdownlint
3. **선택적 검수** — `--review` 시 Sonnet/Opus가 초안 리뷰

## 비용 비교

| 방식 | 960세션 비용 | 시간 |
|------|------------|------|
| Claude Code (현재) | ~$20-50 | ~2-3시간 |
| Haiku 초안 only | ~$1 | ~30분 |
| Haiku + Sonnet 검수 10% | ~$2 | ~40분 |

## Subtasks

| # | 제목 | 파일 | 의존 |
|---|------|------|------|
| 01 | HaikuBackend + 세션 데이터 전처리 | p22-wiki-3-haiku-lint-task-01.md | 없음 |
| 02 | Lint + 후처리 파이프라인 | p22-wiki-3-haiku-lint-task-02.md | Task 01 |
| 03 | 검수 단계 + CLI 통합 | p22-wiki-3-haiku-lint-task-03.md | Task 01, 02 |

## 아키텍처

```
secall wiki update --backend haiku --since 2026-04-01
  │
  ├─ [Task 01] 전처리: DB 세션 조회 → 프로젝트별 그룹핑 → 청크 분할
  │   └─ Haiku API 호출 (messages endpoint) → 초안 마크다운
  │
  ├─ [Task 02] 후처리: frontmatter 교정 → 기존 위키 병합 → 링크 생성
  │   └─ markdownlint (optional)
  │
  └─ [Task 03] 검수 (--review): Sonnet/Opus 리뷰 → 수정 반영
```

## 하위 호환

기존 `claude`, `ollama`, `lmstudio` 백엔드는 그대로 유지.
`haiku`는 새 백엔드로 추가되며, config에서 `wiki.default_backend = "haiku"` 설정 가능.

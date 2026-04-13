---
type: plan
status: draft
updated_at: 2026-04-13
---

# P21 — 시맨틱 엣지 고도화 (Haiku API + ingest 통합)

## Description

기존 규칙 기반 시맨틱 엣지(fixes_bug, modifies_file)에 Haiku API 보강을 추가하고,
ingest 파이프라인에 직접 통합하여 `secall graph build` 없이도 시맨틱 엣지가 자동 생성되도록 한다.

## 현재 상태 (이미 구현됨)

- `extract_semantic_edges()` — `extract.rs:136-212` (규칙 기반 fixes_bug + modifies_file)
- `build.rs:178-199` — graph build 트랜잭션 내 시맨틱 엣지 자동 호출
- 테스트 7개 — extract.rs `#[cfg(test)]`
- confidence: `"INFERRED"` (규칙 기반)

## Expected Outcome

- `secall ingest` 시 시맨틱 엣지 자동 추출 (규칙 기반 + Haiku 옵션)
- `ANTHROPIC_API_KEY` 없어도 규칙 기반 폴백으로 동작
- `--no-semantic` 플래그로 추출 skip 가능
- `[graph] semantic = true/false` 설정으로 전역 on/off

## Subtasks

1. Haiku API 클라이언트 + Config + ingest 통합 + CLI 플래그 — `graph/semantic.rs`(신규), `graph/mod.rs`, `vault/config.rs`, `commands/ingest.rs`, `main.rs`

## 완료 조건

```bash
cargo test -p secall-core
cargo test -p secall
cargo check --workspace
```

## Non-goals

- graph_edges 스키마 변경 (valid_from/valid_to) — 시맨틱 엣지 가치 검증 후 별도 플랜
- recall 쿼리에 그래프 통합 — graph_query MCP 도구로 대체
- LLM 전용 엣지 타입 추가 (discusses_topic 등) — Haiku 정밀도 검증 후 별도 플랜

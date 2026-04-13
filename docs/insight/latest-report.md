# Insight Report — 2026-04-13 12:09

총 10건 발견 — 안정성 (Stability): 4건, 기술 부채 (Technical Debt): 1건, 테스트 (Testing): 5건 ($1.362, 0in/0out)

## debt

- ⬜ **프로젝트 상태 추적 섹션 미관리** [info] — CLAUDE.md

## stability

- ⬜ **post-ingest hook 실패 시 에러가 로그만 남기고 무시됨** [major] — crates/secall/src/commands/ingest.rs
- ⬜ **graph build 시 파일 읽기/파싱 실패를 continue로 조용히 건너뜀** [major] — crates/secall-core/src/graph/build.rs
- ⬜ **is_date_dir가 유효하지 않은 날짜를 통과시킴** [minor] — crates/secall-core/src/graph/build.rs
- ⬜ **이전 리뷰 지적사항 다수 해결됨 — regex 사전 컴파일, skip_embed_types, vector 필터** [info] — crates/secall/src/commands/classify.rs

## test

- ⬜ **신규 log.rs 커맨드에 테스트 모듈 없음 (200 LOC)** [major] — crates/secall/src/commands/log.rs
- ⬜ **classify.rs 커맨드에 테스트 모듈 없음** [minor] — crates/secall/src/commands/classify.rs
- ⬜ **graph.rs 커맨드에 테스트 모듈 없음 (156 LOC)** [minor] — crates/secall/src/commands/graph.rs
- ⬜ **DB 메서드 get_sessions_for_date, get_topics_for_sessions 테스트 부재** [minor] — crates/secall-core/src/store/db.rs
- ⬜ **session_repo.rs trait에 신규 메서드 미반영** [info] — crates/secall-core/src/store/session_repo.rs


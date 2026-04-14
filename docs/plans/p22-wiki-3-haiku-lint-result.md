# Implementation Result: P22 — Wiki 3단계 파이프라인 (Haiku 초안 + lint + 검수)

> Developer: claude
> Branch: N/A
> Date: 2026-04-14 06:42
> Plan Revision: 0

---

## Summary

3건의 수정을 정확히 구현하겠습니다. 먼저 현재 코드 상태를 확인합니다.

이제 vault 경로 규칙을 확인합니다 — 실제 세션 파일 경로가 어떤 패턴인지.

이제 vault 경로 규칙을 정확히 파악했습니다:
- 세션 파일: `raw/sessions/{YYYY-MM-DD}/{agent}_{project}_{id_prefix}.md`
- 예: `raw/sessions/2026-04-13/claude-code_seCall_86b9d1fa.md`

DB의 `vault_path` 컬럼에 이 상대 경로가 저장되어 있을 것입니다. 이를 확인하겠습니다.

완벽합니다. `db.get_session_vault_path(session_id)`로 실제 vault 경로를 알 수 있습니다. 이제 Task 03의 리뷰어가 지적한 3건을 정확히 수정합니다.

**수정 사항 정리:**

1. **lint.rs `insert_obsidian_links()`**: 
   - 링크 타깃을 `[[{full_uuid}|{short}]]` → `[[raw/sessions/...vault_path...|{short}]]`로 변경 (실제 vault 경로 기반)
   - vault_path를 알려면 DB 접근이 필요 → 함수 시그니처에 vault_paths 매핑 추가
   - 이미 링크된 구간만 제외하고 **모든 매치**를 치환

2. **lint.rs `insert_obsidian_links()` 중복 치환**: 첫 번째 매치만이 아니라 본문 전체의 모든 매치를 치환

3. **wiki.rs**: markdownlint 후 파일을 다시 읽어서 검수에 전달

먼저 lint.rs의 `insert_obsidian_links()` 시그니처를 변경하겠습니다. vault_path 정보는 호출자(wiki.rs)에서 DB를 통해 조회하여 `HashMap<session_id, vault_path>`로 전달합니다.

이제 테스트를 새 시그니처에 맞게 수정합니다.

이제 wiki.rs의 호출부를 수정합니다. `insert_obsidian_links()`에 vault_paths를 전달하고, markdownlint 후 파일을 다시 읽어서 review에 전달합니다.

배치 모드 블록을 수정합니다 (vault_paths 조회 + insert_obsidian_links 시그니처 + review에 post-lint 파일 전달):

이제 인크리멘탈 모드도 동일하게 수정합니다.

이제 `collect_vault_paths()` 헬퍼 함수를 추가합니다.

빌드 확인합니다.

경고 없이 빌드 성공. 테스트를 실행합니다.

29 passed (기존 27 + 새 테스트 2). CLI 확인:

최종 변경 파일 상태를 확인합니다.

모든 수정이 올바르게 적용되었습니다.

---

## Verification Results

```
✅ cargo build -p secall-core -p secall — 경고 없이 빌드 성공
✅ cargo test -p secall-core wiki — 29 passed, 0 failed
✅ cargo run -- wiki update --help | grep review — --review, --review-model 정상 표시
```

## 수정 내역 (3건)

### Fix #1 — `insert_obsidian_links()` 링크 타깃을 실제 vault 경로 기반으로 변경

**파일**: `crates/secall-core/src/wiki/lint.rs:110-165`

- 시그니처에 `vault_paths: &HashMap<String, String>` 추가
- `db.get_session_vault_path(sid)`로 조회한 실제 경로 (예: `raw/sessions/2026-04-13/claude-code_seCall_86b9d1fa.md`)에서 `.md`를 제거하여 Obsidian 링크 타깃으로 사용
- vault 경로가 없는 세션은 full UUID fallback
- 호출부(`wiki.rs`)에 `collect_vault_paths()` 헬퍼

## Subtask Results

### 1. 3건의 수정을 정확히 구현하겠습니다. 먼저 현재 코드 상태를 확인합니다.

이제 vault 경로 규칙을 확인합니다 — 실제 세션 파일 경로가 어떤 패턴인지.

이제 vault 경로 규칙을 정확히 파악했습니다:
- 세션 파일: `raw/sessions/{YYYY-MM-DD}/{agent}_{project}_{id_prefix}.md`
- 예: `raw/sessions/2026-04-13/claude-code_seCall_86b9d1fa.md`

DB의 `vault_path` 컬럼에 이 상대 경로가 저장되어 있을 것입니다. 이를 확인하겠습니다.

완벽합니다. `db.get_session_vault_path(session_id)`로 실제 vault 경로를 알 수 있습니다. 이제 Task 03의 리뷰어가 지적한 3건을 정확히 수정합니다.

**수정 사항 정리:**

1. **lint.rs `insert_obsidian_links()`**: 
   - 링크 타깃을 `[[{full_uuid


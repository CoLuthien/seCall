---
type: task
plan: p22-rev-2-wiki
task_number: 3
status: draft
updated_at: 2026-04-14
---

# Task 03 — `run_review()` error급 이슈 자동 수정 재시도

## Changed files

| 파일 | 위치 | 변경 종류 |
|------|------|----------|
| `crates/secall/src/commands/wiki.rs:607-639` | `run_review()` | 반환 타입 `()` → `bool` 변경, error 이슈 존재 시 `true` 반환 |
| `crates/secall/src/commands/wiki.rs:151-157` | 배치 모드 review 호출부 | `run_review()` 반환값 확인 후 재생성 1회 |
| `crates/secall/src/commands/wiki.rs:216-223` | 인크리멘탈 모드 review 호출부 | 동일 |

## Change description

### 1. `wiki.rs:607` — `run_review()` 반환 타입 변경

```rust
// 변경 전
async fn run_review(model: &str, page_content: &str, source_summary: &str) {

// 변경 후
/// error급 이슈가 있으면 true(재생성 필요), 없거나 API 실패 시 false 반환
async fn run_review(model: &str, page_content: &str, source_summary: &str) -> bool {
```

내부 로직 변경:

```rust
async fn run_review(model: &str, page_content: &str, source_summary: &str) -> bool {
    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("  ⚠ Review skipped: ANTHROPIC_API_KEY not set");
            return false;  // 실패 → 재시도 불필요
        }
    };

    eprintln!("  Reviewing with {}...", model);
    match secall_core::wiki::review::review_page(
        &api_key, model, page_content, source_summary,
    )
    .await
    {
        Ok(result) => {
            if result.approved {
                eprintln!("  ✓ Review: approved");
                false  // 승인 → 재시도 불필요
            } else {
                let error_count = result.issues.iter()
                    .filter(|i| i.severity == "error")
                    .count();
                eprintln!("  ⚠ Review: {} issue(s) found ({} error)", 
                    result.issues.len(), error_count);
                for issue in &result.issues {
                    eprintln!("    [{}] {}", issue.severity, issue.description);
                    if let Some(ref sug) = issue.suggestion {
                        eprintln!("      → {}", sug);
                    }
                }
                error_count > 0  // error가 있으면 true(재생성 필요)
            }
        }
        Err(e) => {
            eprintln!("  ⚠ Review failed (skipped): {}", e);
            false  // API 실패 → 재시도 불필요
        }
    }
}
```

### 2. `wiki.rs:151-157` — 배치 모드 호출부: 재시도 로직 추가

현재:
```rust
if review {
    let final_content = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|_| linked.clone());
    let source_summary = build_review_source(&db, &session_ids);
    run_review(&resolved_model, &final_content, &source_summary).await;
}
```

변경 후:
```rust
if review {
    let final_content = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|_| linked.clone());
    let source_summary = build_review_source(&db, &session_ids);
    let needs_regen = run_review(&resolved_model, &final_content, &source_summary).await;

    // error급 이슈 → 1회 재생성 후 재검수 (무한 루프 방지: 최대 1회)
    if needs_regen {
        eprintln!("    Regenerating due to review errors...");
        match backend_box.generate(&proj_prompt).await {
            Ok(regen_output) if !regen_output.trim().is_empty() => {
                let validated2 = secall_core::wiki::lint::validate_frontmatter(
                    &regen_output, &session_ids,
                );
                let merged2 = secall_core::wiki::lint::merge_with_existing(
                    &wiki_dir, &page_path, &validated2, &session_ids,
                ).unwrap_or(validated2);
                let wiki_pages2 = collect_wiki_pages(&wiki_dir);
                let linked2 = secall_core::wiki::lint::insert_obsidian_links(
                    &merged2, &session_ids, &vault_paths, &wiki_pages2,
                );
                std::fs::write(&full_path, &linked2).ok();
                let final2 = std::fs::read_to_string(&full_path)
                    .unwrap_or(linked2);
                // 재검수 (반환값 무시 — 재시도는 1회만)
                run_review(&resolved_model, &final2, &source_summary).await;
            }
            _ => eprintln!("    Regeneration skipped (empty output)"),
        }
    }
}
```

### 3. `wiki.rs:216-223` — 인크리멘탈 모드 호출부: 동일 패턴 적용

배치 모드와 동일하게 `run_review()` 반환값 확인 후, `true`이면 `backend_box.generate(&prompt)` 1회 재호출 → 후처리 재실행 → 재검수.

**주의**: 인크리멘탈 모드는 `proj_prompt`가 없고 `prompt`를 사용. 재생성 시 `prompt` 변수 재사용.

## Dependencies

- Task 01 완료 필요: 재시도 후처리 파이프라인에서 `insert_obsidian_links()` 호출 시 `wiki_pages` 인자가 필요 (Task 01에서 추가되는 시그니처)
- Task 02와는 독립적

## Verification

```bash
# 1. 빌드
cargo build -p secall -p secall-core

# 2. run_review가 bool을 반환하는지 컴파일로 확인
# (빌드 성공이면 타입 검증 완료)

# 3. 코드 구조 확인: run_review 반환값이 호출부에서 사용되는지
grep -n "needs_regen\|run_review" \
    crates/secall/src/commands/wiki.rs
# 출력에 "needs_regen" 변수와 두 호출부(배치/인크리멘탈)가 모두 나와야 함

# 4. 기존 테스트 회귀 없음
cargo test -p secall-core wiki
cargo test -p secall-core wiki::review
```

## Risks

- **재시도 비용**: Haiku 재호출 1회 + Sonnet/Opus 재검수 1회 추가 발생. `--review` 사용 시 사용자가 인지해야 함.
- **재생성 시 merge 중복**: `merge_with_existing()`이 같은 세션 ID를 이미 포함한 페이지를 skip할 수 있음. `session_ids`가 이미 기록된 경우 재생성 후처리에서 기존 내용이 유지됨 — 의도된 동작.
- **proj_prompt 스코프**: 배치 모드의 재시도에서 `proj_prompt`가 루프 내 변수라 접근 가능. 인크리멘탈 모드는 `prompt` 변수(함수 상단에서 생성)를 재사용.

## Scope boundary

**수정 금지:**
- `crates/secall-core/src/wiki/lint.rs` (Task 01 영역)
- `crates/secall/src/commands/wiki.rs:268-326` (Task 02 영역)
- `crates/secall-core/src/wiki/review.rs` — `review_page()` 자체는 수정 없음
- `crates/secall-core/src/wiki/haiku.rs`
- `crates/secall/src/main.rs`
- `crates/secall/src/commands/sync.rs`

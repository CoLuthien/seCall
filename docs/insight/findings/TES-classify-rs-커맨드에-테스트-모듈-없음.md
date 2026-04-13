# classify.rs 커맨드에 테스트 모듈 없음

- **Category**: test
- **Severity**: minor
- **Fix Difficulty**: guided
- **Status**: open
- **File**: crates/secall/src/commands/classify.rs:25

## Description

classify.rs의 `run_backfill` 함수는 `ingest.rs`의 `CompiledRule`과 `apply_classification`을 재사용하므로 핵심 로직은 `ingest.rs` 테스트에서 커버됩니다. 그러나 backfill 특유의 흐름(전체 세션 순회, dry_run 분기)은 테스트되지 않습니다. ingest.rs에 이미 10개의 분류 테스트가 있어 severity는 minor로 판단합니다.

**Evidence**: `let compiled_rules: Vec<super::ingest::CompiledRule> = classification
    .rules
    .iter()
    .map(|rule| { ... })
    .collect::<anyhow::Result<_>>()?;`

## Snippet

```
pub async fn run_backfill(dry_run: bool) -> Result<()> { ... }
```

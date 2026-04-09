---
type: task
status: draft
updated_at: 2026-04-09
plan: secall-p13-windows
task: 2
title: 네이티브 의존성 컴파일 이슈 수정
depends_on: [1]
parallel_group: null
---

# Task 2: 네이티브 의존성 컴파일 이슈 수정

## Changed files

> Task 1의 CI 결과에 따라 범위가 확정됨. 아래는 예상 후보 — 실제 깨지는 것만 수정한다.

- `Cargo.toml` (workspace dependencies) — 의존성 피처 변경
- `crates/secall-core/Cargo.toml` — 조건부 의존성 추가
- `crates/secall-core/src/search/tokenizer.rs` — kiwi-rs 조건부 컴파일
- `crates/secall-core/src/search/ann.rs` — usearch 조건부 컴파일 (필요 시)
- `crates/secall-core/src/search/embedding.rs` — ORT 초기화 (필요 시)

## Change description

### 원칙

1. **크로스플랫폼 통일 우선** — 가능하면 Windows에서도 동일 코드로 빌드
2. **`#[cfg]` 분기는 최소화** — 꼭 필요한 경우만 플랫폼 분기
3. **기존 macOS/Linux 동작 변경 금지**

### 예상 수정 후보별 접근법

#### A. tokenizers `onig` 피처 (컴파일 에러 가능)

**현재**: `Cargo.toml:40` — `tokenizers = { version = "0.21", default-features = false, features = ["onig"] }`

**문제**: `onig` 피처는 Oniguruma C 라이브러리를 빌드. MSVC에서 실패 가능.

**수정 방안**:
- `onig`을 전 플랫폼에서 제거하고 순수 Rust `unicode` 피처로 전환
- `tokenizers = { version = "0.21", default-features = false, features = ["unicode"] }`
- `onig`과 `unicode`는 정규식 엔진 차이일 뿐 — BGE-M3 토크나이저 동작에 영향 없음 (BPE 기반)
- **모든 플랫폼 동일** — `#[cfg]` 분기 불필요

**검증**: 기존 166개 테스트 통과 확인 + 임베딩 결과 동일성 확인

#### B. kiwi-rs (컴파일 에러 가능)

**현재**: `Cargo.toml:41` — `kiwi-rs = "0.1"` / `tokenizer.rs:72` — `KiwiWrapper(kiwi_rs::Kiwi)`

**문제**: kiwi-rs는 C++ 코드를 래핑. MSVC에서 빌드 실패 가능.

**수정 방안 (깨질 경우)**:
```toml
# Cargo.toml (workspace)
[target.'cfg(not(target_os = "windows"))'.dependencies]
kiwi-rs = "0.1"
```
```rust
// tokenizer.rs
#[cfg(not(target_os = "windows"))]
mod kiwi_impl { ... }

// create_tokenizer()에서 Windows면 lindera 직접 사용
```

**영향 범위**: Windows에서 kiwi 토크나이저 사용 불가 → lindera fallback. macOS/Linux는 변경 없음. 현재도 kiwi 실패 시 lindera로 fallback하는 로직 존재 (`tokenizer.rs:152-154`).

#### C. usearch (컴파일 에러 가능)

**현재**: `Cargo.toml:45` — `usearch = "2"` / `ann.rs` — `usearch::Index`

**문제**: usearch는 C++ HNSW 구현. MSVC 빌드 미확인.

**수정 방안 (깨질 경우)**:
- usearch crate는 `cc` crate로 C++ 빌드 — MSVC에서 대체로 동작
- 빌드 실패 시 `#[cfg(not(target_os = "windows"))]`로 ANN 비활성화
- Windows에서는 BLOB 코사인 스캔 fallback (이미 `vector.rs:212-221`에 존재)

#### D. lindera embed-ko-dic

**현재**: `Cargo.toml:30` — `lindera = { version = "2.3.4", features = ["embed-ko-dic"] }`

**예상**: MSVC에서 정상 빌드 가능 (순수 Rust + 바이너리 사전 임베딩). 바이너리 크기만 20-40MB 증가.

#### E. ort (load-dynamic)

**현재**: `Cargo.toml:38` — `ort = { version = "=2.0.0-rc.10", features = ["load-dynamic"] }`

**예상**: 컴파일은 성공 — `load-dynamic`은 런타임에 DLL을 로드하므로 빌드 시점에 DLL 불필요. 런타임 DLL 번들링은 Task 3에서 처리.

### 수정 순서

1. tokenizers `onig` → `unicode` 전환 (전 플랫폼 동일)
2. kiwi-rs 조건부 컴파일 (CI 결과에 따라)
3. usearch 조건부 컴파일 (CI 결과에 따라)
4. 기존 테스트 전체 통과 확인

## Dependencies

- **Task 1 완료 필수** — CI 결과에서 실제 실패 목록을 확인한 후 수정 범위 확정
- 패키지: `tokenizers` unicode 피처 호환성 확인

## Verification

```bash
# 1. macOS/Linux에서 기존 테스트 통과 (regression 없음)
cargo test --all

# 2. clippy 경고 없음
cargo clippy --all-targets --all-features

# 3. fmt 통과
cargo fmt --all -- --check

# 4. Windows CI 통과 확인 (push 후)
gh run list --workflow=ci.yml --limit 1 --json status,conclusion

# 5. Windows job 로그에서 컴파일+테스트 성공 확인
gh run view --log | grep -E "(PASS|FAIL|error\[E)"
```

## Risks

- **tokenizers `unicode` 피처 동작 차이**: `onig`→`unicode` 전환 시 토크나이저 정규식 엔진이 바뀜. BGE-M3는 BPE 기반이라 영향 없을 것으로 예상되나, 임베딩 결과가 미세하게 달라질 가능성 있음. 기존 벡터 DB와 호환 확인 필요.
- **kiwi-rs `#[cfg]` 분기**: Windows에서 한국어 토크나이징 품질 저하 (lindera fallback). 기능적 차이는 있지만 동작은 함.
- **usearch 빌드**: MSVC에서 C++ 컴파일이 성공할 수도 있음 — 무조건 `#[cfg]` 분기하지 말고 CI 결과 확인 후 결정.

## Scope boundary

수정 금지:
- `.github/workflows/ci.yml` (Task 1 영역)
- `.github/workflows/release.yml` (Task 3 영역)
- `crates/secall-core/src/ingest/**` (파서 코드)
- `crates/secall-core/src/vault/**` (볼트 코드)
- `crates/secall-core/src/store/db.rs` (DB 스키마)
- `crates/secall/src/commands/**` (CLI 커맨드)

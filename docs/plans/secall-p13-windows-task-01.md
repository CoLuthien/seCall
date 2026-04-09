---
type: task
status: draft
updated_at: 2026-04-09
plan: secall-p13-windows
task: 1
title: CI에 Windows 빌드 추가 + 깨지는 것 확인
depends_on: []
parallel_group: null
---

# Task 1: CI에 Windows 빌드 추가 + 깨지는 것 확인

## Changed files

- `.github/workflows/ci.yml` (전체 수정 — 매트릭스 구조 변경)

## Change description

### 목표
CI에 `windows-latest` 러너를 추가하여 현재 코드가 MSVC 툴체인에서 컴파일되는지 확인한다. **이 태스크의 목적은 "무엇이 깨지는지 파악"이다.**

### 구현 단계

1. **ci.yml에 OS 매트릭스 추가**
   - 현재: `runs-on: ubuntu-latest` 단일 러너 (line 17)
   - 변경: `strategy.matrix.os: [ubuntu-latest, windows-latest]`
   - `runs-on: ${{ matrix.os }}`로 변경

2. **Windows용 step 분기 추가**
   - `cargo fmt --check`: Windows에서도 동일하게 실행
   - `cargo clippy`: 동일
   - `cargo test`: 동일
   - `cargo audit`: `continue-on-error: true` 유지

3. **캐시 키에 OS 포함**
   - 현재: `key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}` (line 33)
   - 이미 `runner.os`를 사용하므로 Windows/Linux 캐시가 자동 분리됨

4. **CI를 push하고 결과 관찰**
   - 컴파일 성공/실패 여부 확인
   - 실패하는 의존성과 에러 메시지 기록 → Task 2 입력

### 예상 ci.yml 구조

```yaml
jobs:
  check:
    name: Check & Test
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: ${{ runner.os }}-cargo-
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets --all-features
      - run: cargo test --all
      - name: cargo audit
        run: |
          cargo install cargo-audit --locked || true
          cargo audit
        continue-on-error: true
```

### 예상되는 실패 후보

| 의존성 | 실패 유형 | 원인 |
|---|---|---|
| `tokenizers` (onig 피처) | 컴파일 에러 | Oniguruma C 라이브러리 MSVC 빌드 |
| `kiwi-rs` | 컴파일 에러 | C++ 코드 래핑, MSVC 호환 미확인 |
| `usearch` | 컴파일 에러 | C++ HNSW 구현 |
| `lindera` (embed-ko-dic) | 바이너리 크기 경고 | 20-40MB 사전 임베딩 |
| `ort` (load-dynamic) | 컴파일은 성공, 런타임 실패 | DLL 부재 |

## Dependencies

- 없음 (첫 번째 태스크)

## Verification

```bash
# 1. ci.yml 문법 검증 (로컬)
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))" && echo "YAML OK"

# 2. GitHub Actions 실행 확인 (push 후)
gh run list --workflow=ci.yml --limit 1 --json status,conclusion

# 3. Windows job 결과 확인
gh run view --log-failed
```

> **핵심**: Task 1의 성공 기준은 "Windows CI가 통과하는 것"이 아니라 "Windows CI가 실행되어 깨지는 부분이 식별되는 것"이다. CI 실패 자체는 예상된 결과.

## Risks

- **Windows CI 시간**: `windows-latest` 러너는 Ubuntu보다 느림 (캐시 미적중 시 10-15분)
- **CI 비용**: 매트릭스 추가로 GitHub Actions 분당 비용 증가 (public repo면 무료)
- **Cargo cache 경로**: Windows에서 `~/.cargo/`는 `C:\Users\runneradmin\.cargo\`로 매핑됨 — `actions/cache@v4`가 자동 처리

## Scope boundary

수정 금지:
- `release.yml` (Task 3 영역)
- `Cargo.toml`, `crates/*/Cargo.toml` (Task 2 영역)
- `crates/secall-core/src/**` (Task 2 영역)

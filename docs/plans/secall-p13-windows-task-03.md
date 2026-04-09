---
type: task
status: draft
updated_at: 2026-04-09
plan: secall-p13-windows
task: 3
title: Release 워크플로우에 Windows 바이너리 추가
depends_on: [2]
parallel_group: null
---

# Task 3: Release 워크플로우에 Windows 바이너리 추가

## Changed files

- `.github/workflows/release.yml` (매트릭스 + Windows 빌드 step 추가)

## Change description

### 목표
`release.yml`에 `x86_64-pc-windows-msvc` 타겟을 추가하여 GitHub Release에 Windows 바이너리(secall.exe + onnxruntime.dll)를 포함한다.

### 구현 단계

#### 1. 빌드 매트릭스에 Windows 타겟 추가

현재 `release.yml:12-18`:
```yaml
strategy:
  matrix:
    include:
      - target: aarch64-apple-darwin
        os: macos-latest
      - target: x86_64-apple-darwin
        os: macos-14
```

변경:
```yaml
strategy:
  matrix:
    include:
      - target: aarch64-apple-darwin
        os: macos-latest
      - target: x86_64-apple-darwin
        os: macos-14
      - target: x86_64-pc-windows-msvc
        os: windows-latest
```

#### 2. Windows 빌드 step

macOS와 동일하게 `cargo build --release --target x86_64-pc-windows-msvc -p secall` 실행.

#### 3. ORT DLL 번들링 (Windows 전용)

ORT `load-dynamic` 피처는 런타임에 `onnxruntime.dll`을 찾는다. Windows 사용자가 별도 설치 없이 사용하려면 DLL을 ZIP에 번들링해야 한다.

```yaml
- name: Bundle ORT DLL (Windows)
  if: contains(matrix.target, 'windows')
  shell: pwsh
  run: |
    $ORT_VERSION = "1.19.2"
    $URL = "https://github.com/microsoft/onnxruntime/releases/download/v${ORT_VERSION}/onnxruntime-win-x64-${ORT_VERSION}.zip"
    Invoke-WebRequest -Uri $URL -OutFile ort.zip
    Expand-Archive ort.zip -DestinationPath ort
    Copy-Item "ort/onnxruntime-win-x64-${ORT_VERSION}/lib/onnxruntime.dll" `
      "target/${{ matrix.target }}/release/"
```

> **ORT 버전**: `ort = "2.0.0-rc.10"`이 사용하는 ONNX Runtime 버전 확인 필요. Cargo.lock에서 `ort-sys` 버전으로 매핑.

#### 4. 패키징 형식 분기

macOS: `.tar.gz` (현재 유지)
Windows: `.zip` (Windows 사용자 표준)

```yaml
- name: Package (Unix)
  if: "!contains(matrix.target, 'windows')"
  run: |
    cd target/${{ matrix.target }}/release
    tar czf secall-${{ matrix.target }}.tar.gz secall
    mv secall-${{ matrix.target }}.tar.gz ${{ github.workspace }}/

- name: Package (Windows)
  if: contains(matrix.target, 'windows')
  shell: pwsh
  run: |
    cd target/${{ matrix.target }}/release
    Compress-Archive -Path secall.exe, onnxruntime.dll `
      -DestinationPath "${{ github.workspace }}/secall-${{ matrix.target }}.zip"
```

#### 5. Release upload에 Windows 아티팩트 추가

현재 `release.yml:44-47`:
```yaml
files: |
  secall-aarch64-apple-darwin/secall-aarch64-apple-darwin.tar.gz
  secall-x86_64-apple-darwin/secall-x86_64-apple-darwin.tar.gz
```

변경:
```yaml
files: |
  secall-aarch64-apple-darwin/secall-aarch64-apple-darwin.tar.gz
  secall-x86_64-apple-darwin/secall-x86_64-apple-darwin.tar.gz
  secall-x86_64-pc-windows-msvc/secall-x86_64-pc-windows-msvc.zip
```

### ORT DLL 버전 매핑

`ort = "2.0.0-rc.10"`의 ONNX Runtime 호환 버전을 확인하는 방법:
```bash
grep -A 2 'name = "ort-sys"' Cargo.lock
```
`ort-sys` 버전이 ONNX Runtime 릴리스 버전과 매핑된다. 다운로드 URL의 `ORT_VERSION`을 이에 맞춰야 한다.

## Dependencies

- **Task 2 완료 필수** — Windows에서 `cargo build` 성공해야 Release 바이너리 생성 가능
- **ORT GitHub Release** — onnxruntime-win-x64 ZIP 다운로드 가능해야 함

## Verification

```bash
# 1. release.yml YAML 문법 검증
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))" && echo "YAML OK"

# 2. 테스트 태그 push로 Release 워크플로우 트리거
git tag v0.2.4-rc.1 && git push origin v0.2.4-rc.1

# 3. Release 워크플로우 실행 확인
gh run list --workflow=release.yml --limit 1 --json status,conclusion

# 4. 생성된 Release 아티팩트 확인
gh release view v0.2.4-rc.1 --json assets --jq '.assets[].name'
# 기대 출력:
#   secall-aarch64-apple-darwin.tar.gz
#   secall-x86_64-apple-darwin.tar.gz
#   secall-x86_64-pc-windows-msvc.zip

# 5. Windows ZIP 내용 확인 (다운로드 후)
# Manual: ZIP 안에 secall.exe + onnxruntime.dll 존재 확인

# 6. 테스트 태그 정리
gh release delete v0.2.4-rc.1 --yes && git push origin --delete v0.2.4-rc.1 && git tag -d v0.2.4-rc.1
```

## Risks

- **ORT DLL 버전 불일치**: `ort` crate가 기대하는 ONNX Runtime 버전과 다운로드한 DLL 버전이 다르면 런타임 에러. `ort-sys` 버전으로 정확히 매핑해야 함.
- **ORT DLL 크기**: `onnxruntime.dll`은 ~100-150MB. ZIP 파일이 커짐. 사용자 다운로드 시간 증가.
- **Windows 바이너리 서명**: 서명 없이 배포하면 Windows Defender SmartScreen 경고. 현 단계에서는 무시 (code signing은 별도 작업).
- **GitHub Actions 윈도우 빌드 시간**: macOS보다 2-3배 느림. Release 전체 시간 증가.

## Scope boundary

수정 금지:
- `.github/workflows/ci.yml` (Task 1 영역)
- `Cargo.toml`, `crates/*/Cargo.toml` (Task 2 영역)
- `crates/secall-core/src/**` (소스 코드)
- `crates/secall/src/**` (CLI 코드)
- `README.md` (별도 후속 작업)

---
type: plan
status: draft
updated_at: 2026-04-09
plan: secall-p13-windows
version: 1
---

# seCall P13 — Windows 빌드 지원

## Description

`x86_64-pc-windows-msvc` 타겟을 CI/CD에 추가하여 Windows에서 seCall을 빌드·배포한다. 네이티브 C/C++ 의존성(ort, tokenizers+onig, kiwi-rs, usearch, lindera)의 MSVC 컴파일 이슈를 해결하고, ORT DLL을 번들링하여 Windows 바이너리를 GitHub Release에 포함한다.

## Background

- 순수 Rust 코드(파싱, DB, vault, 검색 로직)는 이미 Windows 호환 — 경로 처리에 `\\` 분기문 존재
- 문제는 네이티브 의존성의 MSVC 빌드 + ORT 런타임 DLL
- P8에서 Windows를 명시적으로 non-goal로 제외했으나, 사용자 요청으로 지원 추가

## Expected Outcome

- `cargo build --release --target x86_64-pc-windows-msvc` CI에서 성공
- `cargo test` Windows에서 전체 통과
- GitHub Release에 `secall-x86_64-pc-windows-msvc.zip` (secall.exe + onnxruntime.dll) 포함
- 기존 macOS 빌드·테스트에 regression 없음

## Subtasks

| # | 제목 | depends_on | parallel_group |
|---|------|------------|----------------|
| 1 | CI에 Windows 빌드 추가 + 깨지는 것 확인 | — | — |
| 2 | 네이티브 의존성 컴파일 이슈 수정 | 1 | — |
| 3 | Release 워크플로우에 Windows 바이너리 추가 | 2 | — |

## Constraints

- Task 2는 Task 1의 CI 결과에 의존 — 실제로 깨지는 것만 수정
- 기존 macOS/Linux 빌드에 영향 없어야 함
- `#[cfg(target_os)]` 분기는 최소화 — 가능하면 크로스플랫폼으로 통일

## Non-goals

- ARM Windows (aarch64-pc-windows-msvc) — x86_64만 우선
- Windows installer (MSI/NSIS) — ZIP 배포
- Windows-specific UX (PowerShell 자동완성 등)
- Linux 빌드 추가 (별도 플랜)

# Review Report: seCall P13 — Windows 빌드 지원 — Round 1

> Verdict: fail
> Reviewer: 
> Date: 2026-04-09 10:26
> Plan Revision: 0

---

## Verdict

**fail**

## Findings

1. .github/workflows/release.yml:34 — Windows 릴리스가 `onnxruntime.dll` 1.19.2를 번들링하도록 하드코딩되어 있지만, 현재 프로젝트의 `ort = "=2.0.0-rc.10"`이 사용하는 `ort-sys-2.0.0-rc.10`은 ONNX Runtime 1.22.0을 기대합니다. 이 버전 불일치로 배포된 `secall.exe`가 Windows에서 DLL 로드에 실패할 수 있습니다.

## Recommendations

1. ORT DLL 버전은 하드코딩하지 말고 `ort-sys`가 기대하는 버전을 기준으로 맞추거나, 최소한 plan에 적힌 대로 Cargo.lock/ort-sys 근거를 문서화해 동기화하세요.
2. 다음 re-review 전에는 subtask별 Verification 결과를 결과 문서에 분리해 남겨 두면 계약 대조가 더 명확해집니다.

## Subtask Verification

| # | Subtask | Status |
|---|---------|--------|
| 1 | CI에 Windows 빌드 추가 + 깨지는 것 확인 | ✅ done |
| 2 | 네이티브 의존성 컴파일 이슈 수정 | ✅ done |
| 3 | Release 워크플로우에 Windows 바이너리 추가 | ✅ done |


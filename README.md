# DMS Table Mapping Validator

AWS DMS(Database Migration Service)의 **table-mapping JSON**을 로컬에서 검증하는 데스크톱 GUI 툴.
AWS 자격증명 없이, 완전 오프라인으로 동작합니다. 전체 스펙은 [`SPEC.md`](./SPEC.md) 참고.

## 무엇을 검증하나

- **L0 구문**: JSON 파싱, 루트 타입, 2 MB 한도, 중복 키
- **L1 구조/스키마**: 필수 필드·타입·enum 값 (`rule-type`, `rule-action`, `filter-operator`, `data-type` 등)
- **L2 의미/교차필드**: DMS 고유 규칙 — `explicit`/`define-primary-key`/`table-settings` 와일드카드 금지, 필터 연산자별 값 검증, LOB 모드 상호배타, `rule-id`/`rule-name` 고유성, tablespace 상호배타 등
- **버전 게이팅**: 대상 DMS 엔진 버전에 따른 경고(예: 필터 혼합 연산자 3.5.0+)

규칙 카탈로그(코드 `SYN/STR/SEM/BP`)는 [`SPEC.md` §6](./SPEC.md) 및 `crates/dms-core/src/catalog.rs` 참고.

## 아키텍처

```
crates/dms-core      GUI 비의존 검증 엔진 (자체 위치보존 JSON 파서 + 규칙 + 진단)
crates/dms-cli       dms-core 재사용 CLI (dms-validate) — CI 통합용
src-tauri            Tauri 앱 (commands: validate / list_rules / read|write_text_file)
src                  React + TypeScript + Monaco 프론트엔드
samples              검증용 예시 JSON (정상/오류)
```

검증 로직은 전부 `dms-core`(Rust)에 있고, GUI와 CLI가 같은 엔진을 공유합니다.
모든 진단은 `line:column` 범위 + JSON Pointer + 영/한 메시지 + AWS 문서 URL을 포함합니다.

## 사전 준비

- Rust (stable, 1.80+) / Cargo
- Node 18+ 와 pnpm
- macOS는 Xcode CLT, Linux는 webkit2gtk 등 [Tauri 사전요건](https://tauri.app/start/prerequisites/)

## 개발 실행

```bash
pnpm install          # 최초 1회 (esbuild 빌드 승인은 pnpm-workspace.yaml에 설정됨)
pnpm tauri dev        # 데스크톱 앱 실행 (Vite dev 서버 + Tauri 윈도우)
```

> 참고: pnpm 11은 빌드 스크립트를 기본 차단합니다. 이 저장소는 `pnpm-workspace.yaml`에
> `allowBuilds: { esbuild: true }`를 두어 해결합니다. Tauri CLI가 내부적으로 `pnpm install`을
> 호출하므로 이 설정이 없으면 `pnpm tauri ...`가 실패합니다.

## 배포 빌드

```bash
pnpm tauri build      # dist 번들 + 네이티브 앱(.app/.dmg 등) 생성
```

아이콘을 다시 만들려면:

```bash
node scripts/make-icon.mjs
pnpm tauri icon src-tauri/icons/source.png
```

## CLI

```bash
cargo run -p dms-cli -- samples/valid-basic.json          # 사람용 출력
cargo run -p dms-cli -- --json samples/with-errors.json   # ValidationReport JSON
cat mapping.json | cargo run -p dms-cli -- --dms-version 3.4.6
```

검증 오류가 있으면 종료 코드가 0이 아니므로 CI 게이트로 사용할 수 있습니다.

## 테스트

```bash
cargo test            # 코어 단위 + 통합 테스트
pnpm build            # 프론트엔드 타입체크 + 번들
```

## UI 기능 (SPEC §9)

파일 열기/저장·드래그앤드롭, 샘플 템플릿, 실시간 검증(디바운스) 토글, DMS 버전 선택,
에디터 인라인 마킹, 결과 클릭 시 해당 위치로 점프, 심각도 필터, 결과 복사.

## 프라이버시

입력한 JSON은 외부로 전송되지 않습니다. 모든 검증은 로컬에서 수행되며, AWS 자격증명이나
네트워크 연결이 필요하지 않습니다.

## 라이선스

[MIT](./LICENSE) © Return Zero Inc.

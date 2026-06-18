# DMS Table Mapping Validator — 스펙 정의서

> AWS DMS(Database Migration Service)의 **Table Mapping JSON**을 로컬에서 검증하는 데스크톱 GUI 툴.
> 작성일: 2026-06-17 · 상태: 초안(Draft)

---

## 1. 개요 (Overview)

AWS DMS 작업을 만들 때 사용하는 **table-mapping JSON**은 손으로 편집하는 경우가 많고, 오타·잘못된 enum 값·DMS 고유의 의미 규칙 위반(예: `explicit` 액션에 와일드카드 사용)을 AWS 콘솔/API에 올리기 전까지는 발견하기 어렵다. 이 툴은 **AWS 자격증명 없이 로컬에서** table-mapping JSON을 즉시 검증해, AWS에 적용하기 전에 오류를 잡는 것을 목표로 한다.

### 1.1 목표 (Goals)

- table-mapping JSON의 **구조(structure)·타입·enum·필수값** 검증
- DMS 고유의 **의미(semantic)·교차 필드 규칙** 검증 (예: `explicit`는 단일 객체만, table-settings는 와일드카드 금지, LOB 모드 상호배타 등)
- 오류 위치를 **line/column + JSON Pointer**로 정확히 지목하고, 에디터에 인라인 마킹
- **오프라인·로컬 전용**. 입력 JSON을 외부로 전송하지 않음 (민감한 스키마/테이블명 보호)

### 1.2 비목표 (Non-Goals, 이번 범위 밖)

- Task Settings JS(`ReplicationTaskSettings`) 검증 — *추후 확장 후보*
- 실제 AWS DMS API 호출을 통한 라이브 검증 — *추후 확장 후보*
- 소스/타겟 DB에 실제 접속하여 스키마/테이블 존재 여부 확인
- JSON 자동 수정(auto-fix) — *추후 확장 후보 (경고만 제시)*

---

## 2. 범위 (Scope)

| 항목 | 결정 |
|---|---|
| 검증 대상 | **Table Mapping JSON** (단일 종류) |
| 포함 룰 타입 | `selection`, `transformation`, `table-settings`, `post-processing`, `object-mapping` |
| 검증 깊이 | **L0 구문 + L1 구조/스키마 + L2 의미/교차필드** (아래 §5) |
| 플랫폼/스택 | **Tauri** 데스크톱 앱 (Rust 검증 코어 + React/TS 프론트엔드) |
| 동작 모드 | 완전 로컬·오프라인 |

> **object-mapping 룰**(Kinesis/Kafka/DynamoDB 스트리밍 타겟용)은 구조/enum 검증까지 포함하되, 깊은 의미 검증은 1차 범위에서는 **기본 수준**으로 둔다(§6.6). 사용자가 별도 심화 검증을 선택하지 않았으므로 Phase 2 후보.

---

## 3. 용어 (Glossary)

| 용어 | 정의 |
|---|---|
| **rule** | `rules` 배열의 단일 항목. `rule-type`에 따라 동작이 다름 |
| **object-locator** | 룰이 적용될 소스 객체(스키마/테이블/컬럼)를 지정하는 객체 |
| **wildcard** | `%`(0+ 문자), `_`(1 문자), `[ab]`(문자집합), `[a-d]`(범위), `[_]`(리터럴 `_`) |
| **diagnostic** | 검증 결과 1건 (severity + code + message + 위치) |
| **JSON Pointer** | RFC 6901 경로 (예: `/rules/2/object-locator/schema-name`) |

---

## 4. 검증 대상 문서 구조 (DMS Table Mapping JSON 레퍼런스)

> AWS DMS User Guide 기준. 부록 §A에 출처 URL.

### 4.1 루트 구조

```json
{ "rules": [ { /* rule */ }, … ] }
```

- 루트는 **객체**, 키 `rules`(필수)는 **배열**.
- **최소 1개 이상의 `selection` 룰**이 있어야 함. 나머지 룰 타입은 선택.
- DMS는 **최대 2 MB**의 table-mapping 파일만 처리.

### 4.2 공통 필드 (모든 룰)

| 필드 | 필수 | 타입 | 제약 |
|---|---|---|---|
| `rule-type` | ✓ | string(enum) | `selection`·`transformation`·`table-settings`·`post-processing`·`object-mapping` |
| `rule-id` | ✓ | string/number | **고유한 숫자값** (AWS 예시에서는 문자열 `"1"`로 표기) |
| `rule-name` | ✓ | string | **고유한 alphanumeric 값** |

### 4.3 룰 타입별 필드 요약

룰 타입별 상세 enum/필드는 §6 검증 규칙 카탈로그와 §A 부록에 정리한다. 핵심만:

- **selection**: `rule-action`(`include`/`exclude`/`explicit`), `object-locator`{`schema-name`,`table-name`,`table-type?`}, `load-order?`, `filters?`
- **transformation**: `rule-action`(18종, §6.3), `rule-target`(`schema`/`table`/`column`/`table-tablespace`/`index-tablespace`), `object-locator`, 조건부 필드(`value`,`old-value`,`data-type`,`expression`,`primary-key-def`,`before-image-def`)
- **table-settings**: `object-locator`(와일드카드 금지), `parallel-load?`, `lob-settings?`
- **post-processing**: `rule-action`(`add-tag`) — S3 타겟 전용
- **object-mapping**: `rule-action`(`map-record-to-record`/`map-record-to-document`), `target-table-name?`, `mapping-parameters?`, `kafka-target-topic?`

---

## 5. 검증 레벨 (Validation Levels)

검증은 단계별로 실행되며, **앞 단계가 실패하면 뒤 단계의 일부 검사는 건너뛴다**(예: JSON 파싱 실패 시 구조 검증 불가).

| 레벨 | 이름 | 내용 | 실패 시 |
|---|---|---|---|
| **L0** | 구문(Syntax) | JSON 파싱 가능 여부, 루트 타입, 2MB 한도, 중복 키 | 후속 레벨 중단 |
| **L1** | 구조/스키마(Structure) | 필수 필드 유무, 타입, enum 값, 알 수 없는 필드 | 해당 룰의 L2 일부 스킵 |
| **L2** | 의미/교차필드(Semantic) | DMS 고유 규칙: 고유성, 컨텍스트 제약, 상호배타, 와일드카드 허용 여부 등 | — |
| **L3** | 베스트프랙티스(권고) | 경고/정보성 (성능·버전 게이팅 등) | — (warning/info) |

진단 심각도(severity): **`error`**(AWS가 거부할 항목) · **`warning`**(동작하나 위험/권고 위반) · **`info`**(참고).

---

## 6. 검증 규칙 카탈로그 (Validation Rule Catalog)

각 규칙은 코드(`DMS-xxx`), 적용 대상, 심각도, 조건, 메시지를 가진다. 코드 prefix:
`SYN`(구문) · `STR`(구조) · `SEM`(의미) · `BP`(권고).

> **숫자 필드 표기 규칙**: AWS 예시는 숫자 값을 JSON 숫자 또는 **문자열**로 표기한다(`"length":"4092"`, `"scale":""`, `"bulk-max-size":"100000"`, `"load-order":"1"`). 따라서 모든 숫자 검증은 숫자/숫자문자열(scale는 빈문자열 포함)을 동일하게 허용한다. 문서화되지 않은 범위(예: `precision` 1–38, `bulk-max-size` 상한)는 강제하지 않는다. 근거: [`docs/FIELD-CONSTRAINTS.md`](docs/FIELD-CONSTRAINTS.md).

### 6.0 구문 (SYN, L0)

| 코드 | 심각도 | 조건 / 메시지 |
|---|---|---|
| `SYN001` | error | JSON 파싱 실패. line/column 포함 메시지 |
| `SYN002` | error | 루트가 객체가 아님 (배열/스칼라) |
| `SYN003` | warning | 동일 객체 내 **중복 키** (JSON 허용하나 모호) |
| `SYN004` | error | 파일 크기 > 2 MB (DMS 한도 초과) |

### 6.1 루트 & 공통 (STR/SEM)

| 코드 | 심각도 | 조건 / 메시지 |
|---|---|---|
| `STR001` | error | `rules` 키 없음 |
| `STR002` | error | `rules`가 배열이 아님 |
| `STR003` | warning | `rules`가 빈 배열 |
| `STR010` | error | 룰 항목이 객체가 아님 |
| `STR011` | error | 룰에 `rule-type` 없음 |
| `STR012` | error | `rule-type` 값이 enum 밖 |
| `STR013` | error | 룰에 `rule-id` 없음 |
| `STR014` | error | `rule-id`가 **비숫자**(예: `"abc"`) — AWS는 "a unique numeric value"만 명시하므로 비숫자만 error |
| `STR015` | error | 룰에 `rule-name` 없음 |
| `STR016` | warning | 해당 룰 타입에서 **알 수 없는/허용되지 않은 필드** |
| `STR017` | warning | `rule-id`가 음수·소수·2,147,483,647 초과 (값 표시) — 최댓값/부호 규칙은 **미문서**라 warning |
| `STR018` | error | `rule-name`이 문자열이 아님 |
| `SEM001` | error | **`rule-id` 중복** (둘 이상의 룰이 같은 id) |
| `SEM002` | error | **`rule-name` 중복** |
| `SEM003` | error | **`selection` 룰이 0개** (최소 1개 필요) |
| `BP001` | info | 동일 객체를 겨냥한 transformation 룰이 둘 이상 → 낮은 `rule-id`가 적용됨을 안내 |

### 6.2 selection 룰 (STR/SEM)

| 코드 | 심각도 | 조건 / 메시지 |
|---|---|---|
| `STR100` | error | `rule-action` 없음 |
| `STR101` | error | `rule-action`이 `include`/`exclude`/`explicit` 밖 |
| `STR102` | error | `object-locator` 없음/객체 아님 |
| `STR103` | error | `object-locator.schema-name` 없음 |
| `STR104` | error | `object-locator.table-name` 없음 |
| `STR105` | error | `table-type`가 `table`/`view`/`all` 밖 |
| `STR106` | error | `load-order`가 **양의 정수**(1–2,147,483,647)가 아님 (숫자 문자열 허용) |
| `SEM100` | error | `rule-action: explicit`인데 `schema-name`/`table-name`에 **와일드카드 포함** (explicit은 정확한 단일 객체만) |
| `SEM101` | warning | `filters`가 있는데 `rule-action != include` (필터는 include에서만 적용됨) |
| `SEM102` | warning | `explicit` 액션이 여러 객체를 가리킬 소지 (와일드카드는 STR로 이미 error 처리; 여기선 보조 점검) |

#### 6.2.1 filters (selection 내부)

| 코드 | 심각도 | 조건 / 메시지 |
|---|---|---|
| `STR110` | error | `filters` 항목이 객체 아님 |
| `STR111` | error | `filter-type` 값이 `source` 밖 |
| `STR112` | error | `column-name` 없음 |
| `STR113` | error | `filter-conditions` 없음/배열 아님 |
| `STR114` | error | `filter-operator` 없음 |
| `STR115` | error | `filter-operator`가 enum 밖 (`eq`,`noteq`,`gte`,`lte`,`ste`,`between`,`notbetween`,`null`,`notnull`) |
| `SEM110` | error | `between`/`notbetween`인데 `start-value` 또는 `end-value` 누락 |
| `SEM111` | error | `null`/`notnull`인데 `value`/`start-value`/`end-value`가 존재(불필요) |
| `SEM112` | error | 단일값 연산자(`eq`,`noteq`,`gte`,`lte`,`ste`)인데 `value` 누락 |
| `SEM113` | error | 필터 내 `column-name`/테이블에 **와일드카드 사용** (필터는 정확한 이름만) |
| `BP110` | warning | 혼합(긍정+부정) 연산자 사용 → DMS **3.5.0+** 필요 (버전 게이팅, §7.4) |

### 6.3 transformation 룰 (STR/SEM)

`rule-action` enum (18종): `add-column`, `include-column`, `remove-column`, `rename`, `convert-lowercase`, `convert-uppercase`, `add-prefix`, `remove-prefix`, `replace-prefix`, `add-suffix`, `remove-suffix`, `replace-suffix`, `define-primary-key`, `change-data-type`, `add-before-image-columns`, `data-masking-digits-mask`, `data-masking-digits-randomize`, `data-masking-hash-mask`.

`rule-target` enum: `schema`, `table`, `column`, `table-tablespace`, `index-tablespace`.

| 코드 | 심각도 | 조건 / 메시지 |
|---|---|---|
| `STR200` | error | `rule-action` 없음 |
| `STR201` | error | `rule-action`이 18종 밖 |
| `STR202` | error | `rule-target` 없음 |
| `STR203` | error | `rule-target`이 enum 밖 |
| `STR204` | error | `object-locator` 없음/객체 아님 |
| `STR205` | error | `data-type` 객체의 `type`이 DMS 타입 enum 밖 (§A.3) |
| `STR206` | error | `data-type.precision`/`scale`/`length`가 **음수가 아닌 정수**가 아님 |
| `STR207` | error | `primary-key-def`에 `name` 또는 `columns`(배열) 누락 |
| `STR208` | error | `primary-key-def.origin`이 `unique-index` 밖(설정 시) |
| `STR209` | error | `before-image-def.column-filter`가 `pk-only`/`non-lob`/`all` 밖 |
| `SEM200` | error | `change-data-type` / `data-masking-*`인데 `rule-target != column` |
| `SEM201` | error | `add-column`인데 `value`(새 컬럼명) 누락 |
| `SEM202` | error | `replace-prefix`/`replace-suffix`인데 `old-value` 누락 |
| `SEM203` | error | `define-primary-key`인데 `schema-name`/`table-name`에 **와일드카드 사용**(금지) |
| `SEM204` | error | `define-primary-key`인데 `rule-target != table` |
| `SEM205` | error | `add-before-image-columns`인데 `rule-target != column` |
| `SEM206` | error | `*-tablespace` 타겟인데 `table-tablespace-name`/`index-tablespace-name` 동시 지정(상호배타) |
| `SEM207` | warning | `before-image-def`의 `column-prefix`와 `column-suffix`가 둘 다 빈 문자열 |
| `SEM208` | error | `object-locator.column-name` 없음인데 `rule-target == column` |
| `BP200` | warning | 특수문자(`#`,`\`,`/`,`-`) 포함 컬럼에 transformation 적용 (DMS 미지원) |
| `BP201` | warning | BLOB/CLOB 컬럼에 `remove-column` 외 액션 (미지원 가능) |
| `BP202` | warning | `data-masking-*` 액션 → DMS 버전 게이팅 (§7.4) |
| `BP203` | info | `expression` 내 `$Column` 참조/헤더변수/SQLite 함수 사용 안내 (정적 파싱은 best-effort, §6.3.1) |
| `BP204` | warning | `data-masking-digits-mask`의 `value`가 단일 문자가 아님 |

#### 6.3.1 expression 검증 (best-effort)

`expression` 필드는 SQLite 식 + DMS 헤더/메타 변수를 포함한다. **완전 파싱은 비목표**이나 다음은 점검:

| 코드 | 심각도 | 조건 / 메시지 |
|---|---|---|
| `BP210` | warning | 알 수 없는 `$AR_H_*` / `$AR_M_*` 변수명 (오타 의심) |
| `BP211` | info | 괄호 짝 불일치 등 명백한 구문 이상 (간이 검사) |

### 6.4 table-settings 룰 (STR/SEM)

| 코드 | 심각도 | 조건 / 메시지 |
|---|---|---|
| `STR300` | error | `object-locator` 없음 |
| `STR301` | error | `object-locator.schema-name`/`table-name` 없음 |
| `STR310` | error | `parallel-load.type`이 `partitions-auto`/`subpartitions-auto`/`partitions-list`/`ranges` 밖 |
| `STR311` | error | `type: ranges`인데 `columns`/`boundaries` 누락 또는 배열 아님 |
| `STR312` | error | `type: partitions-list`인데 `partitions` 배열 누락 |
| `STR313` | error | `lob-settings.mode`가 `limited`/`unlimited`/`none` 밖 |
| `STR314` | error | `bulk-max-size`가 **음수가 아닌 정수**가 아님 |
| `STR315` | error | `parallel-load`의 숫자 옵션(`number-of-partitions`/`max-records-skip-per-page`/`batch-size`)이 음수가 아닌 정수가 아님 |
| `SEM300` | error | `object-locator.schema-name`/`table-name`에 **와일드카드(`%`) 사용** (table-settings는 정확한 이름만; AWS 검증 오류와 동일) |
| `SEM301` | error | `lob-settings.mode: limited`인데 `bulk-max-size`가 0 또는 누락 (limited는 nonzero 필요) |
| `SEM302` | warning | `ranges`의 `columns`가 10개 초과, 또는 DOUBLE/FLOAT/BLOB/CLOB/NCLOB 타입 사용 권고 위반 (타입 정보 있을 때) |
| `SEM303` | warning | `boundaries` 각 내부 배열 길이가 `columns` 길이와 불일치 |

### 6.5 post-processing 룰 (STR)

| 코드 | 심각도 | 조건 / 메시지 |
|---|---|---|
| `STR400` | error | `rule-action`이 `add-tag` 밖 |
| `BP400` | info | post-processing은 **S3 타겟 전용**임을 안내 |

### 6.6 object-mapping 룰 (STR, 기본 수준)

| 코드 | 심각도 | 조건 / 메시지 |
|---|---|---|
| `STR500` | error | `rule-action`이 `map-record-to-record`/`map-record-to-document` 밖 |
| `STR501` | error | `object-locator` 없음 |
| `STR510` | error | `mapping-parameters.partition-key-type`이 enum 밖 (`schema-table`,`transaction-id`,`primary-key`,`constant`,`attribute-name`) |
| `STR511` | error | `attribute-mappings[].attribute-type`이 `scalar`/`document` 밖 |
| `STR512` | error | `attribute-mappings[].attribute-sub-type`이 `string`/`json` 밖 |
| `SEM500` | error | `partition-key-type: attribute-name`인데 `partition-key-name` 누락 |
| `BP500` | warning | 동일 task에서 object-mapping과 transformation 룰 혼용 (Kafka에서 금지) — *Phase 2에서 심화* |

---

## 7. 진단 결과 데이터 모델 (Diagnostic Model)

검증 코어(Rust)와 프론트엔드(TS)가 공유하는 계약. JSON으로 직렬화되어 Tauri command 결과로 전달.

```jsonc
// ValidationReport
{
  "ok": false,                 // error가 0건이면 true
  "summary": { "errors": 3, "warnings": 1, "infos": 0 },
  "engineVersion": "0.1.0",
  "targetDmsVersion": "3.5.4", // §7.4 버전 게이팅 기준
  "diagnostics": [
    {
      "code": "SEM100",
      "severity": "error",         // error | warning | info
      "message": "rule-action 'explicit' does not allow wildcards in table-name.",
      "messageKo": "rule-action 'explicit'에는 table-name에 와일드카드를 쓸 수 없습니다.",
      "path": "/rules/2/object-locator/table-name",   // JSON Pointer
      "range": { "startLine": 14, "startCol": 21, "endLine": 14, "endCol": 30 },
      "ruleContext": { "ruleId": "2", "ruleName": "sel-2", "ruleType": "selection" },
      "docUrl": "https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.CustomizingTasks.TableMapping.SelectionTransformation.Selections.html"
    }
  ]
}
```

설계 요점:
- **모든 진단에 `range`(line/col)** 를 부여해 에디터 인라인 마킹·클릭 점프 지원. 이를 위해 JSON 파서는 **위치 정보를 보존하는 파서**(예: Rust `serde_json` 대신 위치 보존이 가능한 파서, 또는 `jsonc-parser`류 토크나이저)를 사용한다.
- `message`(영문, AWS 용어 유지) + `messageKo`(한국어 설명) 동시 제공.
- 각 진단에 가능한 경우 **AWS 문서 URL**을 연결(`docUrl`).

---

## 8. 아키텍처 (Architecture)

```
┌─────────────────────────────────────────────┐
│  Tauri App                                    │
│                                               │
│  ┌──────────────┐        ┌──────────────────┐ │
│  │  Frontend     │  IPC   │  Rust Core        │ │
│  │  (React/TS)   │◄──────►│  (validation)     │ │
│  │               │ invoke │                   │ │
│  │ - Monaco 에디터│        │ - 위치보존 파서    │ │
│  │ - 결과 패널    │        │ - L0~L2 검증 엔진  │ │
│  │ - 파일 I/O UI │        │ - 규칙 카탈로그    │ │
│  └──────────────┘        │ - 진단 생성        │ │
│                          └──────────────────┘ │
└─────────────────────────────────────────────┘
```

### 8.1 검증 코어를 Rust에 두는 이유

- **재사용성**: 동일 코어를 추후 CLI(`dms-validate file.json`)로 노출 가능 → CI 통합.
- **단일 진실 공급원**: 규칙 카탈로그(§6)를 Rust에 한 번만 구현.
- **테스트 용이**: 코어를 순수 함수(`validate(text) -> ValidationReport`)로 두어 픽스처 기반 단위 테스트.
- **성능**: 2MB 한도 내 문서를 즉시 검증.

### 8.2 구조 검증 구현 방식 (권고)

- **L1(구조/스키마)**: 정본(canonical) **JSON Schema(Draft 2020-12)** 를 작성해 문서·테스트의 기준으로 삼되, 사용자 대면 진단은 **친화적 메시지 + 정확한 위치**를 위해 Rust 커스텀 검증 패스로 생성한다. (제네릭 JSON Schema 오류는 위치/문구가 거칠어 UX가 떨어짐.)
- **L2(의미)**: §6의 SEM/BP 규칙을 Rust로 구현. 각 규칙은 독립 함수로 두고 카탈로그 코드와 1:1 매핑.

### 8.3 Tauri command 인터페이스(초안)

```rust
#[tauri::command]
fn validate(content: String, target_dms_version: Option<String>) -> ValidationReport;

#[tauri::command]
fn list_rules() -> Vec<RuleDoc>;   // 규칙 카탈로그(코드/설명/문서URL) 제공

// 파일 열기/저장은 Tauri dialog + fs 플러그인 사용
```

---

## 9. UI/UX 명세

### 9.1 레이아웃

```
┌────────────────────────────────────────────────────────┐
│ 툴바: [열기] [저장] [붙여넣기] [샘플▾] [검증] DMS버전[3.5.4▾]│
├──────────────────────────────┬─────────────────────────┤
│                              │  결과 패널                │
│   Monaco JSON 에디터          │  ┌─ Errors (3) ───────┐  │
│   (인라인 에러 밑줄/거터 마커) │  │ SEM100 /rules/2/... │  │
│                              │  │ STR012 /rules/0/... │  │
│                              │  └─────────────────────┘  │
│                              │  ┌─ Warnings (1) ──────┐  │
│                              │  └─────────────────────┘  │
├──────────────────────────────┴─────────────────────────┤
│ 상태바: ● 3 errors  ▲ 1 warning   2MB 중 0.4MB           │
└────────────────────────────────────────────────────────┘
```

### 9.2 기능 요구사항

| ID | 기능 | 설명 |
|---|---|---|
| UI-1 | 파일 열기 | Tauri dialog로 `.json` 열기, 드래그&드롭 지원 |
| UI-2 | 저장 | 현재 내용 저장 / 다른 이름으로 저장 |
| UI-3 | 붙여넣기/직접 편집 | 빈 상태에서 JSON 붙여넣기 |
| UI-4 | 라이브 검증 | 편집 시 디바운스(예: 400ms) 후 자동 검증, 또는 [검증] 버튼 수동 모드 토글 |
| UI-5 | 인라인 마킹 | error=빨강, warning=노랑 밑줄 + 거터 아이콘 + 호버 시 메시지(영/한) |
| UI-6 | 결과 클릭 점프 | 결과 패널 항목 클릭 → 에디터 해당 `range`로 스크롤·하이라이트 |
| UI-7 | 심각도 필터 | error/warning/info 토글 표시 |
| UI-8 | 샘플 템플릿 | selection/transformation 등 정상 예시 삽입 |
| UI-9 | DMS 버전 선택 | 버전 게이팅(§7.4) 기준 변경, 기본=최신 |
| UI-10 | 결과 복사/내보내기 | 진단 목록을 텍스트/JSON으로 복사 (이슈 공유용) |
| UI-11 | 룰 트리 뷰 (선택) | `rules`를 타입별로 접고 펼치는 구조 뷰 |
| UI-12 | 툴팁 | DMS 버전 선택 옆 ⓘ에 버전 게이팅 설명(데이터 마스킹 3.5.4+, 필터 혼합 3.5.0+, Kafka 멀티토픽 3.4.6+) 호버 툴팁. 툴바 버튼·진단 행에는 네이티브 `title` 툴팁 |

### 9.3 비기능 / 프라이버시

- 입력 JSON·검증 결과는 **외부로 전송하지 않음**. 텔레메트리 없음(또는 명시적 opt-in).
- 최근 파일 목록 등 로컬 설정만 디스크에 저장.

---

## 7.4 DMS 버전 게이팅 (Version Gating)

일부 기능은 DMS 엔진 버전에 따라 동작 여부가 달라진다. `targetDmsVersion`(기본=최신, 예 `3.5.x`)을 기준으로 다음을 **warning**으로 안내:

| 기능 | 최소 버전 | 관련 코드 |
|---|---|---|
| 필터 긍정+부정 연산자 혼합 | 3.5.0+ | `BP110` |
| `data-masking-*` transformation | **3.5.4+** | `BP202` |
| `kafka-target-topic`(멀티토픽) | 3.4.6+ | (object-mapping) |
| TimeTravel `RawDataFormat` 등 | 3.5.0+ | (task settings — 범위 밖) |

> 버전별 정확한 매핑은 구현 단계에서 AWS 릴리스 노트로 확정. 미상 시 info로 강등.

---

## 10. 기술 스택 & 의존성 (제안)

| 영역 | 선택 | 비고 |
|---|---|---|
| 셸 | **Tauri 2.x** | 경량·단일 실행파일 |
| 프론트엔드 | **React + TypeScript + Vite** | rtzr 사내 친숙도 |
| 에디터 | **Monaco Editor** | JSON 하이라이트·마커·점프 기본 지원 |
| 검증 코어 | **Rust** | 위치보존 JSON 파서 + 커스텀 검증 |
| JSON 파싱(위치보존) | `jsonc-parser` 류 / 토크나이저 기반 | line/col 진단 필수 |
| 정본 스키마 | JSON Schema 2020-12 | 문서/테스트 기준 |
| 패키징 | Tauri bundler (.dmg/.app) | macOS 우선, 추후 win/linux |

---

## 11. 비기능 요구사항 (NFR)

- **성능**: 2MB 문서 검증 < 100ms (목표), 라이브 검증 디바운스로 입력 지연 없음.
- **정확성**: §6 규칙은 AWS 문서를 단일 출처로 하고, 회귀 테스트 픽스처로 고정.
- **이식성**: 코어는 GUI 비의존 → CLI 재사용 가능.
- **국제화**: 진단 메시지 영/한 동시.
- **오프라인**: 네트워크 불필요.

---

## 12. 마일스톤 (제안)

| 단계 | 산출물 |
|---|---|
| **M0** | 본 스펙 확정 + 정본 JSON Schema + 샘플/픽스처 수집 |
| **M1** | Rust 코어: 위치보존 파서 + L0·L1 검증 + ValidationReport + 단위테스트 |
| **M2** | Rust 코어: L2 의미 규칙(§6.2~6.6) + 버전 게이팅 |
| **M3** | Tauri 셸 + Monaco 에디터 + 결과 패널 + 인라인 마킹/점프 |
| **M4** | 파일 I/O, 샘플 템플릿, 심각도 필터, 내보내기, 패키징(.app) |
| **M5(후보)** | CLI 노출, Task Settings 검증 확장, auto-fix, 라이브 AWS API 검증 |

---

## 13. 미해결/추후 결정 (Open Questions)

1. **라이브 검증 vs 수동 검증** 기본값? (현재 제안: 라이브, 토글 제공)
2. **DMS 기본 타겟 버전** 고정값? (현재 제안: 최신 안정 버전)
3. **object-mapping 심화 검증**을 Phase 2로 미룰지 / 1차에 포함할지
4. **auto-fix**(따옴표/enum 오타 자동 교정) 도입 여부
5. CLI를 M1부터 같이 노출할지 (코어가 이미 분리되어 있어 저비용)

---

## 부록 A. enum / 타입 전체 목록

### A.1 selection
- `rule-action`: `include`, `exclude`, `explicit`
- `table-type`: `table`, `view`, `all`
- `filter-type`: `source`
- `filter-operator`: `eq`, `noteq`, `gte`, `lte`, `ste`, `between`, `notbetween`, `null`, `notnull`
  - 값 카디널리티: 단일값(`eq`,`noteq`,`gte`,`lte`,`ste`)→`value`; 범위(`between`,`notbetween`)→`start-value`+`end-value`; 무값(`null`,`notnull`)
  - ⚠️ `more`/`less`는 DMS 연산자가 **아님** → reject

### A.2 transformation
- `rule-action`(18): `add-column`, `include-column`, `remove-column`, `rename`, `convert-lowercase`, `convert-uppercase`, `add-prefix`, `remove-prefix`, `replace-prefix`, `add-suffix`, `remove-suffix`, `replace-suffix`, `define-primary-key`, `change-data-type`, `add-before-image-columns`, `data-masking-digits-mask`, `data-masking-digits-randomize`, `data-masking-hash-mask`
- `rule-target`: `schema`, `table`, `column`, `table-tablespace`, `index-tablespace`
- `before-image-def.column-filter`: `pk-only`, `non-lob`, `all`
- `primary-key-def.origin`: `unique-index` (미지정 시 primary key)

### A.3 data-type.type (change-data-type / add-column)
`bytes`, `date`, `time`, `datetime`, `int1`, `int2`, `int4`, `int8`, `numeric`, `real4`, `real8`, `string`, `uint1`, `uint2`, `uint4`, `uint8`, `wstring`, `blob`, `nclob`, `clob`, `boolean`, `set`, `list`, `map`, `tuple`
- 포맷: DATE=`YYYY-MM-DD`/`YYYY:MM:DD`, DATETIME=`YYYY-MM-DD HH:MM:SS`, TIME=`HH:MM:SS`

### A.4 table-settings
- `parallel-load.type`: `partitions-auto`, `subpartitions-auto`(Oracle), `partitions-list`, `ranges`
- `lob-settings.mode`: `limited`, `unlimited`, `none`

### A.5 object-mapping
- `rule-action`: `map-record-to-record`, `map-record-to-document`
- `partition-key-type`: `schema-table`, `transaction-id`, `primary-key`, `constant`, `attribute-name`
- `attribute-type`: `scalar`, `document`
- `attribute-sub-type`: `string`, `json`

### A.6 와일드카드
- `%`(0+), `_`(1), `[_]`(리터럴 `_`), `[ab]`(집합), `[a-d]`(범위)
- **금지 컨텍스트**: filters, `define-primary-key` schema/table, `explicit` selection, table-settings object-locator, transformation `object-locator.data-type`
- 모든 schema/table/column 이름은 **대소문자 구분** (Oracle/Db2는 UPPER)

---

## 부록 B. 참고 문서 (AWS DMS User Guide)

- Table mapping (JSON): `…/CHAP_Tasks.CustomizingTasks.TableMapping.SelectionTransformation.html`
- Selection rules: `…/CHAP_Tasks.CustomizingTasks.TableMapping.SelectionTransformation.Selections.html`
- Source filters: `…/CHAP_Tasks.CustomizingTasks.Filters.html`
- Transformation rules: `…/CHAP_Tasks.CustomizingTasks.TableMapping.SelectionTransformation.Transformations.html`
- Transformation expressions: `…/CHAP_Tasks.CustomizingTasks.TableMapping.SelectionTransformation.Expressions.html`
- Table & collection settings: `…/CHAP_Tasks.CustomizingTasks.TableMapping.SelectionTransformation.Tablesettings.html`
- Wildcards: `…/CHAP_Tasks.CustomizingTasks.TableMapping.SelectionTransformation.Wildcards.html`
- Kinesis object mapping: `…/CHAP_Target.Kinesis.html` · Kafka: `…/CHAP_Target.Kafka.html`

(베이스 URL: `https://docs.aws.amazon.com/dms/latest/userguide/`)

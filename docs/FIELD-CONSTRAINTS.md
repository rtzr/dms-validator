# DMS Table-Mapping — 필드별 제약 조사 (구현 근거)

> AWS DMS User Guide 기준으로 각 필드의 **문서화된 제약**을 조사한 결과.
> "DOCUMENTED" = AWS가 명시 / "NOT DOCUMENTED" = 명시 없음(규칙으로 강제 금지) / "(추론)" = 예시 기반 추론.
> 조사일: 2026-06-18. 이 문서는 검증 규칙(`catalog.rs`)의 근거 자료다.

베이스 URL: `https://docs.aws.amazon.com/dms/latest/userguide/`

---

## 1. 공통 필드

| 필드 | 문서화된 제약 | 비고 |
|---|---|---|
| `rule-id` | "**A unique numeric value**" (Selections). 고유성만 명시. | **최댓값·부호·정수 여부 NOT DOCUMENTED.** 모든 예시는 문자열 `"1"`. |
| `rule-name` | "**An alphanumeric value**", 고유. | 하이픈/언더스코어/공백 허용 여부·길이 **NOT DOCUMENTED** → 엄격 alphanumeric 강제 금지(오탐). |
| `load-order` | "**A positive integer. The maximum value is 2,147,483,647.**" 높을수록 먼저 로드. | 최소값(0 vs 1) 명시 없음이나 "positive" → ≥1로 처리. 기본값 NOT DOCUMENTED. |
| `rule-type` | selection/transformation/table-settings/post-processing/object-mapping. | post-processing=S3, object-mapping=스트리밍/DDB 전용. |

→ 구현: `rule-id` 비숫자=STR014(error), 음수·소수·>2,147,483,647=STR017(**warning**, 미문서 제약이므로). `load-order`=STR106(양의 정수, 문자열 허용).

**중요(전 필드 공통):** AWS 예시는 숫자 값을 **문자열로도** 표기한다 — `"rule-id":"1"`, `"load-order":"1"`, `"length":"4092"`, `"scale":""`, `"bulk-max-size":"100000"`. 검증기는 **숫자/숫자문자열/(scale는 빈문자열)** 을 모두 허용해야 한다(`as_int` 헬퍼).

---

## 2. selection / object-locator

| 필드 | 문서화된 제약 |
|---|---|
| `rule-action` | `include` / `exclude` / `explicit` (DOCUMENTED). |
| `object-locator.schema-name`/`table-name` | 둘 다 사용; 길이 제한 NOT DOCUMENTED. 대소문자 구분(소스 엔드포인트 설정에 따라 달라질 수 있음). |
| `table-type` | `table`/`view`/`all`. 기본값 문자열 NOT DOCUMENTED(생략 시 테이블). |
| explicit + 와일드카드 | "no wildcards … only one object" (DOCUMENTED) → 금지. |

---

## 3. filters

| 항목 | 문서화된 제약 |
|---|---|
| `filter-type` | `source`. |
| `filter-operator` | 정확히 9개: `lte, ste, gte, eq, noteq, between, notbetween, null, notnull`. `ste`=`lte` 별칭. **`more`/`less`는 없음.** |
| 값 카디널리티 | `eq/noteq/gte/lte/ste`→`value` 1개; `between/notbetween`→`start-value`+`end-value`; `null/notnull`→값 없음. (DOCUMENTED) |
| 날짜/시간 포맷 | 날짜 `YYYY-MM-DD`, 일시 `YYYY-MM-DD HH:MM:SS.SSS`. 비교는 SQLite 규칙. |
| AND/OR | 여러 filter=AND, 한 filter 내 여러 condition=OR. (DOCUMENTED) |
| 적용 조건 | `rule-action='include'`일 때만. 와일드카드 불가(정확명만). LOB 컬럼 금지(가이드). |
| 혼합 연산자 | 긍정+부정 혼용은 DMS **3.5.0+**. |

---

## 4. transformation

| 항목 | 문서화된 제약 |
|---|---|
| `rule-action` (18종) | add-column, include-column, remove-column, rename, convert-lowercase/uppercase, add/remove/replace-prefix, add/remove/replace-suffix, define-primary-key, change-data-type, add-before-image-columns, data-masking-digits-mask/digits-randomize/hash-mask. 대소문자 구분. |
| `rule-target` | schema/table/column/table-tablespace/index-tablespace. tablespace는 **Oracle 전용**. |
| `data-type.type` | 25종(bytes…tuple). |
| `data-type.precision/scale/length` | "an integer value"만. **범위(1–38 등) NOT DOCUMENTED.** datetime 소수초 scale 최대 9(유일한 명시 상한). scale은 빈문자열 예시 존재. length는 문자열 예시(`"4092"`). |
| `value`/`old-value` | replace-prefix/suffix는 `old-value` 필요(예시). add-column은 `value`=새 컬럼명. data-masking-digits-mask는 `value`=단일 문자. |
| `expression` | SQLite 식만(연산자·명령 불가). 헤더변수 6개($AR_H_*), 메타변수 4개($AR_M_*), 함수 목록 명시. `operation_indicator('D','U','I')`는 소스·타겟 모두 PK 필요. |
| `primary-key-def` | `origin`=`unique-index`(생략 시 PK), `columns` 배열, `name`(예시 항상 존재). columns 최소 개수 NOT DOCUMENTED. |
| `before-image-def` | `column-filter`=`pk-only`/`non-lob`/`all`, `column-prefix` 기본 `BI_`, `column-suffix` 기본 빈값. prefix·suffix 동시 빈값 금지. LOB 미지원. |
| data-masking | **최소 버전 3.5.4**(중요: 3.4.6 아님). digits-mask=value 단일 문자, randomize/hash=파라미터 없음(`mask-format`/`start`/`end`/`key` **NOT DOCUMENTED**). 모두 column 타겟. change-data-type과 동일 컬럼 공존 허용(이때 change-data-type rule-id가 더 작아야 함). |
| 한계 | 객체당 액션 1개(마스킹 예외). 특수문자(`#`,`\`,`/`,`-`) 컬럼 미지원. BLOB/CLOB은 remove-column만. 동일 객체 다중 룰 시 낮은 rule-id 적용. |

---

## 5. table-settings

| 항목 | 문서화된 제약 |
|---|---|
| `parallel-load.type` | partitions-auto / subpartitions-auto(**Oracle 전용**) / partitions-list / ranges. PostgreSQL은 ranges만. |
| `ranges.columns` | **최대 10개.** DOUBLE/FLOAT/BLOB/CLOB/NCLOB 불가. |
| `ranges.boundaries` | 배열의 배열, 각 내부 길이=`columns` 개수. 세그먼트 수 = boundaries + 1. |
| `partitions-list` | `partitions`(문자열 배열), `subpartitions`(Oracle 전용). |
| MongoDB/DocDB partitions-auto | `number-of-partitions` 기본 16, `collection-count-from-metadata` 기본 true, `max-records-skip-per-page` 기본 10000, `batch-size` 기본 0(=서버 최대). **숫자 min/max 전부 NOT DOCUMENTED.** |
| `lob-settings.mode` | limited(기본)/unlimited/none. |
| `lob-settings.bulk-max-size` | 단위 **KB**. limited는 **non-zero 필수**. unlimited는 0=표준/non-zero=조합. **min/max NOT DOCUMENTED.** 문자열 예시(`"100000"`). |
| object-locator | 와일드카드(`%`) 금지(정확한 schema+table). 에러: "Exact schema and table name required when using table settings rule." |
| 병렬 세그먼트 | `MaxFullLoadSubTasks`로 제한, 최대 49. |

---

## 6. object-mapping (Kinesis / Kafka / DynamoDB)

| 항목 | 문서화된 제약 |
|---|---|
| `rule-action` | `map-record-to-record` / `map-record-to-document` (세 타겟 공통). |
| `partition-key-type` | Kinesis: schema-table/transaction-id/primary-key/constant/attribute-name. Kafka: 동일(추론). **DynamoDB는 partition-key-type 미사용** — `partition-key-name`/`sort-key-name` 사용. |
| `partition-key-name` | `attribute-name`일 때 필수(Kinesis/Kafka) + `attribute-mappings` 필요. |
| `attribute-type` | scalar / document. |
| `attribute-sub-type` | Kinesis/Kafka: string/json. **DynamoDB: string/dynamodb-map/dynamodb-list.** (타겟 의존) |
| `value` | `${columnName}` 템플릿. document는 중첩 객체. |
| `kafka-target-topic` | DMS **3.4.6+**. 룰의 형제 키(=mapping-parameters 밖). |
| object-mapping ↔ transformation 공존 금지 | **Kafka 전용.** Kinesis/DynamoDB는 미문서(강제 금지하면 오탐). |

---

## 7. 문서화되지 않음 → 규칙으로 강제 금지 목록

- `rule-id` 최댓값·부호·정수 여부 (→ warning만)
- `rule-name` 길이·문자집합(엄격 alphanumeric 금지)
- `precision`/`scale`/`length` 범위 (1–38 등), 기본값, 단위(bytes/chars)
- `bulk-max-size`/`number-of-partitions`/`max-records-skip-per-page`/`batch-size` min/max
- data-masking의 `mask-format`/`start`/`end`/`key` 파라미터(존재하지 않음)
- `primary-key-def.columns` 최소 개수, `name` 필수 여부
- DynamoDB `partition-key-type`
- Kinesis/DynamoDB의 object-mapping↔transformation 배타성
- `target-table-name`/`kafka-target-topic` 길이·문자집합

---

## 8. 이번 조사로 반영된 변경 (구현)

1. **숫자-문자열 허용**: `as_int` 헬퍼 도입 → `length`/`scale`(빈문자열)/`bulk-max-size`/`load-order`/parallel-load 숫자가 문자열이어도 오탐 없음. (조사 전 오탐이던 항목 수정)
2. **rule-id 재분류**: 비숫자=STR014(error), 음수·소수·과대=STR017(**warning**, 미문서 제약).
3. **load-order**: 양의 정수(1–2,147,483,647)로 정정(STR106).
4. **data-masking 버전 게이팅 3.5.4**로 정정(BP202).
5. **attribute-sub-type**에 `dynamodb-map`/`dynamodb-list` 추가(DynamoDB 오탐 제거, STR512).
6. **BP204** 추가: data-masking-digits-mask value는 단일 문자.

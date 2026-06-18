//! The rule catalog (SPEC §6): every diagnostic code mapped to its severity,
//! default bilingual message, and the relevant AWS doc page.

use crate::model::Severity::{self, *};

const BASE: &str = "https://docs.aws.amazon.com/dms/latest/userguide/";

// Doc page suffixes.
const P_TABLEMAPPING: &str =
    "CHAP_Tasks.CustomizingTasks.TableMapping.SelectionTransformation.html";
const P_SELECTIONS: &str =
    "CHAP_Tasks.CustomizingTasks.TableMapping.SelectionTransformation.Selections.html";
const P_FILTERS: &str = "CHAP_Tasks.CustomizingTasks.Filters.html";
const P_TRANSFORM: &str =
    "CHAP_Tasks.CustomizingTasks.TableMapping.SelectionTransformation.Transformations.html";
const P_EXPR: &str =
    "CHAP_Tasks.CustomizingTasks.TableMapping.SelectionTransformation.Expressions.html";
const P_TABLESETTINGS: &str =
    "CHAP_Tasks.CustomizingTasks.TableMapping.SelectionTransformation.Tablesettings.html";
const P_WILDCARDS: &str =
    "CHAP_Tasks.CustomizingTasks.TableMapping.SelectionTransformation.Wildcards.html";
const P_KINESIS: &str = "CHAP_Target.Kinesis.html";

pub struct Meta {
    pub severity: Severity,
    pub en: &'static str,
    pub ko: &'static str,
    pub page: &'static str,
}

impl Meta {
    pub fn doc_url(&self) -> String {
        format!("{BASE}{}", self.page)
    }
}

/// Every code defined by the catalog, in display order.
pub const ALL_CODES: &[&str] = &[
    "SYN001", "SYN002", "SYN003", "SYN004", //
    "STR001", "STR002", "STR003", "STR010", "STR011", "STR012", "STR013", "STR014", "STR015",
    "STR016", "STR017", "STR018", "SEM001", "SEM002", "SEM003", "BP001", //
    "STR100", "STR101", "STR102", "STR103", "STR104", "STR105", "STR106", "SEM100", "SEM101",
    "SEM102", //
    "STR110", "STR111", "STR112", "STR113", "STR114", "STR115", "SEM110", "SEM111", "SEM112",
    "SEM113", "BP110", //
    "STR200", "STR201", "STR202", "STR203", "STR204", "STR205", "STR206", "STR207", "STR208",
    "STR209", "SEM200", "SEM201", "SEM202", "SEM203", "SEM204", "SEM205", "SEM206", "SEM207",
    "SEM208", "BP200", "BP201", "BP202", "BP203", "BP204", "BP210", "BP211", //
    "STR300", "STR301", "STR310", "STR311", "STR312", "STR313", "STR314", "STR315", "SEM300",
    "SEM301", "SEM302", "SEM303", //
    "STR400", "BP400", //
    "STR500", "STR501", "STR510", "STR511", "STR512", "SEM500", "BP500",
];

pub fn meta(code: &str) -> Meta {
    macro_rules! m {
        ($sev:expr, $en:expr, $ko:expr, $page:expr) => {
            Meta {
                severity: $sev,
                en: $en,
                ko: $ko,
                page: $page,
            }
        };
    }

    match code {
        // -- L0 syntax --
        "SYN001" => m!(Error, "Invalid JSON syntax.", "JSON 구문이 올바르지 않습니다.", P_TABLEMAPPING),
        "SYN002" => m!(Error, "The root must be a JSON object.", "최상위는 JSON 객체여야 합니다.", P_TABLEMAPPING),
        "SYN003" => m!(Warning, "Duplicate object key.", "동일 객체 안에 중복된 키가 있습니다.", P_TABLEMAPPING),
        "SYN004" => m!(Error, "The table-mapping document exceeds the 2 MB DMS limit.", "table-mapping 문서가 DMS 한도(2 MB)를 초과했습니다.", P_TABLEMAPPING),

        // -- root & common --
        "STR001" => m!(Error, "Missing required 'rules' array.", "필수 'rules' 배열이 없습니다.", P_TABLEMAPPING),
        "STR002" => m!(Error, "'rules' must be an array.", "'rules'는 배열이어야 합니다.", P_TABLEMAPPING),
        "STR003" => m!(Warning, "'rules' is empty.", "'rules' 배열이 비어 있습니다.", P_TABLEMAPPING),
        "STR010" => m!(Error, "Each rule must be a JSON object.", "각 룰은 JSON 객체여야 합니다.", P_TABLEMAPPING),
        "STR011" => m!(Error, "Rule is missing 'rule-type'.", "룰에 'rule-type'이 없습니다.", P_TABLEMAPPING),
        "STR012" => m!(Error, "Invalid 'rule-type'.", "'rule-type' 값이 허용 목록에 없습니다.", P_TABLEMAPPING),
        "STR013" => m!(Error, "Rule is missing 'rule-id'.", "룰에 'rule-id'가 없습니다.", P_TABLEMAPPING),
        "STR014" => m!(Error, "'rule-id' must be a numeric value.", "'rule-id'는 숫자 값이어야 합니다.", P_TABLEMAPPING),
        "STR015" => m!(Error, "Rule is missing 'rule-name'.", "룰에 'rule-name'이 없습니다.", P_TABLEMAPPING),
        "STR016" => m!(Warning, "Unknown field for this rule type.", "이 룰 타입에서 알 수 없는 필드입니다.", P_TABLEMAPPING),
        "STR017" => m!(Warning, "'rule-id' should be a small non-negative integer; AWS documents only 'a unique numeric value' (no documented max/sign), so unusual values (negative, decimal, or > 2,147,483,647) may be rejected.", "'rule-id'는 작은 음수가 아닌 정수가 권장됩니다. AWS는 'a unique numeric value'만 명시(최댓값·부호 규칙 미문서화)하므로 비정상 값(음수·소수·2,147,483,647 초과)은 거부될 수 있습니다.", P_TABLEMAPPING),
        "STR018" => m!(Error, "'rule-name' must be a string.", "'rule-name'은 문자열이어야 합니다.", P_TABLEMAPPING),
        "SEM001" => m!(Error, "Duplicate 'rule-id'; rule ids must be unique.", "'rule-id'가 중복되었습니다. 룰 id는 고유해야 합니다.", P_TABLEMAPPING),
        "SEM002" => m!(Error, "Duplicate 'rule-name'; rule names must be unique.", "'rule-name'이 중복되었습니다. 룰 이름은 고유해야 합니다.", P_TABLEMAPPING),
        "SEM003" => m!(Error, "At least one 'selection' rule is required.", "최소 1개의 'selection' 룰이 필요합니다.", P_SELECTIONS),
        "BP001" => m!(Info, "Multiple transformation rules target the same object; the lower rule-id is applied.", "동일 객체를 겨냥한 transformation 룰이 여러 개입니다. 더 낮은 rule-id가 적용됩니다.", P_TRANSFORM),

        // -- selection --
        "STR100" => m!(Error, "Selection rule is missing 'rule-action'.", "selection 룰에 'rule-action'이 없습니다.", P_SELECTIONS),
        "STR101" => m!(Error, "Invalid selection 'rule-action'.", "selection 'rule-action' 값이 올바르지 않습니다.", P_SELECTIONS),
        "STR102" => m!(Error, "Selection rule is missing 'object-locator'.", "selection 룰에 'object-locator'가 없습니다.", P_SELECTIONS),
        "STR103" => m!(Error, "'object-locator' is missing 'schema-name'.", "'object-locator'에 'schema-name'이 없습니다.", P_SELECTIONS),
        "STR104" => m!(Error, "'object-locator' is missing 'table-name'.", "'object-locator'에 'table-name'이 없습니다.", P_SELECTIONS),
        "STR105" => m!(Error, "Invalid 'table-type'.", "'table-type' 값이 올바르지 않습니다.", P_SELECTIONS),
        "STR106" => m!(Error, "'load-order' must be a positive integer (1 – 2,147,483,647).", "'load-order'는 양의 정수(1 – 2,147,483,647)여야 합니다.", P_SELECTIONS),
        "SEM100" => m!(Error, "'explicit' selection rules cannot use wildcards.", "'explicit' selection 룰에는 와일드카드를 쓸 수 없습니다.", P_WILDCARDS),
        "SEM101" => m!(Warning, "'filters' only apply when rule-action is 'include'.", "'filters'는 rule-action이 'include'일 때만 적용됩니다.", P_FILTERS),
        "SEM102" => m!(Warning, "'explicit' selection rules must reference exactly one object.", "'explicit' selection 룰은 정확히 하나의 객체만 가리켜야 합니다.", P_SELECTIONS),

        // -- filters --
        "STR110" => m!(Error, "Each filter must be a JSON object.", "각 filter는 JSON 객체여야 합니다.", P_FILTERS),
        "STR111" => m!(Error, "Invalid 'filter-type'.", "'filter-type' 값이 올바르지 않습니다.", P_FILTERS),
        "STR112" => m!(Error, "Filter is missing 'column-name'.", "filter에 'column-name'이 없습니다.", P_FILTERS),
        "STR113" => m!(Error, "'filter-conditions' must be a non-empty array.", "'filter-conditions'는 비어있지 않은 배열이어야 합니다.", P_FILTERS),
        "STR114" => m!(Error, "Filter condition is missing 'filter-operator'.", "filter-condition에 'filter-operator'가 없습니다.", P_FILTERS),
        "STR115" => m!(Error, "Invalid 'filter-operator'.", "'filter-operator' 값이 올바르지 않습니다.", P_FILTERS),
        "SEM110" => m!(Error, "'between'/'notbetween' require both 'start-value' and 'end-value'.", "'between'/'notbetween'은 'start-value'와 'end-value'가 모두 필요합니다.", P_FILTERS),
        "SEM111" => m!(Error, "'null'/'notnull' must not carry a value.", "'null'/'notnull'에는 값을 지정하지 않습니다.", P_FILTERS),
        "SEM112" => m!(Error, "This operator requires a 'value'.", "이 연산자는 'value'가 필요합니다.", P_FILTERS),
        "SEM113" => m!(Error, "Filters cannot use wildcards.", "filter에는 와일드카드를 쓸 수 없습니다.", P_FILTERS),
        "BP110" => m!(Warning, "Mixing positive and negative filter operators requires DMS 3.5.0+.", "긍정/부정 필터 연산자 혼용은 DMS 3.5.0 이상이 필요합니다.", P_FILTERS),

        // -- transformation --
        "STR200" => m!(Error, "Transformation rule is missing 'rule-action'.", "transformation 룰에 'rule-action'이 없습니다.", P_TRANSFORM),
        "STR201" => m!(Error, "Invalid transformation 'rule-action'.", "transformation 'rule-action' 값이 올바르지 않습니다.", P_TRANSFORM),
        "STR202" => m!(Error, "Transformation rule is missing 'rule-target'.", "transformation 룰에 'rule-target'이 없습니다.", P_TRANSFORM),
        "STR203" => m!(Error, "Invalid 'rule-target'.", "'rule-target' 값이 올바르지 않습니다.", P_TRANSFORM),
        "STR204" => m!(Error, "Transformation rule is missing 'object-locator'.", "transformation 룰에 'object-locator'가 없습니다.", P_TRANSFORM),
        "STR205" => m!(Error, "Invalid 'data-type.type'.", "'data-type.type' 값이 올바르지 않습니다.", P_TRANSFORM),
        "STR206" => m!(Error, "'data-type' precision/scale/length must be non-negative integers.", "'data-type'의 precision/scale/length는 음수가 아닌 정수여야 합니다.", P_TRANSFORM),
        "STR207" => m!(Error, "'primary-key-def' requires 'name' and a 'columns' array.", "'primary-key-def'에는 'name'과 'columns' 배열이 필요합니다.", P_TRANSFORM),
        "STR208" => m!(Error, "Invalid 'primary-key-def.origin'.", "'primary-key-def.origin' 값이 올바르지 않습니다.", P_TRANSFORM),
        "STR209" => m!(Error, "Invalid 'before-image-def.column-filter'.", "'before-image-def.column-filter' 값이 올바르지 않습니다.", P_TRANSFORM),
        "SEM200" => m!(Error, "This action is only valid when rule-target is 'column'.", "이 액션은 rule-target이 'column'일 때만 유효합니다.", P_TRANSFORM),
        "SEM201" => m!(Error, "'add-column' requires 'value' (the new column name).", "'add-column'에는 'value'(새 컬럼명)가 필요합니다.", P_TRANSFORM),
        "SEM202" => m!(Error, "'replace-prefix'/'replace-suffix' require 'old-value'.", "'replace-prefix'/'replace-suffix'에는 'old-value'가 필요합니다.", P_TRANSFORM),
        "SEM203" => m!(Error, "'define-primary-key' cannot use wildcards in schema/table name.", "'define-primary-key'의 schema/table 이름에는 와일드카드를 쓸 수 없습니다.", P_WILDCARDS),
        "SEM204" => m!(Error, "'define-primary-key' requires rule-target 'table'.", "'define-primary-key'는 rule-target이 'table'이어야 합니다.", P_TRANSFORM),
        "SEM205" => m!(Error, "'add-before-image-columns' requires rule-target 'column'.", "'add-before-image-columns'는 rule-target이 'column'이어야 합니다.", P_TRANSFORM),
        "SEM206" => m!(Error, "'table-tablespace-name' and 'index-tablespace-name' are mutually exclusive.", "'table-tablespace-name'과 'index-tablespace-name'은 동시에 쓸 수 없습니다.", P_TRANSFORM),
        "SEM207" => m!(Warning, "'before-image-def' should not have both prefix and suffix empty.", "'before-image-def'의 prefix와 suffix를 모두 빈 문자열로 두면 안 됩니다.", P_TRANSFORM),
        "SEM208" => m!(Error, "rule-target 'column' requires 'object-locator.column-name'.", "rule-target이 'column'이면 'object-locator.column-name'이 필요합니다.", P_TRANSFORM),
        "BP200" => m!(Warning, "Transformations on columns with special characters (#, \\, /, -) are unsupported.", "특수문자(#, \\, /, -)가 포함된 컬럼에는 transformation이 지원되지 않습니다.", P_TRANSFORM),
        "BP201" => m!(Warning, "Only 'remove-column' is supported on BLOB/CLOB columns.", "BLOB/CLOB 컬럼에는 'remove-column'만 지원됩니다.", P_TRANSFORM),
        "BP202" => m!(Warning, "Data-masking actions require DMS engine version 3.5.4 or later.", "데이터 마스킹 액션은 DMS 엔진 3.5.4 이상이 필요합니다.", P_TRANSFORM),
        "BP203" => m!(Info, "Expression uses $-references; static validation is best-effort.", "expression의 $참조는 정적 검증이 best-effort입니다.", P_EXPR),
        "BP204" => m!(Warning, "'data-masking-digits-mask' value should be a single ASCII printable character.", "'data-masking-digits-mask'의 value는 단일 ASCII 출력 가능 문자여야 합니다.", P_TRANSFORM),
        "BP210" => m!(Warning, "Unknown $AR_H_*/$AR_M_* expression variable (possible typo).", "알 수 없는 $AR_H_*/$AR_M_* expression 변수입니다(오타 의심).", P_EXPR),
        "BP211" => m!(Info, "Possible syntax issue in expression (unbalanced parentheses).", "expression에 구문 이상이 있을 수 있습니다(괄호 짝 불일치).", P_EXPR),

        // -- table-settings --
        "STR300" => m!(Error, "table-settings rule is missing 'object-locator'.", "table-settings 룰에 'object-locator'가 없습니다.", P_TABLESETTINGS),
        "STR301" => m!(Error, "'object-locator' requires 'schema-name' and 'table-name'.", "'object-locator'에는 'schema-name'과 'table-name'이 필요합니다.", P_TABLESETTINGS),
        "STR310" => m!(Error, "Invalid 'parallel-load.type'.", "'parallel-load.type' 값이 올바르지 않습니다.", P_TABLESETTINGS),
        "STR311" => m!(Error, "'ranges' requires 'columns' and 'boundaries' arrays.", "'ranges'에는 'columns'와 'boundaries' 배열이 필요합니다.", P_TABLESETTINGS),
        "STR312" => m!(Error, "'partitions-list' requires a 'partitions' array.", "'partitions-list'에는 'partitions' 배열이 필요합니다.", P_TABLESETTINGS),
        "STR313" => m!(Error, "Invalid 'lob-settings.mode'.", "'lob-settings.mode' 값이 올바르지 않습니다.", P_TABLESETTINGS),
        "STR314" => m!(Error, "'bulk-max-size' must be a non-negative integer.", "'bulk-max-size'는 음수가 아닌 정수여야 합니다.", P_TABLESETTINGS),
        "STR315" => m!(Error, "parallel-load numeric option must be a non-negative integer.", "parallel-load의 숫자 옵션은 음수가 아닌 정수여야 합니다.", P_TABLESETTINGS),
        "SEM300" => m!(Error, "table-settings 'object-locator' cannot use wildcards.", "table-settings의 'object-locator'에는 와일드카드를 쓸 수 없습니다.", P_TABLESETTINGS),
        "SEM301" => m!(Error, "lob-settings mode 'limited' requires a non-zero 'bulk-max-size'.", "lob-settings의 'limited' 모드는 0이 아닌 'bulk-max-size'가 필요합니다.", P_TABLESETTINGS),
        "SEM302" => m!(Warning, "'ranges' allows at most 10 columns and not DOUBLE/FLOAT/BLOB/CLOB/NCLOB.", "'ranges'는 컬럼 최대 10개이며 DOUBLE/FLOAT/BLOB/CLOB/NCLOB은 권장되지 않습니다.", P_TABLESETTINGS),
        "SEM303" => m!(Warning, "Each 'boundaries' entry length should match the 'columns' count.", "'boundaries' 각 항목 길이는 'columns' 개수와 일치해야 합니다.", P_TABLESETTINGS),

        // -- post-processing --
        "STR400" => m!(Error, "post-processing rule-action must be 'add-tag'.", "post-processing의 rule-action은 'add-tag'여야 합니다.", P_TABLEMAPPING),
        "BP400" => m!(Info, "post-processing rules apply to Amazon S3 targets only.", "post-processing 룰은 Amazon S3 타겟에만 적용됩니다.", P_TABLEMAPPING),

        // -- object-mapping --
        "STR500" => m!(Error, "Invalid object-mapping 'rule-action'.", "object-mapping 'rule-action' 값이 올바르지 않습니다.", P_KINESIS),
        "STR501" => m!(Error, "object-mapping rule is missing 'object-locator'.", "object-mapping 룰에 'object-locator'가 없습니다.", P_KINESIS),
        "STR510" => m!(Error, "Invalid 'partition-key-type'.", "'partition-key-type' 값이 올바르지 않습니다.", P_KINESIS),
        "STR511" => m!(Error, "Invalid 'attribute-type'.", "'attribute-type' 값이 올바르지 않습니다.", P_KINESIS),
        "STR512" => m!(Error, "Invalid 'attribute-sub-type'.", "'attribute-sub-type' 값이 올바르지 않습니다.", P_KINESIS),
        "SEM500" => m!(Error, "partition-key-type 'attribute-name' requires 'partition-key-name'.", "partition-key-type이 'attribute-name'이면 'partition-key-name'이 필요합니다.", P_KINESIS),
        "BP500" => m!(Warning, "object-mapping and transformation rules cannot be combined (Kafka).", "object-mapping과 transformation 룰은 함께 쓸 수 없습니다(Kafka).", P_KINESIS),

        _ => m!(Error, "Unknown diagnostic.", "알 수 없는 진단입니다.", P_TABLEMAPPING),
    }
}

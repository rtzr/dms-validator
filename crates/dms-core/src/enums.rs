//! Allowed enum value sets for DMS table-mapping JSON (SPEC §A).

pub const RULE_TYPES: &[&str] = &[
    "selection",
    "transformation",
    "table-settings",
    "post-processing",
    "object-mapping",
];

// --- selection ---
pub const SELECTION_ACTIONS: &[&str] = &["include", "exclude", "explicit"];
pub const TABLE_TYPES: &[&str] = &["table", "view", "all"];
pub const FILTER_TYPES: &[&str] = &["source"];
pub const FILTER_OPERATORS: &[&str] = &[
    "eq",
    "noteq",
    "gte",
    "lte",
    "ste",
    "between",
    "notbetween",
    "null",
    "notnull",
];
/// Operators taking a single `value`.
pub const SINGLE_VALUE_OPERATORS: &[&str] = &["eq", "noteq", "gte", "lte", "ste"];
/// Operators taking `start-value` + `end-value`.
pub const RANGE_OPERATORS: &[&str] = &["between", "notbetween"];
/// Operators taking no value at all.
pub const NO_VALUE_OPERATORS: &[&str] = &["null", "notnull"];
/// Negative operators — mixing these with positive ones needs DMS 3.5.0+.
pub const NEGATIVE_OPERATORS: &[&str] = &["noteq", "notbetween", "notnull"];
pub const POSITIVE_OPERATORS: &[&str] = &["eq", "gte", "lte", "ste", "between"];

// --- transformation ---
pub const TRANSFORMATION_ACTIONS: &[&str] = &[
    "add-column",
    "include-column",
    "remove-column",
    "rename",
    "convert-lowercase",
    "convert-uppercase",
    "add-prefix",
    "remove-prefix",
    "replace-prefix",
    "add-suffix",
    "remove-suffix",
    "replace-suffix",
    "define-primary-key",
    "change-data-type",
    "add-before-image-columns",
    "data-masking-digits-mask",
    "data-masking-digits-randomize",
    "data-masking-hash-mask",
];
pub const DATA_MASKING_ACTIONS: &[&str] = &[
    "data-masking-digits-mask",
    "data-masking-digits-randomize",
    "data-masking-hash-mask",
];
pub const TRANSFORMATION_TARGETS: &[&str] = &[
    "schema",
    "table",
    "column",
    "table-tablespace",
    "index-tablespace",
];
pub const DATA_TYPES: &[&str] = &[
    "bytes", "date", "time", "datetime", "int1", "int2", "int4", "int8", "numeric", "real4",
    "real8", "string", "uint1", "uint2", "uint4", "uint8", "wstring", "blob", "nclob", "clob",
    "boolean", "set", "list", "map", "tuple",
];
pub const COLUMN_FILTERS: &[&str] = &["pk-only", "non-lob", "all"];
pub const PK_ORIGINS: &[&str] = &["unique-index"];
/// Known DMS header / metadata expression variables (without the leading `$`).
pub const EXPRESSION_VARS: &[&str] = &[
    "AR_H_STREAM_POSITION",
    "AR_H_TIMESTAMP",
    "AR_H_COMMIT_TIMESTAMP",
    "AR_H_OPERATION",
    "AR_H_USER",
    "AR_H_CHANGE_SEQ",
    "AR_M_SOURCE_SCHEMA",
    "AR_M_SOURCE_TABLE_NAME",
    "AR_M_SOURCE_COLUMN_NAME",
    "AR_M_SOURCE_COLUMN_DATATYPE",
];

// --- table-settings ---
pub const PARALLEL_LOAD_TYPES: &[&str] = &[
    "partitions-auto",
    "subpartitions-auto",
    "partitions-list",
    "ranges",
];
pub const LOB_MODES: &[&str] = &["limited", "unlimited", "none"];

// --- post-processing ---
pub const POST_PROCESSING_ACTIONS: &[&str] = &["add-tag"];

// --- object-mapping ---
pub const OBJECT_MAPPING_ACTIONS: &[&str] = &["map-record-to-record", "map-record-to-document"];
pub const PARTITION_KEY_TYPES: &[&str] = &[
    "schema-table",
    "transaction-id",
    "primary-key",
    "constant",
    "attribute-name",
];
pub const ATTRIBUTE_TYPES: &[&str] = &["scalar", "document"];
// string/json (Kinesis/Kafka) + dynamodb-map/dynamodb-list (DynamoDB targets).
pub const ATTRIBUTE_SUB_TYPES: &[&str] = &["string", "json", "dynamodb-map", "dynamodb-list"];

pub fn contains(set: &[&str], value: &str) -> bool {
    set.contains(&value)
}

/// Comma-separated quoted list for error messages, e.g. `'a', 'b', 'c'`.
pub fn quoted_list(set: &[&str]) -> String {
    set.iter()
        .map(|v| format!("'{v}'"))
        .collect::<Vec<_>>()
        .join(", ")
}

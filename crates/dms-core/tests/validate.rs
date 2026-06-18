//! Integration tests for the table-mapping validator (SPEC §6).

use dms_core::{validate, Severity};

fn codes(content: &str) -> Vec<String> {
    validate(content, None)
        .diagnostics
        .into_iter()
        .map(|d| d.code)
        .collect()
}

fn has(content: &str, code: &str) -> bool {
    codes(content).iter().any(|c| c == code)
}

const VALID: &str = r#"{
  "rules": [
    {
      "rule-type": "selection",
      "rule-id": "1",
      "rule-name": "sel-1",
      "rule-action": "include",
      "object-locator": { "schema-name": "HR", "table-name": "employees" }
    },
    {
      "rule-type": "transformation",
      "rule-id": "2",
      "rule-name": "xf-1",
      "rule-action": "rename",
      "rule-target": "table",
      "object-locator": { "schema-name": "HR", "table-name": "employees" },
      "value": "emp"
    }
  ]
}"#;

#[test]
fn valid_document_has_no_errors() {
    let report = validate(VALID, None);
    assert!(report.ok, "expected ok, got: {:?}", report.diagnostics);
    assert_eq!(report.summary.errors, 0);
}

#[test]
fn invalid_json_reports_syn001() {
    let report = validate("{ not json", None);
    assert!(!report.ok);
    assert!(report.diagnostics.iter().any(|d| d.code == "SYN001"));
    // SYN001 carries a position.
    let d = report
        .diagnostics
        .iter()
        .find(|d| d.code == "SYN001")
        .unwrap();
    assert!(d.range.is_some());
}

#[test]
fn missing_rules_reports_str001() {
    assert!(has(r#"{"foo": 1}"#, "STR001"));
}

#[test]
fn root_not_object_reports_syn002() {
    assert!(has("[]", "SYN002"));
}

#[test]
fn no_selection_rule_reports_sem003() {
    let doc = r#"{"rules":[
      {"rule-type":"transformation","rule-id":"1","rule-name":"x",
       "rule-action":"rename","rule-target":"table",
       "object-locator":{"schema-name":"s","table-name":"t"},"value":"u"}
    ]}"#;
    assert!(has(doc, "SEM003"));
}

#[test]
fn duplicate_rule_id_reports_sem001() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"a","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"}},
      {"rule-type":"selection","rule-id":"1","rule-name":"b","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"u"}}
    ]}"#;
    assert!(has(doc, "SEM001"));
}

#[test]
fn duplicate_rule_name_reports_sem002() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"dup","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"}},
      {"rule-type":"selection","rule-id":"2","rule-name":"dup","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"u"}}
    ]}"#;
    assert!(has(doc, "SEM002"));
}

#[test]
fn explicit_with_wildcard_reports_sem100() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"a","rule-action":"explicit",
       "object-locator":{"schema-name":"HR","table-name":"emp%"}}
    ]}"#;
    assert!(has(doc, "SEM100"));
}

#[test]
fn underscore_is_not_a_wildcard() {
    // Plain underscore in an explicit name must NOT trigger SEM100.
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"a","rule-action":"explicit",
       "object-locator":{"schema-name":"HR","table-name":"my_table"}}
    ]}"#;
    assert!(!has(doc, "SEM100"));
}

#[test]
fn invalid_filter_operator_reports_str115() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"a","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"},
       "filters":[{"filter-type":"source","column-name":"id",
         "filter-conditions":[{"filter-operator":"more","value":"1"}]}]}
    ]}"#;
    assert!(has(doc, "STR115"));
}

#[test]
fn between_without_bounds_reports_sem110() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"a","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"},
       "filters":[{"filter-type":"source","column-name":"id",
         "filter-conditions":[{"filter-operator":"between","value":"1"}]}]}
    ]}"#;
    assert!(has(doc, "SEM110"));
}

#[test]
fn filters_outside_include_reports_sem101() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"a","rule-action":"exclude",
       "object-locator":{"schema-name":"s","table-name":"t"},
       "filters":[{"filter-type":"source","column-name":"id",
         "filter-conditions":[{"filter-operator":"eq","value":"1"}]}]}
    ]}"#;
    assert!(has(doc, "SEM101"));
}

#[test]
fn change_data_type_on_non_column_reports_sem200() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"s","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"}},
      {"rule-type":"transformation","rule-id":"2","rule-name":"x","rule-action":"change-data-type",
       "rule-target":"table","object-locator":{"schema-name":"s","table-name":"t"},
       "data-type":{"type":"string","length":50}}
    ]}"#;
    assert!(has(doc, "SEM200"));
}

#[test]
fn invalid_data_type_reports_str205() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"s","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"}},
      {"rule-type":"transformation","rule-id":"2","rule-name":"x","rule-action":"change-data-type",
       "rule-target":"column",
       "object-locator":{"schema-name":"s","table-name":"t","column-name":"c"},
       "data-type":{"type":"varchar2"}}
    ]}"#;
    assert!(has(doc, "STR205"));
}

#[test]
fn define_primary_key_with_wildcard_reports_sem203() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"s","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"}},
      {"rule-type":"transformation","rule-id":"2","rule-name":"x","rule-action":"define-primary-key",
       "rule-target":"table","object-locator":{"schema-name":"HR%","table-name":"t"},
       "primary-key-def":{"name":"pk","columns":["id"]}}
    ]}"#;
    assert!(has(doc, "SEM203"));
}

#[test]
fn table_settings_wildcard_reports_sem300() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"s","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"}},
      {"rule-type":"table-settings","rule-id":"2","rule-name":"ts",
       "object-locator":{"schema-name":"HR","table-name":"emp%"},
       "lob-settings":{"mode":"limited","bulk-max-size":32}}
    ]}"#;
    assert!(has(doc, "SEM300"));
}

#[test]
fn lob_limited_zero_size_reports_sem301() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"s","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"}},
      {"rule-type":"table-settings","rule-id":"2","rule-name":"ts",
       "object-locator":{"schema-name":"HR","table-name":"emp"},
       "lob-settings":{"mode":"limited","bulk-max-size":0}}
    ]}"#;
    assert!(has(doc, "SEM301"));
}

#[test]
fn object_mapping_invalid_action_reports_str500() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"s","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"}},
      {"rule-type":"object-mapping","rule-id":"2","rule-name":"om",
       "rule-action":"map-record","object-locator":{"schema-name":"s","table-name":"t"}}
    ]}"#;
    assert!(has(doc, "STR500"));
}

#[test]
fn unknown_field_reports_str016_warning() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"s","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"},
       "typoo-field":true}
    ]}"#;
    let report = validate(doc, None);
    let d = report.diagnostics.iter().find(|d| d.code == "STR016");
    assert!(d.is_some());
    assert_eq!(d.unwrap().severity, Severity::Warning);
    assert!(
        report.ok,
        "STR016 is a warning and must not fail the document"
    );
}

#[test]
fn oversized_rule_id_reports_str017_warning() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"999999999999999999999999","rule-name":"a",
       "rule-action":"include","object-locator":{"schema-name":"s","table-name":"t"}}
    ]}"#;
    let report = validate(doc, None);
    let d = report.diagnostics.iter().find(|d| d.code == "STR017");
    assert!(d.is_some(), "expected STR017 for oversized rule-id");
    assert_eq!(d.unwrap().severity, Severity::Warning);
}

#[test]
fn rule_id_just_over_i32_reports_str017() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":2147483648,"rule-name":"a",
       "rule-action":"include","object-locator":{"schema-name":"s","table-name":"t"}}
    ]}"#;
    assert!(has(doc, "STR017"));
}

#[test]
fn negative_numeric_rule_id_reports_str017_warning() {
    // AWS only documents "a unique numeric value"; a negative is unusual, not a
    // documented hard error, so it is a warning (STR017), not STR014.
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":-5,"rule-name":"a",
       "rule-action":"include","object-locator":{"schema-name":"s","table-name":"t"}}
    ]}"#;
    assert!(has(doc, "STR017"));
    assert!(!has(doc, "STR014"));
}

#[test]
fn non_numeric_rule_id_reports_str014() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"abc","rule-name":"a",
       "rule-action":"include","object-locator":{"schema-name":"s","table-name":"t"}}
    ]}"#;
    assert!(has(doc, "STR014"));
}

#[test]
fn numeric_string_fields_are_not_false_positives() {
    // AWS examples encode numeric fields as strings and use empty scale.
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"s","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"},"load-order":"5"},
      {"rule-type":"transformation","rule-id":"2","rule-name":"x","rule-action":"add-column",
       "rule-target":"column","object-locator":{"schema-name":"s","table-name":"t","column-name":"c"},
       "value":"c2","data-type":{"type":"string","length":"4092","scale":""}},
      {"rule-type":"table-settings","rule-id":"3","rule-name":"ts",
       "object-locator":{"schema-name":"s","table-name":"t"},
       "lob-settings":{"mode":"limited","bulk-max-size":"100000"}}
    ]}"#;
    let report = validate(doc, None);
    assert!(
        !has(doc, "STR206"),
        "string/empty length & scale must be accepted"
    );
    assert!(!has(doc, "STR314"), "string bulk-max-size must be accepted");
    assert!(
        !has(doc, "SEM301"),
        "string non-zero bulk-max-size satisfies limited mode"
    );
    assert!(!has(doc, "STR106"), "string load-order must be accepted");
    assert!(report.ok, "diagnostics: {:?}", report.diagnostics);
}

#[test]
fn load_order_zero_reports_str106() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"a","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"},"load-order":0}
    ]}"#;
    assert!(has(doc, "STR106"));
}

#[test]
fn dynamodb_attribute_sub_type_is_accepted() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"s","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"}},
      {"rule-type":"object-mapping","rule-id":"2","rule-name":"om",
       "rule-action":"map-record-to-document","object-locator":{"schema-name":"s","table-name":"t"},
       "mapping-parameters":{"partition-key-type":"primary-key",
         "attribute-mappings":[{"target-attribute-name":"d","attribute-type":"document",
           "attribute-sub-type":"dynamodb-map","value":{"x":"${c}"}}]}}
    ]}"#;
    assert!(
        !has(doc, "STR512"),
        "dynamodb-map sub-type must be accepted"
    );
}

#[test]
fn data_masking_version_gate_is_3_5_4() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"s","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"}},
      {"rule-type":"transformation","rule-id":"2","rule-name":"m","rule-action":"data-masking-hash-mask",
       "rule-target":"column","object-locator":{"schema-name":"s","table-name":"t","column-name":"c"}}
    ]}"#;
    assert!(!has(doc, "BP202"), "default target 3.5.4 supports masking");
    assert!(validate(doc, Some("3.5.0"))
        .diagnostics
        .iter()
        .any(|d| d.code == "BP202"));
}

#[test]
fn valid_small_rule_id_has_no_range_warning() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"42","rule-name":"a",
       "rule-action":"include","object-locator":{"schema-name":"s","table-name":"t"}}
    ]}"#;
    assert!(!has(doc, "STR017"));
    assert!(!has(doc, "STR014"));
}

#[test]
fn negative_precision_reports_str206() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"s","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"}},
      {"rule-type":"transformation","rule-id":"2","rule-name":"x","rule-action":"change-data-type",
       "rule-target":"column","object-locator":{"schema-name":"s","table-name":"t","column-name":"c"},
       "data-type":{"type":"numeric","precision":-3,"scale":2}}
    ]}"#;
    assert!(has(doc, "STR206"));
}

#[test]
fn negative_bulk_max_size_reports_str314() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"s","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"}},
      {"rule-type":"table-settings","rule-id":"2","rule-name":"ts",
       "object-locator":{"schema-name":"s","table-name":"t"},
       "lob-settings":{"mode":"unlimited","bulk-max-size":-10}}
    ]}"#;
    assert!(has(doc, "STR314"));
}

#[test]
fn non_string_rule_name_reports_str018() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":123,
       "rule-action":"include","object-locator":{"schema-name":"s","table-name":"t"}}
    ]}"#;
    assert!(has(doc, "STR018"));
}

#[test]
fn negative_parallel_load_partitions_reports_str315() {
    let doc = r#"{"rules":[
      {"rule-type":"selection","rule-id":"1","rule-name":"s","rule-action":"include",
       "object-locator":{"schema-name":"s","table-name":"t"}},
      {"rule-type":"table-settings","rule-id":"2","rule-name":"ts",
       "object-locator":{"schema-name":"s","table-name":"t"},
       "parallel-load":{"type":"partitions-auto","number-of-partitions":-4}}
    ]}"#;
    assert!(has(doc, "STR315"));
}

#[test]
fn ranges_have_one_based_line_columns() {
    let report = validate("{\n  \"rules\": 5\n}", None);
    let d = report
        .diagnostics
        .iter()
        .find(|d| d.code == "STR002")
        .unwrap();
    let r = d.range.unwrap();
    assert_eq!(r.start_line, 2);
    assert!(r.start_col >= 1);
}

#[test]
fn report_serializes_to_camelcase_json() {
    let report = validate(VALID, Some("3.5.4"));
    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("\"targetDmsVersion\":\"3.5.4\""));
    assert!(json.contains("\"summary\""));
}

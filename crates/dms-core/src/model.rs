//! The diagnostic data model shared with the frontend (SPEC §7).
//!
//! Serialized to camelCase JSON so it can be returned verbatim from a Tauri
//! command and consumed by the TypeScript UI.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: u32,
    pub col: u32,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_type: Option<String>,
}

impl RuleContext {
    pub fn is_empty(&self) -> bool {
        self.rule_id.is_none() && self.rule_name.is_none() && self.rule_type.is_none()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    /// English message (keeps AWS terminology verbatim).
    pub message: String,
    /// Korean explanation.
    pub message_ko: String,
    /// RFC 6901 JSON Pointer to the offending node.
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_context: Option<RuleContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub errors: u32,
    pub warnings: u32,
    pub infos: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub ok: bool,
    pub summary: Summary,
    pub engine_version: String,
    pub target_dms_version: String,
    pub diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    pub fn errors(&self) -> u32 {
        self.summary.errors
    }
}

/// Static metadata for one catalog entry — used by the `list_rules` command so
/// the UI can show a reference table.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleDoc {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub message_ko: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_url: Option<String>,
}

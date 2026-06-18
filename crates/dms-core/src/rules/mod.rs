//! Rule orchestration (SPEC §6). `run` walks the parsed document, dispatches
//! each rule to its type-specific validator, and emits cross-rule diagnostics.

pub mod object_mapping;
pub mod post_processing;
pub mod selection;
pub mod table_settings;
pub mod transformation;
pub mod wildcard;

use crate::context::Ctx;
use crate::enums;
use crate::json::{Node, Span};
use crate::model::RuleContext;
use std::collections::HashMap;

/// A collected identifier occurrence: (normalized value, span, json-pointer).
type Occurrence = (String, Span, String);

pub fn run(ctx: &mut Ctx, root: &Node) {
    duplicate_keys(ctx, root, "");

    let Some(_) = root.as_object() else {
        ctx.report("SYN002", "", root.span(), None);
        return;
    };

    let rules_node = match root.get("rules") {
        None => {
            ctx.report("STR001", "", head(root.span()), None);
            return;
        }
        Some(m) => &m.value,
    };
    let rules = match rules_node.as_array() {
        None => {
            ctx.report("STR002", "/rules", rules_node.span(), None);
            return;
        }
        Some(a) => a,
    };
    if rules.is_empty() {
        ctx.report("STR003", "/rules", rules_node.span(), None);
    }

    let mut ids: Vec<Occurrence> = Vec::new();
    let mut names: Vec<Occurrence> = Vec::new();
    let mut transform_targets: Vec<Occurrence> = Vec::new();
    let mut selection_count = 0usize;
    let mut has_transformation = false;
    let mut first_object_mapping: Option<(String, Span)> = None;

    for (i, rule) in rules.iter().enumerate() {
        let path = format!("/rules/{i}");
        if !rule.is_object() {
            ctx.report("STR010", &path, rule.span(), None);
            continue;
        }

        let rtype = rule.get_value("rule-type").and_then(Node::as_str);
        let rc = RuleContext {
            rule_id: rule.get_value("rule-id").map(id_to_string),
            rule_name: rule
                .get_value("rule-name")
                .and_then(Node::as_str)
                .map(str::to_string),
            rule_type: rtype.map(str::to_string),
        };

        // rule-type
        match rule.get("rule-type") {
            None => ctx.report("STR011", &path, head(rule.span()), Some(rc.clone())),
            Some(m) => {
                if m.value
                    .as_str()
                    .map_or(true, |s| !enums::contains(enums::RULE_TYPES, s))
                {
                    let got = m.value.as_str().unwrap_or(m.value.kind());
                    enum_error(
                        ctx,
                        "STR012",
                        &format!("{path}/rule-type"),
                        m.value.span(),
                        got,
                        enums::RULE_TYPES,
                        Some(rc.clone()),
                    );
                }
            }
        }

        // rule-id — must be a non-negative integer within the supported range
        match rule.get("rule-id") {
            None => ctx.report("STR013", &path, head(rule.span()), Some(rc.clone())),
            Some(m) => {
                let id_path = format!("{path}/rule-id");
                let (kind, cand) = classify_rule_id(&m.value);
                match kind {
                    RuleIdKind::Valid => ids.push((cand, m.value.span(), id_path)),
                    RuleIdKind::Unusual => {
                        // still track for uniqueness, but flag the unusual value
                        ids.push((cand.clone(), m.value.span(), id_path.clone()));
                        ctx.report_detail(
                            "STR017",
                            &id_path,
                            m.value.span(),
                            &format!("(got '{cand}')"),
                            &format!("(받은 값 '{cand}')"),
                            Some(rc.clone()),
                        );
                    }
                    RuleIdKind::Invalid => {
                        ctx.report("STR014", &id_path, m.value.span(), Some(rc.clone()))
                    }
                }
            }
        }

        // rule-name — required string
        match rule.get("rule-name") {
            None => ctx.report("STR015", &path, head(rule.span()), Some(rc.clone())),
            Some(m) => match m.value.as_str() {
                Some(s) => names.push((s.to_string(), m.value.span(), format!("{path}/rule-name"))),
                None => ctx.report(
                    "STR018",
                    &format!("{path}/rule-name"),
                    m.value.span(),
                    Some(rc.clone()),
                ),
            },
        }

        // dispatch
        match rtype {
            Some("selection") => {
                selection_count += 1;
                selection::validate(ctx, rule, &path, &rc);
            }
            Some("transformation") => {
                has_transformation = true;
                transformation::validate(ctx, rule, &path, &rc, &mut transform_targets);
            }
            Some("table-settings") => table_settings::validate(ctx, rule, &path, &rc),
            Some("post-processing") => post_processing::validate(ctx, rule, &path, &rc),
            Some("object-mapping") => {
                if first_object_mapping.is_none() {
                    first_object_mapping = Some((path.clone(), rule.span()));
                }
                object_mapping::validate(ctx, rule, &path, &rc);
            }
            _ => {}
        }
    }

    report_dups(ctx, &ids, "SEM001");
    report_dups(ctx, &names, "SEM002");

    if selection_count == 0 {
        ctx.report("SEM003", "/rules", head(rules_node.span()), None);
    }

    // BP001: multiple transformation rules targeting the same object.
    report_dups_info(ctx, &transform_targets, "BP001");

    // BP500: object-mapping cannot be combined with transformation rules (Kafka).
    if has_transformation {
        if let Some((path, span)) = first_object_mapping {
            ctx.report("BP500", &path, head(span), None);
        }
    }
}

// ---------------------------------------------------------------------------
// shared helpers used by submodules
// ---------------------------------------------------------------------------

/// A 1-character marker span at the start of `span` (used for "missing field"
/// diagnostics so the editor points at the enclosing object's opening brace).
pub fn head(span: Span) -> Span {
    (span.0, span.0 + 1)
}

/// Reports an invalid-enum diagnostic, appending the offending value and the
/// list of allowed values to the catalog message.
pub fn enum_error(
    ctx: &mut Ctx,
    code: &str,
    path: &str,
    span: Span,
    got: &str,
    allowed: &[&str],
    rc: Option<RuleContext>,
) {
    let en = format!("Got '{got}'. Allowed: {}.", enums::quoted_list(allowed));
    let ko = format!("(받은 값 '{got}', 허용: {})", enums::quoted_list(allowed));
    ctx.report_detail(code, path, span, &en, &ko, rc);
}

/// Emits `STR016` for any rule key that is not in the allowed set.
pub fn check_unknown_fields(
    ctx: &mut Ctx,
    rule: &Node,
    path: &str,
    allowed: &[&str],
    rc: &RuleContext,
) {
    if let Some(members) = rule.as_object() {
        for m in members {
            if !allowed.contains(&m.key.as_str()) {
                ctx.report_detail(
                    "STR016",
                    &format!("{path}/{}", escape_ptr(&m.key)),
                    m.key_span,
                    &format!("('{}')", m.key),
                    &format!("('{}')", m.key),
                    Some(rc.clone()),
                );
            }
        }
    }
}

/// RFC 6901 JSON Pointer token escaping.
pub fn escape_ptr(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

/// Integer value of a node that may be a JSON number **or** a numeric string.
/// AWS examples encode numeric fields as strings (e.g. `"length": "4092"`,
/// `"bulk-max-size": "100000"`), so accepting both avoids false positives.
pub fn as_int(n: &Node) -> Option<i64> {
    match n {
        Node::Number(s, _) if !s.contains(['.', 'e', 'E']) => s.parse::<i64>().ok(),
        Node::Str(s, _) => {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                t.parse::<i64>().ok()
            }
        }
        _ => None,
    }
}

enum RuleIdKind {
    /// A small, non-negative integer — the form AWS examples always use.
    Valid,
    /// Numeric but unusual: negative, decimal, or beyond the 32-bit max. AWS only
    /// documents "a unique numeric value" (no max/sign/integer rule), so these are
    /// flagged as warnings rather than hard errors.
    Unusual,
    /// Not numeric at all — violates AWS's documented "numeric value" requirement.
    Invalid,
}

fn classify_rule_id(n: &Node) -> (RuleIdKind, String) {
    let cand = match n {
        Node::Number(s, _) => s.clone(),
        Node::Str(s, _) => s.trim().to_string(),
        _ => return (RuleIdKind::Invalid, String::new()),
    };
    if cand.is_empty() {
        return (RuleIdKind::Invalid, cand);
    }
    if cand.bytes().all(|b| b.is_ascii_digit()) {
        // a clean non-negative integer (if it fits the 32-bit range)
        return match cand.parse::<u64>() {
            Ok(v) if v <= 2_147_483_647 => (RuleIdKind::Valid, cand),
            _ => (RuleIdKind::Unusual, cand), // too large
        };
    }
    // numeric but not a clean non-negative integer (negative / decimal) → unusual;
    // genuinely non-numeric (e.g. "abc") → invalid
    if cand.parse::<f64>().is_ok() {
        (RuleIdKind::Unusual, cand)
    } else {
        (RuleIdKind::Invalid, cand)
    }
}

fn id_to_string(n: &Node) -> String {
    match n {
        Node::Number(s, _) | Node::Str(s, _) => s.clone(),
        _ => String::new(),
    }
}

fn report_dups(ctx: &mut Ctx, items: &[Occurrence], code: &str) {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for (val, _, _) in items {
        *counts.entry(val.as_str()).or_insert(0) += 1;
    }
    for (val, span, path) in items {
        if counts.get(val.as_str()).copied().unwrap_or(0) > 1 {
            ctx.report(code, path, *span, None);
        }
    }
}

fn report_dups_info(ctx: &mut Ctx, items: &[Occurrence], code: &str) {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for (val, _, _) in items {
        *counts.entry(val.as_str()).or_insert(0) += 1;
    }
    // Emit only on the second-and-later occurrences to keep it quiet.
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for (val, span, path) in items {
        let n = seen.entry(val.as_str()).or_insert(0);
        *n += 1;
        if counts.get(val.as_str()).copied().unwrap_or(0) > 1 && *n > 1 {
            ctx.report(code, path, *span, None);
        }
    }
}

/// Recursively flags duplicate keys within any object (SYN003).
fn duplicate_keys(ctx: &mut Ctx, node: &Node, path: &str) {
    match node {
        Node::Object(members, _) => {
            let mut seen: HashMap<&str, usize> = HashMap::new();
            for m in members {
                *seen.entry(m.key.as_str()).or_insert(0) += 1;
            }
            for m in members {
                let child_path = format!("{path}/{}", escape_ptr(&m.key));
                if seen.get(m.key.as_str()).copied().unwrap_or(0) > 1 {
                    ctx.report_detail(
                        "SYN003",
                        &child_path,
                        m.key_span,
                        &format!("('{}')", m.key),
                        &format!("('{}')", m.key),
                        None,
                    );
                }
                duplicate_keys(ctx, &m.value, &child_path);
            }
        }
        Node::Array(items, _) => {
            for (i, item) in items.iter().enumerate() {
                duplicate_keys(ctx, item, &format!("{path}/{i}"));
            }
        }
        _ => {}
    }
}

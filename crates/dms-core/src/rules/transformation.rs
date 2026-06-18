//! `transformation` rule validation (SPEC §6.3) including `data-type`,
//! `primary-key-def`, `before-image-def`, and best-effort expression checks.

use super::wildcard::has_wildcard;
use super::{check_unknown_fields, enum_error, head};
use crate::context::Ctx;
use crate::enums;
use crate::json::{Node, Span};
use crate::model::RuleContext;
use crate::version::Version;

const ALLOWED: &[&str] = &[
    "rule-type",
    "rule-id",
    "rule-name",
    "rule-action",
    "rule-target",
    "object-locator",
    "value",
    "old-value",
    "data-type",
    "expression",
    "primary-key-def",
    "before-image-def",
];

pub fn validate(
    ctx: &mut Ctx,
    rule: &Node,
    path: &str,
    rc: &RuleContext,
    targets: &mut Vec<(String, Span, String)>,
) {
    check_unknown_fields(ctx, rule, path, ALLOWED, rc);

    let action = rule.get_value("rule-action").and_then(Node::as_str);
    let target = rule.get_value("rule-target").and_then(Node::as_str);

    // rule-action
    match rule.get("rule-action") {
        None => ctx.report("STR200", path, head(rule.span()), Some(rc.clone())),
        Some(m) => {
            if action.map_or(true, |a| !enums::contains(enums::TRANSFORMATION_ACTIONS, a)) {
                let got = m.value.as_str().unwrap_or(m.value.kind());
                enum_error(
                    ctx,
                    "STR201",
                    &format!("{path}/rule-action"),
                    m.value.span(),
                    got,
                    enums::TRANSFORMATION_ACTIONS,
                    Some(rc.clone()),
                );
            }
        }
    }

    // rule-target
    match rule.get("rule-target") {
        None => ctx.report("STR202", path, head(rule.span()), Some(rc.clone())),
        Some(m) => {
            if target.map_or(true, |t| !enums::contains(enums::TRANSFORMATION_TARGETS, t)) {
                let got = m.value.as_str().unwrap_or(m.value.kind());
                enum_error(
                    ctx,
                    "STR203",
                    &format!("{path}/rule-target"),
                    m.value.span(),
                    got,
                    enums::TRANSFORMATION_TARGETS,
                    Some(rc.clone()),
                );
            }
        }
    }

    // object-locator
    let locator = match rule.get("object-locator") {
        None => {
            ctx.report("STR204", path, head(rule.span()), Some(rc.clone()));
            None
        }
        Some(m) if !m.value.is_object() => {
            ctx.report(
                "STR204",
                &format!("{path}/object-locator"),
                m.value.span(),
                Some(rc.clone()),
            );
            None
        }
        Some(m) => Some(&m.value),
    };

    // --- action-specific cross-field checks ---
    if let Some(a) = action {
        if (a == "change-data-type" || enums::contains(enums::DATA_MASKING_ACTIONS, a))
            && target != Some("column")
        {
            ctx.report("SEM200", path, head(rule.span()), Some(rc.clone()));
        }
        if a == "add-column" && rule.get("value").is_none() {
            ctx.report("SEM201", path, head(rule.span()), Some(rc.clone()));
        }
        if (a == "replace-prefix" || a == "replace-suffix") && rule.get("old-value").is_none() {
            ctx.report("SEM202", path, head(rule.span()), Some(rc.clone()));
        }
        if a == "define-primary-key" {
            if target != Some("table") {
                ctx.report("SEM204", path, head(rule.span()), Some(rc.clone()));
            }
            if let Some(loc) = locator {
                check_pk_wildcards(ctx, loc, &format!("{path}/object-locator"), rc);
            }
            // primary-key-def is required for this action
            if rule.get("primary-key-def").is_none() {
                ctx.report("STR207", path, head(rule.span()), Some(rc.clone()));
            }
        }
        if a == "add-before-image-columns" && target != Some("column") {
            ctx.report("SEM205", path, head(rule.span()), Some(rc.clone()));
        }
        // data-masking requires DMS engine 3.5.4+ (documented)
        if enums::contains(enums::DATA_MASKING_ACTIONS, a)
            && !ctx.target.at_least(Version::new(3, 5, 4))
        {
            ctx.report("BP202", path, head(rule.span()), Some(rc.clone()));
        }
        // data-masking-digits-mask value must be a single character
        if a == "data-masking-digits-mask" {
            if let Some(m) = rule.get("value") {
                let single = m
                    .value
                    .as_str()
                    .map(|s| s.chars().count() == 1)
                    .unwrap_or(false);
                if !single {
                    ctx.report(
                        "BP204",
                        &format!("{path}/value"),
                        m.value.span(),
                        Some(rc.clone()),
                    );
                }
            }
        }
    }

    // object-locator sub-fields
    if let Some(loc) = locator {
        let loc_path = format!("{path}/object-locator");
        // mutually-exclusive tablespace names
        if loc.get("table-tablespace-name").is_some() && loc.get("index-tablespace-name").is_some()
        {
            ctx.report("SEM206", &loc_path, head(loc.span()), Some(rc.clone()));
        }
        // column target requires column-name
        if target == Some("column") && loc.get("column-name").is_none() {
            ctx.report("SEM208", &loc_path, head(loc.span()), Some(rc.clone()));
        }
        // special-character columns are unsupported (BP200)
        if let Some(col) = loc.get_value("column-name").and_then(Node::as_str) {
            if col.contains(['#', '\\', '/', '-']) {
                ctx.report(
                    "BP200",
                    &format!("{loc_path}/column-name"),
                    loc.get("column-name").map(|m| m.value.span()).unwrap(),
                    Some(rc.clone()),
                );
            }
        }
        // BLOB/CLOB columns only support remove-column (BP201)
        if let Some(dt) = loc.get_value("data-type").and_then(|n| n.get_value("type")) {
            if matches!(dt.as_str(), Some("blob" | "clob" | "nclob"))
                && action != Some("remove-column")
            {
                ctx.report("BP201", &loc_path, head(loc.span()), Some(rc.clone()));
            }
        }

        // track target object for BP001 (lower rule-id wins on collision)
        let key = locator_key(loc, target);
        targets.push((key, rule.span(), path.to_string()));
    }

    // data-type object
    if let Some(m) = rule.get("data-type") {
        validate_data_type(ctx, &m.value, &format!("{path}/data-type"), rc);
    }

    // primary-key-def object
    if let Some(m) = rule.get("primary-key-def") {
        validate_pk_def(ctx, &m.value, &format!("{path}/primary-key-def"), rc);
    }

    // before-image-def object
    if let Some(m) = rule.get("before-image-def") {
        validate_before_image(ctx, &m.value, &format!("{path}/before-image-def"), rc);
    }

    // expression (best-effort)
    if let Some(m) = rule.get("expression") {
        if let Some(expr) = m.value.as_str() {
            validate_expression(ctx, expr, &format!("{path}/expression"), m.value.span(), rc);
        }
    }
}

fn check_pk_wildcards(ctx: &mut Ctx, loc: &Node, loc_path: &str, rc: &RuleContext) {
    for key in ["schema-name", "table-name"] {
        if let Some(m) = loc.get(key) {
            if m.value.as_str().map(has_wildcard).unwrap_or(false) {
                ctx.report(
                    "SEM203",
                    &format!("{loc_path}/{key}"),
                    m.value.span(),
                    Some(rc.clone()),
                );
            }
        }
    }
}

fn validate_data_type(ctx: &mut Ctx, dt: &Node, path: &str, rc: &RuleContext) {
    if let Some(m) = dt.get("type") {
        if m.value
            .as_str()
            .map_or(true, |t| !enums::contains(enums::DATA_TYPES, t))
        {
            let got = m.value.as_str().unwrap_or(m.value.kind());
            enum_error(
                ctx,
                "STR205",
                &format!("{path}/type"),
                m.value.span(),
                got,
                enums::DATA_TYPES,
                Some(rc.clone()),
            );
        }
    }
    for key in ["precision", "scale", "length"] {
        if let Some(m) = dt.get(key) {
            // AWS examples encode these as numbers, numeric strings ("4092"),
            // or an empty string (scale). Reject only negatives / non-numerics.
            let ok = match &m.value {
                Node::Str(s, _) if s.trim().is_empty() => true,
                v => super::as_int(v).map_or(false, |x| x >= 0),
            };
            if !ok {
                ctx.report(
                    "STR206",
                    &format!("{path}/{key}"),
                    m.value.span(),
                    Some(rc.clone()),
                );
            }
        }
    }
}

fn validate_pk_def(ctx: &mut Ctx, pk: &Node, path: &str, rc: &RuleContext) {
    let has_name = pk.get_value("name").and_then(Node::as_str).is_some();
    let has_columns = pk
        .get_value("columns")
        .map(|n| n.as_array().is_some())
        .unwrap_or(false);
    if !has_name || !has_columns {
        ctx.report("STR207", path, head(pk.span()), Some(rc.clone()));
    }
    if let Some(m) = pk.get("origin") {
        if m.value
            .as_str()
            .map_or(true, |o| !enums::contains(enums::PK_ORIGINS, o))
        {
            let got = m.value.as_str().unwrap_or(m.value.kind());
            enum_error(
                ctx,
                "STR208",
                &format!("{path}/origin"),
                m.value.span(),
                got,
                enums::PK_ORIGINS,
                Some(rc.clone()),
            );
        }
    }
}

fn validate_before_image(ctx: &mut Ctx, bi: &Node, path: &str, rc: &RuleContext) {
    if let Some(m) = bi.get("column-filter") {
        if m.value
            .as_str()
            .map_or(true, |c| !enums::contains(enums::COLUMN_FILTERS, c))
        {
            let got = m.value.as_str().unwrap_or(m.value.kind());
            enum_error(
                ctx,
                "STR209",
                &format!("{path}/column-filter"),
                m.value.span(),
                got,
                enums::COLUMN_FILTERS,
                Some(rc.clone()),
            );
        }
    }
    let prefix = bi.get_value("column-prefix").and_then(Node::as_str);
    let suffix = bi.get_value("column-suffix").and_then(Node::as_str);
    if prefix == Some("") && suffix == Some("") {
        ctx.report("SEM207", path, head(bi.span()), Some(rc.clone()));
    }
}

fn validate_expression(ctx: &mut Ctx, expr: &str, path: &str, span: Span, rc: &RuleContext) {
    // BP203: informational note that $-references exist.
    if expr.contains('$') {
        ctx.report("BP203", path, span, Some(rc.clone()));
        // BP210: unknown $AR_H_* / $AR_M_* variable names.
        for var in extract_ar_vars(expr) {
            if !enums::contains(enums::EXPRESSION_VARS, &var) {
                ctx.report_detail(
                    "BP210",
                    path,
                    span,
                    &format!("('${var}')"),
                    &format!("('${var}')"),
                    Some(rc.clone()),
                );
            }
        }
    }
    // BP211: unbalanced parentheses.
    let opens = expr.matches('(').count();
    let closes = expr.matches(')').count();
    if opens != closes {
        ctx.report("BP211", path, span, Some(rc.clone()));
    }
}

/// Pulls `$AR_H_...` / `$AR_M_...` tokens out of an expression string.
fn extract_ar_vars(expr: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = expr.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            let name = &expr[start..j];
            if name.starts_with("AR_H_") || name.starts_with("AR_M_") {
                out.push(name.to_string());
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

/// Normalized key identifying the object a transformation targets.
fn locator_key(loc: &Node, target: Option<&str>) -> String {
    let s = loc
        .get_value("schema-name")
        .and_then(Node::as_str)
        .unwrap_or("");
    let t = loc
        .get_value("table-name")
        .and_then(Node::as_str)
        .unwrap_or("");
    let c = loc
        .get_value("column-name")
        .and_then(Node::as_str)
        .unwrap_or("");
    format!("{}|{s}.{t}.{c}", target.unwrap_or(""))
}

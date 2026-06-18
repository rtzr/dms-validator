//! `table-settings` rule validation (SPEC §6.4).

use super::wildcard::has_wildcard;
use super::{check_unknown_fields, enum_error, head};
use crate::context::Ctx;
use crate::enums;
use crate::json::Node;
use crate::model::RuleContext;

const ALLOWED: &[&str] = &[
    "rule-type",
    "rule-id",
    "rule-name",
    "object-locator",
    "parallel-load",
    "lob-settings",
];

pub fn validate(ctx: &mut Ctx, rule: &Node, path: &str, rc: &RuleContext) {
    check_unknown_fields(ctx, rule, path, ALLOWED, rc);

    match rule.get("object-locator") {
        None => ctx.report("STR300", path, head(rule.span()), Some(rc.clone())),
        Some(m) if !m.value.is_object() => {
            ctx.report(
                "STR300",
                &format!("{path}/object-locator"),
                m.value.span(),
                Some(rc.clone()),
            );
        }
        Some(m) => {
            let loc = &m.value;
            let loc_path = format!("{path}/object-locator");
            if loc.get("schema-name").is_none() || loc.get("table-name").is_none() {
                ctx.report("STR301", &loc_path, head(loc.span()), Some(rc.clone()));
            }
            // no wildcards allowed in table-settings locators
            for key in ["schema-name", "table-name"] {
                if let Some(km) = loc.get(key) {
                    if km.value.as_str().map(has_wildcard).unwrap_or(false) {
                        ctx.report(
                            "SEM300",
                            &format!("{loc_path}/{key}"),
                            km.value.span(),
                            Some(rc.clone()),
                        );
                    }
                }
            }
        }
    }

    if let Some(m) = rule.get("parallel-load") {
        validate_parallel_load(ctx, &m.value, &format!("{path}/parallel-load"), rc);
    }

    if let Some(m) = rule.get("lob-settings") {
        validate_lob_settings(ctx, &m.value, &format!("{path}/lob-settings"), rc);
    }
}

fn validate_parallel_load(ctx: &mut Ctx, pl: &Node, path: &str, rc: &RuleContext) {
    if !pl.is_object() {
        return;
    }
    let pl_type = pl.get_value("type").and_then(Node::as_str);
    if let Some(m) = pl.get("type") {
        if pl_type.map_or(true, |t| !enums::contains(enums::PARALLEL_LOAD_TYPES, t)) {
            let got = m.value.as_str().unwrap_or(m.value.kind());
            enum_error(
                ctx,
                "STR310",
                &format!("{path}/type"),
                m.value.span(),
                got,
                enums::PARALLEL_LOAD_TYPES,
                Some(rc.clone()),
            );
        }
    }

    match pl_type {
        Some("ranges") => {
            let columns = pl.get_value("columns").and_then(Node::as_array);
            let boundaries = pl.get_value("boundaries").and_then(Node::as_array);
            if columns.is_none() || boundaries.is_none() {
                ctx.report("STR311", path, head(pl.span()), Some(rc.clone()));
            }
            if let Some(cols) = columns {
                if cols.len() > 10 {
                    ctx.report(
                        "SEM302",
                        &format!("{path}/columns"),
                        pl.get("columns").map(|m| m.value.span()).unwrap(),
                        Some(rc.clone()),
                    );
                }
                if let Some(bounds) = boundaries {
                    for (i, b) in bounds.iter().enumerate() {
                        if b.as_array().map(|a| a.len()) != Some(cols.len()) {
                            ctx.report(
                                "SEM303",
                                &format!("{path}/boundaries/{i}"),
                                b.span(),
                                Some(rc.clone()),
                            );
                        }
                    }
                }
            }
        }
        Some("partitions-list") => {
            if pl
                .get_value("partitions")
                .and_then(Node::as_array)
                .is_none()
            {
                ctx.report("STR312", path, head(pl.span()), Some(rc.clone()));
            }
        }
        _ => {}
    }

    // Numeric options (MongoDB/DocumentDB partitions-auto) must be non-negative integers.
    for key in [
        "number-of-partitions",
        "max-records-skip-per-page",
        "batch-size",
    ] {
        if let Some(m) = pl.get(key) {
            let bad = super::as_int(&m.value).map_or(true, |v| v < 0);
            if bad {
                ctx.report(
                    "STR315",
                    &format!("{path}/{key}"),
                    m.value.span(),
                    Some(rc.clone()),
                );
            }
        }
    }
}

fn validate_lob_settings(ctx: &mut Ctx, lob: &Node, path: &str, rc: &RuleContext) {
    if !lob.is_object() {
        return;
    }
    let mode = lob.get_value("mode").and_then(Node::as_str);
    if let Some(m) = lob.get("mode") {
        if mode.map_or(true, |x| !enums::contains(enums::LOB_MODES, x)) {
            let got = m.value.as_str().unwrap_or(m.value.kind());
            enum_error(
                ctx,
                "STR313",
                &format!("{path}/mode"),
                m.value.span(),
                got,
                enums::LOB_MODES,
                Some(rc.clone()),
            );
        }
    }

    let bulk = lob.get("bulk-max-size");
    if let Some(m) = bulk {
        // accepts numbers and numeric strings ("100000"); rejects negatives / non-numerics
        let bad = super::as_int(&m.value).map_or(true, |v| v < 0);
        if bad {
            ctx.report(
                "STR314",
                &format!("{path}/bulk-max-size"),
                m.value.span(),
                Some(rc.clone()),
            );
        }
    }

    if mode == Some("limited") {
        let nonzero = bulk
            .and_then(|m| super::as_int(&m.value))
            .map(|v| v != 0)
            .unwrap_or(false);
        if !nonzero {
            ctx.report("SEM301", path, head(lob.span()), Some(rc.clone()));
        }
    }
}

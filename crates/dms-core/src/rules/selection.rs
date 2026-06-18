//! `selection` rule validation (SPEC §6.2) and its `filters` (§6.2.1).

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
    "rule-action",
    "object-locator",
    "load-order",
    "filters",
];

pub fn validate(ctx: &mut Ctx, rule: &Node, path: &str, rc: &RuleContext) {
    check_unknown_fields(ctx, rule, path, ALLOWED, rc);

    // rule-action
    let action = rule.get_value("rule-action").and_then(Node::as_str);
    match rule.get("rule-action") {
        None => ctx.report("STR100", path, head(rule.span()), Some(rc.clone())),
        Some(m) => {
            if action.map_or(true, |a| !enums::contains(enums::SELECTION_ACTIONS, a)) {
                let got = m.value.as_str().unwrap_or(m.value.kind());
                enum_error(
                    ctx,
                    "STR101",
                    &format!("{path}/rule-action"),
                    m.value.span(),
                    got,
                    enums::SELECTION_ACTIONS,
                    Some(rc.clone()),
                );
            }
        }
    }

    // object-locator
    match rule.get("object-locator") {
        None => ctx.report("STR102", path, head(rule.span()), Some(rc.clone())),
        Some(m) if !m.value.is_object() => {
            ctx.report(
                "STR102",
                &format!("{path}/object-locator"),
                m.value.span(),
                Some(rc.clone()),
            );
        }
        Some(m) => {
            let loc = &m.value;
            let loc_path = format!("{path}/object-locator");
            check_locator(ctx, loc, &loc_path, action, rc);
        }
    }

    // load-order
    if let Some(m) = rule.get("load-order") {
        // documented as a positive integer with max 2,147,483,647; AWS encodes it
        // as a number or numeric string.
        let ok = super::as_int(&m.value)
            .map(|v| (1..=2_147_483_647).contains(&v))
            .unwrap_or(false);
        if !ok {
            ctx.report(
                "STR106",
                &format!("{path}/load-order"),
                m.value.span(),
                Some(rc.clone()),
            );
        }
    }

    // filters
    if let Some(m) = rule.get("filters") {
        let filters_path = format!("{path}/filters");
        if action != Some("include") {
            ctx.report("SEM101", &filters_path, m.value.span(), Some(rc.clone()));
        }
        if let Some(items) = m.value.as_array() {
            for (i, f) in items.iter().enumerate() {
                validate_filter(ctx, f, &format!("{filters_path}/{i}"), rc);
            }
        }
    }
}

fn check_locator(
    ctx: &mut Ctx,
    loc: &Node,
    loc_path: &str,
    action: Option<&str>,
    rc: &RuleContext,
) {
    let schema = loc.get("schema-name");
    let table = loc.get("table-name");
    if schema.is_none() {
        ctx.report("STR103", loc_path, head(loc.span()), Some(rc.clone()));
    }
    if table.is_none() {
        ctx.report("STR104", loc_path, head(loc.span()), Some(rc.clone()));
    }

    if let Some(m) = loc.get("table-type") {
        if m.value
            .as_str()
            .map_or(true, |t| !enums::contains(enums::TABLE_TYPES, t))
        {
            let got = m.value.as_str().unwrap_or(m.value.kind());
            enum_error(
                ctx,
                "STR105",
                &format!("{loc_path}/table-type"),
                m.value.span(),
                got,
                enums::TABLE_TYPES,
                Some(rc.clone()),
            );
        }
    }

    // explicit rules cannot use wildcards
    if action == Some("explicit") {
        for (key, member) in [("schema-name", schema), ("table-name", table)] {
            if let Some(m) = member {
                if m.value.as_str().map(has_wildcard).unwrap_or(false) {
                    ctx.report(
                        "SEM100",
                        &format!("{loc_path}/{key}"),
                        m.value.span(),
                        Some(rc.clone()),
                    );
                }
            }
        }
    }
}

fn validate_filter(ctx: &mut Ctx, filter: &Node, path: &str, rc: &RuleContext) {
    if !filter.is_object() {
        ctx.report("STR110", path, filter.span(), Some(rc.clone()));
        return;
    }

    if let Some(m) = filter.get("filter-type") {
        if m.value
            .as_str()
            .map_or(true, |t| !enums::contains(enums::FILTER_TYPES, t))
        {
            let got = m.value.as_str().unwrap_or(m.value.kind());
            enum_error(
                ctx,
                "STR111",
                &format!("{path}/filter-type"),
                m.value.span(),
                got,
                enums::FILTER_TYPES,
                Some(rc.clone()),
            );
        }
    }

    match filter.get("column-name") {
        None => ctx.report("STR112", path, head(filter.span()), Some(rc.clone())),
        Some(m) => {
            if m.value.as_str().map(has_wildcard).unwrap_or(false) {
                ctx.report(
                    "SEM113",
                    &format!("{path}/column-name"),
                    m.value.span(),
                    Some(rc.clone()),
                );
            }
        }
    }

    let conditions = filter.get("filter-conditions");
    match conditions {
        Some(m) if m.value.as_array().map(|a| !a.is_empty()).unwrap_or(false) => {
            let arr = m.value.as_array().unwrap();
            let cond_path = format!("{path}/filter-conditions");
            let mut has_positive = false;
            let mut has_negative = false;
            for (i, cond) in arr.iter().enumerate() {
                if let Some(op) = validate_condition(ctx, cond, &format!("{cond_path}/{i}"), rc) {
                    if enums::contains(enums::POSITIVE_OPERATORS, &op) {
                        has_positive = true;
                    }
                    if enums::contains(enums::NEGATIVE_OPERATORS, &op) {
                        has_negative = true;
                    }
                }
            }
            // BP110: mixing positive + negative operators needs DMS 3.5.0+
            if has_positive
                && has_negative
                && !ctx.target.at_least(crate::version::Version::new(3, 5, 0))
            {
                ctx.report("BP110", &cond_path, head(m.value.span()), Some(rc.clone()));
            }
        }
        _ => ctx.report("STR113", path, head(filter.span()), Some(rc.clone())),
    }
}

/// Validates one filter-condition; returns the operator string if present/valid.
fn validate_condition(ctx: &mut Ctx, cond: &Node, path: &str, rc: &RuleContext) -> Option<String> {
    if !cond.is_object() {
        ctx.report("STR110", path, cond.span(), Some(rc.clone()));
        return None;
    }
    let op_member = match cond.get("filter-operator") {
        None => {
            ctx.report("STR114", path, head(cond.span()), Some(rc.clone()));
            return None;
        }
        Some(m) => m,
    };
    let op = match op_member.value.as_str() {
        Some(o) if enums::contains(enums::FILTER_OPERATORS, o) => o.to_string(),
        _ => {
            let got = op_member.value.as_str().unwrap_or(op_member.value.kind());
            enum_error(
                ctx,
                "STR115",
                &format!("{path}/filter-operator"),
                op_member.value.span(),
                got,
                enums::FILTER_OPERATORS,
                Some(rc.clone()),
            );
            return None;
        }
    };

    let has_value = cond.get("value").is_some();
    let has_start = cond.get("start-value").is_some();
    let has_end = cond.get("end-value").is_some();
    let span = cond.span();

    if enums::contains(enums::RANGE_OPERATORS, &op) {
        if !(has_start && has_end) {
            ctx.report("SEM110", path, head(span), Some(rc.clone()));
        }
    } else if enums::contains(enums::NO_VALUE_OPERATORS, &op) {
        if has_value || has_start || has_end {
            ctx.report("SEM111", path, head(span), Some(rc.clone()));
        }
    } else if enums::contains(enums::SINGLE_VALUE_OPERATORS, &op) && !has_value {
        ctx.report("SEM112", path, head(span), Some(rc.clone()));
    }

    Some(op)
}

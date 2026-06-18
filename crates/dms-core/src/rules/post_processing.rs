//! `post-processing` rule validation (SPEC §6.5). S3-target only.

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
    "value",
    "tag-set",
];

pub fn validate(ctx: &mut Ctx, rule: &Node, path: &str, rc: &RuleContext) {
    check_unknown_fields(ctx, rule, path, ALLOWED, rc);

    match rule.get("rule-action") {
        None => ctx.report("STR400", path, head(rule.span()), Some(rc.clone())),
        Some(m) => {
            if m.value.as_str().map_or(true, |a| {
                !enums::contains(enums::POST_PROCESSING_ACTIONS, a)
            }) {
                let got = m.value.as_str().unwrap_or(m.value.kind());
                enum_error(
                    ctx,
                    "STR400",
                    &format!("{path}/rule-action"),
                    m.value.span(),
                    got,
                    enums::POST_PROCESSING_ACTIONS,
                    Some(rc.clone()),
                );
            }
        }
    }

    // Informational reminder about the S3-only scope.
    ctx.report("BP400", path, head(rule.span()), Some(rc.clone()));
}

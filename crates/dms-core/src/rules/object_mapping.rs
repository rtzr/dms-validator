//! `object-mapping` rule validation (SPEC §6.6). Basic structural depth; deep
//! semantic checks for streaming targets are Phase 2.

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
    "target-table-name",
    "mapping-parameters",
    "kafka-target-topic",
];

pub fn validate(ctx: &mut Ctx, rule: &Node, path: &str, rc: &RuleContext) {
    check_unknown_fields(ctx, rule, path, ALLOWED, rc);

    if let Some(m) = rule.get("rule-action") {
        if m.value
            .as_str()
            .map_or(true, |a| !enums::contains(enums::OBJECT_MAPPING_ACTIONS, a))
        {
            let got = m.value.as_str().unwrap_or(m.value.kind());
            enum_error(
                ctx,
                "STR500",
                &format!("{path}/rule-action"),
                m.value.span(),
                got,
                enums::OBJECT_MAPPING_ACTIONS,
                Some(rc.clone()),
            );
        }
    }

    if rule.get("object-locator").is_none() {
        ctx.report("STR501", path, head(rule.span()), Some(rc.clone()));
    }

    if let Some(m) = rule.get("mapping-parameters") {
        validate_mapping_params(ctx, &m.value, &format!("{path}/mapping-parameters"), rc);
    }
}

fn validate_mapping_params(ctx: &mut Ctx, mp: &Node, path: &str, rc: &RuleContext) {
    if !mp.is_object() {
        return;
    }

    let pk_type = mp.get_value("partition-key-type").and_then(Node::as_str);
    if let Some(m) = mp.get("partition-key-type") {
        if pk_type.map_or(true, |t| !enums::contains(enums::PARTITION_KEY_TYPES, t)) {
            let got = m.value.as_str().unwrap_or(m.value.kind());
            enum_error(
                ctx,
                "STR510",
                &format!("{path}/partition-key-type"),
                m.value.span(),
                got,
                enums::PARTITION_KEY_TYPES,
                Some(rc.clone()),
            );
        }
    }
    if pk_type == Some("attribute-name") && mp.get("partition-key-name").is_none() {
        ctx.report("SEM500", path, head(mp.span()), Some(rc.clone()));
    }

    if let Some(am) = mp.get_value("attribute-mappings").and_then(Node::as_array) {
        let am_path = format!("{path}/attribute-mappings");
        for (i, mapping) in am.iter().enumerate() {
            let mpath = format!("{am_path}/{i}");
            if let Some(m) = mapping.get("attribute-type") {
                if m.value
                    .as_str()
                    .map_or(true, |t| !enums::contains(enums::ATTRIBUTE_TYPES, t))
                {
                    let got = m.value.as_str().unwrap_or(m.value.kind());
                    enum_error(
                        ctx,
                        "STR511",
                        &format!("{mpath}/attribute-type"),
                        m.value.span(),
                        got,
                        enums::ATTRIBUTE_TYPES,
                        Some(rc.clone()),
                    );
                }
            }
            if let Some(m) = mapping.get("attribute-sub-type") {
                if m.value
                    .as_str()
                    .map_or(true, |t| !enums::contains(enums::ATTRIBUTE_SUB_TYPES, t))
                {
                    let got = m.value.as_str().unwrap_or(m.value.kind());
                    enum_error(
                        ctx,
                        "STR512",
                        &format!("{mpath}/attribute-sub-type"),
                        m.value.span(),
                        got,
                        enums::ATTRIBUTE_SUB_TYPES,
                        Some(rc.clone()),
                    );
                }
            }
        }
    }
}

//! `dms-core` — AWS DMS table-mapping JSON validation engine.
//!
//! GUI-independent. The single entry point [`validate`] takes the raw document
//! text and returns a [`ValidationReport`] (SPEC §7) with precise `line:column`
//! ranges for every diagnostic. Reused by both the Tauri app and the CLI.

mod catalog;
mod context;
mod enums;
mod json;
mod line_index;
mod model;
mod rules;
mod version;

pub use model::{
    Diagnostic, Position, Range, RuleContext, RuleDoc, Severity, Summary, ValidationReport,
};
pub use version::Version;

use context::Ctx;

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_DMS_VERSION: &str = "3.5.4";
/// DMS hard limit on the size of a table-mapping document.
pub const MAX_DOC_BYTES: usize = 2 * 1024 * 1024;

/// Validates a DMS table-mapping JSON document.
///
/// `target_dms_version` gates version-specific warnings (SPEC §7.4); when
/// `None`, [`DEFAULT_DMS_VERSION`] is assumed.
pub fn validate(content: &str, target_dms_version: Option<&str>) -> ValidationReport {
    let target_str = target_dms_version
        .map(str::to_string)
        .unwrap_or_else(|| DEFAULT_DMS_VERSION.to_string());
    let target = Version::parse(&target_str);
    let mut ctx = Ctx::new(content, target);

    // L0: size limit (SYN004) — reported but parsing still proceeds.
    if content.len() > MAX_DOC_BYTES {
        ctx.report("SYN004", "", (0, 0), None);
    }

    // L0: parse.
    match json::parse(content) {
        Err(e) => {
            ctx.report_detail(
                "SYN001",
                "",
                (e.offset, e.offset),
                &e.message,
                &e.message,
                None,
            );
        }
        Ok(root) => {
            rules::run(&mut ctx, &root);
        }
    }

    finish(ctx.diags, target_str)
}

/// Returns the full rule catalog (SPEC §6) for the UI reference table.
pub fn list_rules() -> Vec<RuleDoc> {
    catalog::ALL_CODES
        .iter()
        .map(|code| {
            let m = catalog::meta(code);
            RuleDoc {
                code: code.to_string(),
                severity: m.severity,
                message: m.en.to_string(),
                message_ko: m.ko.to_string(),
                doc_url: Some(m.doc_url()),
            }
        })
        .collect()
}

fn finish(mut diagnostics: Vec<Diagnostic>, target_str: String) -> ValidationReport {
    // Stable, useful ordering: by start position, then severity, then code.
    diagnostics.sort_by(|a, b| {
        let ak = a
            .range
            .map(|r| (r.start_line, r.start_col))
            .unwrap_or((0, 0));
        let bk = b
            .range
            .map(|r| (r.start_line, r.start_col))
            .unwrap_or((0, 0));
        ak.cmp(&bk)
            .then(sev_rank(a.severity).cmp(&sev_rank(b.severity)))
            .then(a.code.cmp(&b.code))
    });

    let mut summary = Summary::default();
    for d in &diagnostics {
        match d.severity {
            Severity::Error => summary.errors += 1,
            Severity::Warning => summary.warnings += 1,
            Severity::Info => summary.infos += 1,
        }
    }

    ValidationReport {
        ok: summary.errors == 0,
        summary,
        engine_version: ENGINE_VERSION.to_string(),
        target_dms_version: target_str,
        diagnostics,
    }
}

fn sev_rank(s: Severity) -> u8 {
    match s {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Info => 2,
    }
}

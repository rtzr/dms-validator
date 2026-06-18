//! `dms-validate` — CLI front-end for the `dms-core` validation engine.
//!
//! Usage:
//!   dms-validate [--json] [--dms-version X.Y.Z] <file.json>
//!   cat file.json | dms-validate [--json] [--dms-version X.Y.Z]
//!
//! Exit code is non-zero when the document has validation errors.

use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut json_out = false;
    let mut version: Option<String> = None;
    let mut file: Option<String> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json_out = true,
            "--dms-version" => version = args.next(),
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') => {
                eprintln!("unknown option: {other}");
                return ExitCode::from(2);
            }
            other => file = Some(other.to_string()),
        }
    }

    let content = match read_input(file.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error reading input: {e}");
            return ExitCode::from(2);
        }
    };

    let report = dms_core::validate(&content, version.as_deref());

    if json_out {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
    } else {
        print_human(&report);
    }

    if report.ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn read_input(file: Option<&str>) -> std::io::Result<String> {
    match file {
        Some(path) => std::fs::read_to_string(path),
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

fn print_human(report: &dms_core::ValidationReport) {
    for d in &report.diagnostics {
        let loc = d
            .range
            .map(|r| format!("{}:{}", r.start_line, r.start_col))
            .unwrap_or_default();
        let sev = match d.severity {
            dms_core::Severity::Error => "error",
            dms_core::Severity::Warning => "warn ",
            dms_core::Severity::Info => "info ",
        };
        println!("{sev}  {:<7} {loc:>9}  {}  [{}]", d.code, d.message, d.path);
    }
    let s = &report.summary;
    println!(
        "\n{} error(s), {} warning(s), {} info — target DMS {} — {}",
        s.errors,
        s.warnings,
        s.infos,
        report.target_dms_version,
        if report.ok { "OK" } else { "FAILED" }
    );
}

fn print_help() {
    println!(
        "dms-validate — validate AWS DMS table-mapping JSON\n\n\
         USAGE:\n  dms-validate [--json] [--dms-version X.Y.Z] <file.json>\n  \
         cat file.json | dms-validate [--json]\n\n\
         OPTIONS:\n  --json            emit the full ValidationReport as JSON\n  \
         --dms-version     target DMS engine version for version-gating (default 3.5.4)\n  \
         -h, --help        show this help"
    );
}

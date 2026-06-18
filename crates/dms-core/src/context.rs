//! Validation context: accumulates diagnostics and resolves byte spans into
//! editor ranges via the line index.

use crate::catalog;
use crate::line_index::LineIndex;
use crate::model::{Diagnostic, RuleContext};
use crate::version::Version;

pub struct Ctx<'a> {
    li: LineIndex<'a>,
    pub target: Version,
    pub diags: Vec<Diagnostic>,
}

impl<'a> Ctx<'a> {
    pub fn new(src: &'a str, target: Version) -> Self {
        Ctx {
            li: LineIndex::new(src),
            target,
            diags: Vec::new(),
        }
    }

    fn make(
        &self,
        code: &str,
        path: &str,
        span: (usize, usize),
        en: String,
        ko: String,
        rc: Option<RuleContext>,
    ) -> Diagnostic {
        let meta = catalog::meta(code);
        Diagnostic {
            code: code.to_string(),
            severity: meta.severity,
            message: en,
            message_ko: ko,
            path: path.to_string(),
            range: Some(self.li.range(span)),
            rule_context: rc.filter(|r| !r.is_empty()),
            doc_url: Some(meta.doc_url()),
        }
    }

    /// Report using the catalog's default bilingual message.
    pub fn report(
        &mut self,
        code: &str,
        path: &str,
        span: (usize, usize),
        rc: Option<RuleContext>,
    ) {
        let meta = catalog::meta(code);
        let d = self.make(
            code,
            path,
            span,
            meta.en.to_string(),
            meta.ko.to_string(),
            rc,
        );
        self.diags.push(d);
    }

    /// Report appending an extra detail clause to each default message.
    pub fn report_detail(
        &mut self,
        code: &str,
        path: &str,
        span: (usize, usize),
        en_extra: &str,
        ko_extra: &str,
        rc: Option<RuleContext>,
    ) {
        let meta = catalog::meta(code);
        let en = format!("{} {}", meta.en, en_extra);
        let ko = format!("{} {}", meta.ko, ko_extra);
        let d = self.make(code, path, span, en, ko, rc);
        self.diags.push(d);
    }
}

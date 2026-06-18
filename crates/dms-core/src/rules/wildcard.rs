//! Wildcard detection for "no-wildcard" contexts (SPEC §A.6).
//!
//! DMS treats `%`, `_`, and `[...]` character classes as wildcards. We
//! deliberately do **not** treat a bare `_` as a wildcard here: real schema and
//! table identifiers contain underscores constantly (`my_table`), so flagging
//! every `_` would drown the user in false positives. We flag the unambiguous
//! wildcards (`%` and bracket classes) as hard errors instead.

pub fn has_wildcard(s: &str) -> bool {
    if s.contains('%') {
        return true;
    }
    if let (Some(open), Some(close)) = (s.find('['), s.rfind(']')) {
        if open < close {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_percent_and_brackets() {
        assert!(has_wildcard("HR%"));
        assert!(has_wildcard("emp[0-9]"));
        assert!(has_wildcard("tbl[ab]"));
    }

    #[test]
    fn ignores_plain_underscore() {
        assert!(!has_wildcard("my_table"));
        assert!(!has_wildcard("SCHEMA_1"));
    }
}

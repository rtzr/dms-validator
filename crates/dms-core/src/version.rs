//! Lightweight DMS engine version parsing for version-gating (SPEC §7.4).

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version {
            major,
            minor,
            patch,
        }
    }

    /// Parses strings like `"3.5.4"`, `"3.5"`, `"3.5.4-rc"`. Unknown / malformed
    /// input falls back to the default engine version so gating stays lenient.
    pub fn parse(s: &str) -> Self {
        let cleaned: String = s
            .trim()
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let mut parts = cleaned.split('.').filter(|p| !p.is_empty());
        let major = parts.next().and_then(|p| p.parse().ok());
        match major {
            Some(major) => Version {
                major,
                minor: parts.next().and_then(|p| p.parse().ok()).unwrap_or(0),
                patch: parts.next().and_then(|p| p.parse().ok()).unwrap_or(0),
            },
            None => Version::default_engine(),
        }
    }

    pub fn default_engine() -> Self {
        Version::new(3, 5, 4)
    }

    pub fn at_least(&self, other: Version) -> bool {
        *self >= other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_and_partial() {
        assert_eq!(Version::parse("3.5.4"), Version::new(3, 5, 4));
        assert_eq!(Version::parse("3.5"), Version::new(3, 5, 0));
        assert_eq!(Version::parse("3.4.6-beta"), Version::new(3, 4, 6));
    }

    #[test]
    fn ordering() {
        assert!(Version::new(3, 5, 0).at_least(Version::new(3, 5, 0)));
        assert!(Version::new(3, 5, 1).at_least(Version::new(3, 5, 0)));
        assert!(!Version::new(3, 4, 9).at_least(Version::new(3, 5, 0)));
    }
}

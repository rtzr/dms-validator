//! A minimal, position-preserving JSON parser.
//!
//! The validator needs precise byte spans for every value *and* every object
//! key so diagnostics can point at an exact `line:column` range in the editor.
//! `serde_json` discards that information, so we keep a tiny hand-written
//! recursive-descent parser here. It is strict JSON (no comments, no trailing
//! commas) — exactly what AWS DMS accepts — so anything it rejects is a real
//! `SYN001` syntax error.

/// Inclusive-start / exclusive-end byte offsets into the source text.
pub type Span = (usize, usize);

#[derive(Debug, Clone)]
pub enum Node {
    Null(Span),
    Bool(#[allow(dead_code)] bool, Span),
    /// Raw textual form preserved (e.g. `"3.14"`, `"-5"`).
    Number(String, Span),
    Str(String, Span),
    Array(Vec<Node>, Span),
    Object(Vec<Member>, Span),
}

#[derive(Debug, Clone)]
pub struct Member {
    pub key: String,
    /// Span of the key string literal, including quotes.
    pub key_span: Span,
    pub value: Node,
    /// Span covering key-start .. value-end. Kept for completeness / future use.
    #[allow(dead_code)]
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub offset: usize,
}

impl Node {
    pub fn span(&self) -> Span {
        match self {
            Node::Null(s)
            | Node::Bool(_, s)
            | Node::Number(_, s)
            | Node::Str(_, s)
            | Node::Array(_, s)
            | Node::Object(_, s) => *s,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Node::Null(_) => "null",
            Node::Bool(..) => "boolean",
            Node::Number(..) => "number",
            Node::Str(..) => "string",
            Node::Array(..) => "array",
            Node::Object(..) => "object",
        }
    }

    pub fn as_object(&self) -> Option<&[Member]> {
        match self {
            Node::Object(m, _) => Some(m),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Node]> {
        match self {
            Node::Array(a, _) => Some(a),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Node::Str(s, _) => Some(s),
            _ => None,
        }
    }

    #[allow(dead_code)] // part of the complete JSON value API; exercised by tests
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Node::Bool(b, _) => Some(*b),
            _ => None,
        }
    }

    #[allow(dead_code)] // complete JSON value API
    pub fn as_number_raw(&self) -> Option<&str> {
        match self {
            Node::Number(s, _) => Some(s),
            _ => None,
        }
    }

    #[allow(dead_code)] // part of the complete JSON value API; exercised by tests
    pub fn is_null(&self) -> bool {
        matches!(self, Node::Null(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Node::Object(..))
    }

    /// True when this number is an integer (no fraction / exponent).
    #[allow(dead_code)] // complete JSON value API; numeric coercion now goes through rules::as_int
    pub fn is_integer(&self) -> bool {
        match self {
            Node::Number(s, _) => !s.contains(['.', 'e', 'E']),
            _ => false,
        }
    }

    #[allow(dead_code)] // complete JSON value API
    pub fn as_i64(&self) -> Option<i64> {
        self.as_number_raw().and_then(|s| s.parse::<i64>().ok())
    }

    /// First member matching `key` (DMS keys are unique in practice; duplicate
    /// detection is handled separately by `SYN003`).
    pub fn get(&self, key: &str) -> Option<&Member> {
        self.as_object()
            .and_then(|members| members.iter().find(|m| m.key == key))
    }

    /// Convenience: the value node for `key`, if this is an object.
    pub fn get_value(&self, key: &str) -> Option<&Node> {
        self.get(key).map(|m| &m.value)
    }
}

pub fn parse(input: &str) -> Result<Node, ParseError> {
    let mut p = Parser {
        b: input.as_bytes(),
        pos: 0,
    };
    p.skip_ws();
    let node = p.parse_value()?;
    p.skip_ws();
    if p.pos != p.b.len() {
        return Err(p.err("unexpected trailing characters after the top-level JSON value"));
    }
    Ok(node)
}

struct Parser<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn err(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            offset: self.pos.min(self.b.len()),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.b.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn parse_value(&mut self) -> Result<Node, ParseError> {
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => {
                let (s, span) = self.parse_string_raw()?;
                Ok(Node::Str(s, span))
            }
            Some(b't') | Some(b'f') => self.parse_bool(),
            Some(b'n') => self.parse_null(),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.parse_number(),
            Some(_) => Err(self.err("expected a JSON value")),
            None => Err(self.err("unexpected end of input, expected a JSON value")),
        }
    }

    fn parse_object(&mut self) -> Result<Node, ParseError> {
        let start = self.pos;
        self.pos += 1; // consume '{'
        let mut members = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Node::Object(members, (start, self.pos)));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(self.err("expected a string key in object"));
            }
            let (key, key_span) = self.parse_string_raw()?;
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err(self.err("expected ':' after object key"));
            }
            self.pos += 1; // consume ':'
            self.skip_ws();
            let value = self.parse_value()?;
            let member_span = (key_span.0, value.span().1);
            members.push(Member {
                key,
                key_span,
                value,
                span: member_span,
            });
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    continue;
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(self.err("expected ',' or '}' in object")),
            }
        }
        Ok(Node::Object(members, (start, self.pos)))
    }

    fn parse_array(&mut self) -> Result<Node, ParseError> {
        let start = self.pos;
        self.pos += 1; // consume '['
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Node::Array(items, (start, self.pos)));
        }
        loop {
            self.skip_ws();
            let value = self.parse_value()?;
            items.push(value);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    continue;
                }
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(self.err("expected ',' or ']' in array")),
            }
        }
        Ok(Node::Array(items, (start, self.pos)))
    }

    fn parse_bool(&mut self) -> Result<Node, ParseError> {
        let start = self.pos;
        if self.b[self.pos..].starts_with(b"true") {
            self.pos += 4;
            Ok(Node::Bool(true, (start, self.pos)))
        } else if self.b[self.pos..].starts_with(b"false") {
            self.pos += 5;
            Ok(Node::Bool(false, (start, self.pos)))
        } else {
            Err(self.err("invalid literal, expected 'true' or 'false'"))
        }
    }

    fn parse_null(&mut self) -> Result<Node, ParseError> {
        let start = self.pos;
        if self.b[self.pos..].starts_with(b"null") {
            self.pos += 4;
            Ok(Node::Null((start, self.pos)))
        } else {
            Err(self.err("invalid literal, expected 'null'"))
        }
    }

    fn parse_number(&mut self) -> Result<Node, ParseError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(c) if c.is_ascii_digit() => {
                while matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                    self.pos += 1;
                }
            }
            _ => return Err(self.err("invalid number, expected a digit")),
        }
        // fraction
        if self.peek() == Some(b'.') {
            self.pos += 1;
            if !matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                return Err(self.err("invalid number, expected a digit after '.'"));
            }
            while matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        // exponent
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                return Err(self.err("invalid number, expected a digit in exponent"));
            }
            while matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        let raw = std::str::from_utf8(&self.b[start..self.pos])
            .unwrap_or("")
            .to_string();
        Ok(Node::Number(raw, (start, self.pos)))
    }

    /// Parses a string literal, returning the decoded value and the span
    /// (including the surrounding quotes).
    fn parse_string_raw(&mut self) -> Result<(String, Span), ParseError> {
        let start = self.pos;
        self.pos += 1; // consume opening quote
        let mut out = String::new();
        loop {
            let c = match self.peek() {
                Some(c) => c,
                None => return Err(self.err("unterminated string literal")),
            };
            match c {
                b'"' => {
                    self.pos += 1;
                    return Ok((out, (start, self.pos)));
                }
                b'\\' => {
                    self.pos += 1; // consume the backslash; now at the escape char
                    let esc = self
                        .peek()
                        .ok_or_else(|| self.err("unterminated escape sequence"))?;
                    match esc {
                        b'"' => {
                            out.push('"');
                            self.pos += 1;
                        }
                        b'\\' => {
                            out.push('\\');
                            self.pos += 1;
                        }
                        b'/' => {
                            out.push('/');
                            self.pos += 1;
                        }
                        b'b' => {
                            out.push('\u{08}');
                            self.pos += 1;
                        }
                        b'f' => {
                            out.push('\u{0C}');
                            self.pos += 1;
                        }
                        b'n' => {
                            out.push('\n');
                            self.pos += 1;
                        }
                        b'r' => {
                            out.push('\r');
                            self.pos += 1;
                        }
                        b't' => {
                            out.push('\t');
                            self.pos += 1;
                        }
                        b'u' => {
                            self.pos += 1; // consume 'u'
                            let cp = self.read_hex4()?; // leaves pos after the 4 hex digits
                            if (0xD800..=0xDBFF).contains(&cp) {
                                // high surrogate: expect a following \uXXXX low surrogate
                                if self.b[self.pos..].starts_with(b"\\u") {
                                    self.pos += 2; // consume "\u"
                                    let low = self.read_hex4()?;
                                    if (0xDC00..=0xDFFF).contains(&low) {
                                        let c = 0x10000 + ((cp - 0xD800) << 10) + (low - 0xDC00);
                                        out.push(char::from_u32(c).unwrap_or('\u{FFFD}'));
                                    } else {
                                        out.push('\u{FFFD}');
                                    }
                                } else {
                                    out.push('\u{FFFD}');
                                }
                            } else {
                                out.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                            }
                        }
                        _ => return Err(self.err("invalid escape sequence")),
                    }
                }
                _ => {
                    // Copy a (possibly multi-byte UTF-8) character verbatim.
                    let len = utf8_len(c);
                    let end = (self.pos + len).min(self.b.len());
                    match std::str::from_utf8(&self.b[self.pos..end]) {
                        Ok(s) => out.push_str(s),
                        Err(_) => return Err(self.err("invalid UTF-8 in string literal")),
                    }
                    self.pos = end;
                }
            }
        }
    }

    /// Reads exactly 4 hex digits starting at the current position and leaves
    /// `self.pos` pointing just after them.
    fn read_hex4(&mut self) -> Result<u32, ParseError> {
        if self.pos + 4 > self.b.len() {
            return Err(self.err("incomplete \\u escape"));
        }
        let mut val = 0u32;
        for _ in 0..4 {
            let d = self.b[self.pos];
            let nib = match d {
                b'0'..=b'9' => (d - b'0') as u32,
                b'a'..=b'f' => (d - b'a' + 10) as u32,
                b'A'..=b'F' => (d - b'A' + 10) as u32,
                _ => return Err(self.err("invalid hex digit in \\u escape")),
            };
            val = val * 16 + nib;
            self.pos += 1;
        }
        Ok(val)
    }
}

fn utf8_len(first: u8) -> usize {
    if first < 0x80 {
        1
    } else if first >> 5 == 0b110 {
        2
    } else if first >> 4 == 0b1110 {
        3
    } else if first >> 3 == 0b11110 {
        4
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scalars_and_nesting() {
        let n = parse(r#"{"a": true, "b": null, "c": [1, "x"]}"#).unwrap();
        assert_eq!(n.get_value("a").and_then(Node::as_bool), Some(true));
        assert!(n.get_value("b").unwrap().is_null());
        let c = n.get_value("c").unwrap().as_array().unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c[1].as_str(), Some("x"));
    }

    #[test]
    fn string_escapes_and_unicode() {
        let n = parse(r#"{"k": "line\nbreak é 😀"}"#).unwrap();
        assert_eq!(n.get_value("k").unwrap().as_str(), Some("line\nbreak é 😀"));
    }

    #[test]
    fn reports_offset_on_error() {
        let err = parse("{\n  \"a\": ,\n}").unwrap_err();
        assert!(err.offset > 0);
    }

    #[test]
    fn rejects_trailing_garbage() {
        assert!(parse("{} junk").is_err());
        assert!(parse("[1,2,]").is_err());
    }
}

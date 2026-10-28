//! A data-only Lua table reader. No interpreter, evaluation, or regular-expression conversion.
use crate::Result;

pub const MAX_TEXT: usize = 32 * 1024 * 1024;
const MAX_DEPTH: usize = 128;
const MAX_NODES: usize = 500_000;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Table(Vec<(Key, Value)>),
    String(Vec<u8>),
    Number(String),
    Bool(bool),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Key {
    String(Vec<u8>),
    Number(String),
    Bool(bool),
}

impl Value {
    pub fn get(&self, path: &str) -> Option<&Value> {
        path.split('.').try_fold(self, |node, name| match node {
            Value::Table(entries) => entries.iter().find_map(|(key, v)| {
                matches!(key, Key::String(k) if k == name.as_bytes()).then_some(v)
            }),
            _ => None,
        })
    }

    pub fn get_mut(&mut self, path: &str) -> Option<&mut Value> {
        path.split('.').try_fold(self, |node, name| match node {
            Value::Table(entries) => entries.iter_mut().find_map(|(key, v)| {
                matches!(key, Key::String(k) if k == name.as_bytes()).then_some(v)
            }),
            _ => None,
        })
    }

    pub fn node_count(&self) -> usize {
        match self {
            Self::Table(entries) => 1 + entries.iter().map(|(_, v)| v.node_count()).sum::<usize>(),
            _ => 1,
        }
    }
}

pub fn parse(input: &[u8]) -> Result<Value> {
    if input.len() > MAX_TEXT {
        return Err("Save text exceeds the 32 MiB limit.".into());
    }
    let mut parser = Parser {
        input,
        pos: 0,
        nodes: 0,
    };
    parser.space()?;
    if parser.identifier()? != "return" {
        return parser.fail("Expected 'return' followed by a Lua table");
    }
    let value = parser.value(0)?;
    if !matches!(value, Value::Table(_)) {
        return parser.fail("The save root must be a table");
    }
    parser.space()?;
    parser.eat(b';');
    parser.space()?;
    if parser.pos != input.len() {
        return parser.fail("Unexpected code after the table");
    }
    Ok(value)
}

/// Validate a complete decimal Lua numeric literal; retain its original precision and spelling.
pub fn valid_number(text: &str) -> bool {
    let b = text.as_bytes();
    let mut p = usize::from(b.first().is_some_and(|c| *c == b'-'));
    let start = p;
    while b.get(p).is_some_and(u8::is_ascii_digit) {
        p += 1;
    }
    let mut digits = p - start;
    if b.get(p) == Some(&b'.') {
        p += 1;
        let start = p;
        while b.get(p).is_some_and(u8::is_ascii_digit) {
            p += 1;
        }
        digits += p - start;
    }
    if digits == 0 {
        return false;
    }
    if b.get(p).is_some_and(|c| matches!(c, b'e' | b'E')) {
        p += 1;
        if b.get(p).is_some_and(|c| matches!(c, b'+' | b'-')) {
            p += 1;
        }
        let start = p;
        while b.get(p).is_some_and(u8::is_ascii_digit) {
            p += 1;
        }
        if p == start {
            return false;
        }
    }
    p == b.len() && text.parse::<f64>().is_ok_and(f64::is_finite)
}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
    nodes: usize,
}

impl Parser<'_> {
    fn fail<T>(&self, message: &str) -> Result<T> {
        let line = self.input[..self.pos]
            .iter()
            .filter(|c| **c == b'\n')
            .count()
            + 1;
        Err(format!("{message} (line {line}, byte {}).", self.pos + 1))
    }
    fn eat(&mut self, c: u8) -> bool {
        if self.input.get(self.pos) == Some(&c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, c: u8) -> Result<()> {
        self.space()?;
        if self.eat(c) {
            Ok(())
        } else {
            self.fail(&format!("Expected '{}'", c as char))
        }
    }
    fn long_start(&self) -> Option<usize> {
        if self.input.get(self.pos) != Some(&b'[') {
            return None;
        }
        let mut i = self.pos + 1;
        while self.input.get(i) == Some(&b'=') {
            i += 1;
        }
        (self.input.get(i) == Some(&b'[')).then_some(i - self.pos - 1)
    }
    fn long_string(&mut self, equals: usize) -> Result<Vec<u8>> {
        self.pos += equals + 2;
        self.eat(b'\r');
        self.eat(b'\n');
        let start = self.pos;
        while self.pos < self.input.len() {
            if self.input[self.pos] == b']'
                && self
                    .input
                    .get(self.pos + 1..self.pos + equals + 1)
                    .is_some_and(|v| v.iter().all(|c| *c == b'='))
                && self.input.get(self.pos + equals + 1) == Some(&b']')
            {
                let bytes = self.input[start..self.pos].to_vec();
                self.pos += equals + 2;
                // Lua normalizes newlines in long strings.
                let mut out = Vec::new();
                let mut i = 0;
                while i < bytes.len() {
                    if bytes[i] == b'\r' {
                        out.push(b'\n');
                        i += 1;
                        if bytes.get(i) == Some(&b'\n') {
                            i += 1;
                        }
                    } else {
                        out.push(bytes[i]);
                        i += 1;
                    }
                }
                return Ok(out);
            }
            self.pos += 1;
        }
        self.fail("Unterminated long string or comment")
    }
    fn space(&mut self) -> Result<()> {
        loop {
            while self
                .input
                .get(self.pos)
                .is_some_and(u8::is_ascii_whitespace)
            {
                self.pos += 1;
            }
            if self.input.get(self.pos..self.pos + 2) != Some(b"--") {
                return Ok(());
            }
            self.pos += 2;
            if let Some(n) = self.long_start() {
                self.long_string(n)?;
            } else {
                while self.input.get(self.pos).is_some_and(|c| *c != b'\n') {
                    self.pos += 1;
                }
            }
        }
    }
    fn identifier(&mut self) -> Result<String> {
        self.space()?;
        let start = self.pos;
        if !self
            .input
            .get(self.pos)
            .is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_')
        {
            return self.fail("Expected an identifier");
        }
        self.pos += 1;
        while self
            .input
            .get(self.pos)
            .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_')
        {
            self.pos += 1;
        }
        Ok(String::from_utf8(self.input[start..self.pos].to_vec()).unwrap())
    }
    fn quoted(&mut self) -> Result<Vec<u8>> {
        let quote = self.input[self.pos];
        self.pos += 1;
        let mut out = Vec::new();
        while let Some(&c) = self.input.get(self.pos) {
            self.pos += 1;
            if c == quote {
                return Ok(out);
            }
            if c == b'\n' || c == b'\r' {
                return self.fail("Unescaped newline in string");
            }
            if c != b'\\' {
                out.push(c);
                continue;
            }
            let Some(&e) = self.input.get(self.pos) else {
                return self.fail("Incomplete string escape");
            };
            self.pos += 1;
            match e {
                b'a' => out.push(7),
                b'b' => out.push(8),
                b'f' => out.push(12),
                b'n' => out.push(10),
                b'r' => out.push(13),
                b't' => out.push(9),
                b'v' => out.push(11),
                b'\\' | b'\'' | b'"' => out.push(e),
                b'\n' => out.push(b'\n'),
                b'\r' => {
                    self.eat(b'\n');
                    out.push(b'\n');
                }
                b'z' => {
                    while self
                        .input
                        .get(self.pos)
                        .is_some_and(u8::is_ascii_whitespace)
                    {
                        self.pos += 1;
                    }
                }
                b'x' => {
                    let Some(hex) = self.input.get(self.pos..self.pos + 2) else {
                        return self.fail("Incomplete hex escape");
                    };
                    let value = std::str::from_utf8(hex)
                        .ok()
                        .and_then(|s| u8::from_str_radix(s, 16).ok());
                    let Some(value) = value else {
                        return self.fail("Invalid hex escape");
                    };
                    out.push(value);
                    self.pos += 2;
                }
                b'0'..=b'9' => {
                    let mut value = (e - b'0') as u16;
                    for _ in 0..2 {
                        if let Some(c @ b'0'..=b'9') = self.input.get(self.pos) {
                            value = value * 10 + (c - b'0') as u16;
                            self.pos += 1;
                        } else {
                            break;
                        }
                    }
                    if value > 255 {
                        return self.fail("Decimal escape exceeds 255");
                    }
                    out.push(value as u8);
                }
                _ => return self.fail("Unsupported string escape"),
            }
        }
        self.fail("Unterminated string")
    }
    fn value(&mut self, depth: usize) -> Result<Value> {
        self.space()?;
        self.nodes += 1;
        if depth > MAX_DEPTH || self.nodes > MAX_NODES {
            return self.fail("Save exceeds nesting or node limits");
        }
        match self.input.get(self.pos).copied() {
            Some(b'{') => self.table(depth),
            Some(b'"' | b'\'') => Ok(Value::String(self.quoted()?)),
            Some(b'[') => match self.long_start() {
                Some(n) => Ok(Value::String(self.long_string(n)?)),
                None => self.fail("Expected a long string"),
            },
            Some(b'-' | b'.' | b'0'..=b'9') => {
                let start = self.pos;
                while self.input.get(self.pos).is_some_and(|c| c.is_ascii_digit() || matches!(c, b'-' | b'+' | b'.' | b'e' | b'E')) { self.pos += 1; }
                let number = std::str::from_utf8(&self.input[start..self.pos]).unwrap();
                if !valid_number(number) { return self.fail("Invalid or non-finite number"); }
                Ok(Value::Number(number.into()))
            }
            Some(_) => match self.identifier()?.as_str() {
                "true" => Ok(Value::Bool(true)), "false" => Ok(Value::Bool(false)),
                _ => self.fail("Only tables, strings, finite decimal numbers, and booleans are supported; Lua code is never executed"),
            },
            None => self.fail("Unexpected end of file"),
        }
    }
    fn table(&mut self, depth: usize) -> Result<Value> {
        self.pos += 1;
        let mut entries = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut index = 1;
        loop {
            self.space()?;
            if self.eat(b'}') {
                return Ok(Value::Table(entries));
            }
            let key;
            let value;
            if self.input.get(self.pos) == Some(&b'[') && self.long_start().is_none() {
                self.pos += 1;
                key = match self.value(depth + 1)? {
                    Value::String(v) => Key::String(v),
                    Value::Number(v) => Key::Number(v),
                    Value::Bool(v) => Key::Bool(v),
                    _ => return self.fail("Table keys must be strings, numbers, or booleans"),
                };
                self.expect(b']')?;
                self.expect(b'=')?;
                value = self.value(depth + 1)?;
            } else {
                let saved = self.pos;
                let name = if self
                    .input
                    .get(self.pos)
                    .is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_')
                {
                    Some(self.identifier()?)
                } else {
                    None
                };
                self.space()?;
                if let Some(name) = name.filter(|_| self.eat(b'=')) {
                    if [
                        "and", "break", "do", "else", "elseif", "end", "false", "for", "function",
                        "if", "in", "local", "nil", "not", "or", "repeat", "return", "then",
                        "true", "until", "while",
                    ]
                    .contains(&name.as_str())
                    {
                        return self.fail("Lua keywords cannot be bare table keys");
                    }
                    key = Key::String(name.into_bytes());
                    value = self.value(depth + 1)?;
                } else {
                    self.pos = saved;
                    key = Key::Number(index.to_string());
                    index += 1;
                    value = self.value(depth + 1)?;
                }
            }
            let identity = match &key {
                Key::String(v) => (0, v.clone()),
                Key::Number(n) => {
                    let n = n.parse::<f64>().unwrap();
                    (
                        1,
                        (if n == 0.0 { 0.0 } else { n })
                            .to_bits()
                            .to_le_bytes()
                            .to_vec(),
                    )
                }
                Key::Bool(b) => (2, vec![u8::from(*b)]),
            };
            if !seen.insert(identity) {
                return self.fail("Duplicate table key; refusing ambiguous data");
            }
            entries.push((key, value));
            self.space()?;
            if self.eat(b',') || self.eat(b';') {
                continue;
            }
            if self.input.get(self.pos) != Some(&b'}') {
                return self.fail("Expected ',' or '}'");
            }
        }
    }
}

fn quoted(bytes: &[u8], out: &mut String) {
    out.push('"');
    for &b in bytes {
        match b {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            32..=126 => out.push(b as char),
            _ => out.push_str(&format!("\\{b:03}")),
        }
    }
    out.push('"');
}

pub fn serialize(value: &Value) -> String {
    fn write(value: &Value, depth: usize, out: &mut String) {
        match value {
            Value::String(v) => quoted(v, out),
            Value::Number(v) => out.push_str(v),
            Value::Bool(v) => out.push_str(if *v { "true" } else { "false" }),
            Value::Table(entries) => {
                out.push('{');
                for (key, value) in entries {
                    out.push('\n');
                    out.push_str(&"  ".repeat(depth + 1));
                    out.push('[');
                    match key {
                        Key::String(v) => quoted(v, out),
                        Key::Number(v) => out.push_str(v),
                        Key::Bool(v) => out.push_str(if *v { "true" } else { "false" }),
                    }
                    out.push_str("] = ");
                    write(value, depth + 1, out);
                    out.push(',');
                }
                if !entries.is_empty() {
                    out.push('\n');
                    out.push_str(&"  ".repeat(depth));
                }
                out.push('}');
            }
        }
    }
    let mut out = String::from("return ");
    write(value, 0, &mut out);
    out.push('\n');
    out
}

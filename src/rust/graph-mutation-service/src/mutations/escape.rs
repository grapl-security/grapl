use std::fmt::Formatter;

use grapl_graph_descriptions::{ImmutableIntProp,
                               ImmutableUintProp};

#[derive(Clone, Debug)]
pub struct Escaped(String);

impl std::ops::Deref for Escaped {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl std::fmt::Display for Escaped {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for Escaped {
    fn from(prop: u64) -> Self {
        Self(format!(r#""{}""#, prop))
    }
}

impl From<i64> for Escaped {
    fn from(prop: i64) -> Self {
        Self(format!(r#""{}""#, prop))
    }
}

impl From<&u64> for Escaped {
    fn from(prop: &u64) -> Self {
        Self(format!(r#""{}""#, prop))
    }
}

impl From<&i64> for Escaped {
    fn from(prop: &i64) -> Self {
        Self(format!(r#""{}""#, prop))
    }
}
impl From<ImmutableUintProp> for Escaped {
    fn from(ImmutableUintProp { prop }: ImmutableUintProp) -> Self {
        Self(format!(r#""{}""#, prop))
    }
}

impl From<ImmutableIntProp> for Escaped {
    fn from(ImmutableIntProp { prop }: ImmutableIntProp) -> Self {
        Self(format!(r#""{}""#, prop))
    }
}

pub fn escape_quote(s: &str) -> Escaped {
    // otherwise we need to double quote it

    let mut output = String::with_capacity(s.len());
    output.push('"');

    for c in s.chars() {
        if c == '"' {
            output += "\\\"";
        } else if c == '\\' {
            output += "\\\\";
        } else {
            output.push(c);
        }
    }

    output.push('"');
    Escaped(output)
}

use crate::v1beta1::NodeProperty;
use prost::alloc::fmt::Formatter;

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

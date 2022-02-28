use std::borrow::Cow;

use crate::error::Error;

pub(crate) fn from_zero_or_hex_str(hex_str: &str) -> Result<u64, std::num::ParseIntError> {
    if hex_str == "0" {
        Ok(0_u64)
    } else {
        let hex_str = hex_str.trim_start_matches("0x");
        u64::from_str_radix(hex_str, 16)
    }
}

pub(crate) fn parse_win_guid_str(guid_str: &str) -> Result<uuid::Uuid, uuid::Error> {
    let guid_str = guid_str
        .trim_start_matches(|c| c == '{')
        .trim_end_matches(|c| c == '}');
    uuid::Uuid::parse_str(guid_str)
}

pub(crate) fn unescape_xml(raw: &str) -> Result<Cow<'_, str>, Error> {
    let mut unescaped: Option<String> = None;
    let mut last_end = 0;
    let raw_bytes = raw.as_bytes();

    fn named_entity(name: &str) -> Option<&str> {
        let s = match name {
            "lt" => "<",
            "gt" => ">",
            "amp" => "&",
            "apos" => "'",
            "quot" => "\"",
            _ => return None,
        };
        Some(s)
    }

    fn parse_number(num_str: &str) -> Result<char, Error> {
        let (value, radix) = if let Some(stripped) = num_str.strip_prefix('x') {
            (stripped, 16)
        } else {
            (num_str, 10)
        };

        let code = u32::from_str_radix(value, radix)?;

        match std::char::from_u32(code) {
            Some(c) => Ok(c),
            None => Err(Error::InvalidXmlCharacterReference(num_str.to_string())),
        }
    }

    let mut iter = memchr::memchr2_iter(b'&', b';', raw_bytes);
    while let Some(start) = iter.by_ref().find(|p| raw_bytes[*p] == b'&') {
        match iter.next() {
            Some(end) if raw_bytes[end] == b';' => {
                // If unescaped is None then allocate a new String
                if unescaped.is_none() {
                    unescaped = Some(String::with_capacity(raw.len()))
                }
                let unescaped = unescaped.as_mut().expect("initialized");

                unescaped.push_str(&raw[last_end..start]);

                // search for character correctness
                let pat = &raw[start + 1..end];
                if let Some(s) = named_entity(pat) {
                    unescaped.push_str(s);
                } else if let Some(stripped) = pat.strip_prefix('#') {
                    let num_char = parse_number(stripped)?;
                    unescaped.push(num_char);
                } else {
                    return Err(Error::InvalidXmlCharacterReference(
                        raw[start + 1..end].to_string(),
                    ));
                }

                last_end = end + 1;
            }
            _ => {
                return Err(Error::InvalidXmlCharacterReference(
                    raw[start..raw.len()].to_string(),
                ))
            }
        }
    }

    if let Some(mut unescaped) = unescaped {
        if let Some(raw) = raw.get(last_end..) {
            unescaped.push_str(raw);
        }

        Ok(Cow::Owned(unescaped))
    } else {
        Ok(Cow::Borrowed(raw))
    }
}

macro_rules! next_text_str {
    ($tokenizer:ident) => {{
        let mut result = None;
        while let Some(token) = $tokenizer.next() {
            match token? {
                Token::Text { text } => result = Some(text.as_str()),
                Token::ElementEnd {
                    end: xmlparser::ElementEnd::Close(_, _),
                    ..
                } => break,
                _ => {}
            }
        }
        match result {
            Some(r) => r,
            None => continue,
        }
    }};
}

macro_rules! get_name_attribute {
    ($tokenizer:ident) => {{
        let mut result = None;
        while let Some(token) = $tokenizer.next() {
            match token? {
                Token::Attribute { local, value, .. } => match local.as_str() {
                    "Name" => result = Some(value.as_str()),
                    _ => {}
                },
                Token::ElementEnd { .. } => break,
                _ => {}
            }
        }
        match result {
            Some(r) => r,
            None => continue,
        }
    }};
}

pub(crate) use get_name_attribute;
pub(crate) use next_text_str;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_text() {
        assert_eq!(
            "<test>elemet body</test>",
            unescape_xml("&lt;test&gt;elemet body&lt;/test&gt;").unwrap()
        );
        assert_eq!(";test>", unescape_xml(";test&gt;").unwrap());

        assert_eq!(
            Err(Error::InvalidXmlCharacterReference("&gt".to_string())),
            unescape_xml("test&gt")
        );
    }

    #[test]
    fn escape_ownership() {
        let borrowed = unescape_xml("nothing to escape").unwrap();
        assert_eq!(Cow::Borrowed("nothing to escape"), borrowed);
        let owned = unescape_xml("with test to escape &lt;").unwrap();
        assert_eq!(
            Cow::Owned::<String>("with test to escape <".to_string()),
            owned
        );
    }

    #[test]
    fn escape_xml_entity_ref() {
        assert_eq!("&", unescape_xml("&amp;").unwrap());
        assert_eq!("<", unescape_xml("&lt;").unwrap());
        assert_eq!(">", unescape_xml("&gt;").unwrap());
        assert_eq!("\"", unescape_xml("&quot;").unwrap());
        assert_eq!("'", unescape_xml("&apos;").unwrap());

        assert_eq!(
            Err(Error::InvalidXmlCharacterReference("bogus".to_string())),
            unescape_xml("&bogus;")
        );
    }

    #[test]
    fn escape_xml_numeric_dec_ref() {
        assert_eq!(" ", unescape_xml("&#32;").unwrap());
        assert_eq!("☣", unescape_xml("&#9763;").unwrap());
        assert_eq!("𓂀", unescape_xml("&#77952;").unwrap());

        assert_eq!(
            Err(Error::InvalidXmlCharacterReference("1114112".to_string())),
            unescape_xml("&#1114112;")
        );
    }

    #[test]
    fn escape_xml_numeric_hex_ref() {
        assert_eq!(" ", unescape_xml("&#x20;").unwrap());
        assert_eq!("☣", unescape_xml("&#x2623;").unwrap());
        assert_eq!("𓂀", unescape_xml("&#x13080;").unwrap());

        assert_eq!(
            Err(Error::InvalidXmlCharacterReference("x110000".to_string())),
            unescape_xml("&#x110000;")
        );
    }
}

use std::{
    borrow::Cow,
    str::FromStr,
};

use chrono::{
    DateTime,
    TimeZone,
    Utc,
};
use xmlparser::StrSpan;

use crate::error::{
    Error,
    Result,
};

pub(crate) fn from_zero_or_hex_str(span: &StrSpan) -> Result<u64> {
    let hex_str = span.as_str();

    if hex_str == "0" {
        Ok(0_u64)
    } else {
        let hex_str = hex_str.trim_start_matches("0x");
        u64::from_str_radix(hex_str, 16).map_err(|source| Error::ParseInt {
            value: hex_str.to_string(),
            position: span.start(),
            source,
        })
    }
}

pub(crate) fn parse_win_guid_str(span: &StrSpan) -> Result<uuid::Uuid> {
    let guid_str = span
        .as_str()
        .trim_start_matches(|c| c == '{')
        .trim_end_matches(|c| c == '}');
    uuid::Uuid::parse_str(guid_str).map_err(|source| Error::ParseUuid {
        value: guid_str.to_string(),
        position: span.start(),
        source,
    })
}

pub(crate) fn parse_int<T>(span: &StrSpan) -> Result<T>
where
    T: FromStr<Err = std::num::ParseIntError>,
{
    let value = span.as_str();

    value.parse::<T>().map_err(|source| Error::ParseInt {
        value: value.to_string(),
        position: span.start(),
        source,
    })
}

pub(crate) fn parse_bool(span: &StrSpan) -> Result<bool> {
    let value = span.as_str();

    value.parse().map_err(|source| Error::ParseBool {
        value: value.to_string(),
        position: span.start(),
        source,
    })
}

pub(crate) fn parse_utc(span: &StrSpan) -> Result<DateTime<Utc>> {
    let value = span.as_str();
    value
        .parse::<DateTime<Utc>>()
        .map_err(|source| Error::ParseDateTime {
            value: value.to_string(),
            position: span.start(),
            format: None,
            source,
        })
}

pub(crate) fn parse_utc_from_str(span: &StrSpan, format: &str) -> Result<DateTime<Utc>> {
    let value = span.as_str();

    Utc.datetime_from_str(value, format)
        .map_err(|source| Error::ParseDateTime {
            value: value.to_string(),
            position: span.start(),
            format: Some(format.to_string()),
            source,
        })
}

pub(crate) fn parse_ip_addr(span: &StrSpan) -> Result<std::net::IpAddr> {
    let value = span.as_str();

    value
        .parse::<std::net::IpAddr>()
        .map_err(|source| Error::ParseIpAddress {
            value: value.to_string(),
            position: span.start(),
            source,
        })
}

pub(crate) fn unescape_xml<'a, 'b: 'a>(span: &'a StrSpan<'b>) -> Result<Cow<'b, str>> {
    let mut unescaped: Option<String> = None;
    let mut last_end = 0;
    let raw = span.as_str();
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

    let parse_number = |num_str: &str| -> Result<char> {
        let (value, radix) = if let Some(stripped) = num_str.strip_prefix('x') {
            (stripped, 16)
        } else {
            (num_str, 10)
        };

        let code = u32::from_str_radix(value, radix).map_err(|source| Error::ParseInt {
            value: value.to_string(),
            position: span.start(),
            source,
        })?;

        match std::char::from_u32(code) {
            Some(c) => Ok(c),
            None => Err(Error::InvalidXmlCharacterReference(num_str.to_string())),
        }
    };

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

macro_rules! next_text_str_span {
    ($tokenizer:ident) => {{
        let mut result = None;
        while let Some(token) = $tokenizer.next() {
            match token? {
                Token::Text { text } => result = Some(text),
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
pub(crate) use next_text_str_span;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_text() -> Result<()> {
        assert_eq!(
            "<test>elemet body</test>",
            unescape_xml(&StrSpan::from("&lt;test&gt;elemet body&lt;/test&gt;"))?
        );
        assert_eq!(";test>", unescape_xml(&StrSpan::from(";test&gt;"))?);

        assert_eq!(
            Err(Error::InvalidXmlCharacterReference("&gt".to_string())),
            unescape_xml(&StrSpan::from("test&gt"))
        );

        Ok(())
    }

    #[test]
    fn escape_ownership() -> Result<()> {
        assert_eq!(
            Cow::Borrowed("nothing to escape"),
            unescape_xml(&StrSpan::from("nothing to escape"))?
        );
        assert_eq!(
            Cow::Owned::<String>("with test to escape <".to_string()),
            unescape_xml(&StrSpan::from("with test to escape &lt;"))?
        );

        Ok(())
    }

    #[test]
    fn escape_xml_entity_ref() -> Result<()> {
        assert_eq!("&", unescape_xml(&StrSpan::from("&amp;"))?);
        assert_eq!("<", unescape_xml(&StrSpan::from("&lt;"))?);
        assert_eq!(">", unescape_xml(&StrSpan::from("&gt;"))?);
        assert_eq!("\"", unescape_xml(&StrSpan::from("&quot;"))?);
        assert_eq!("'", unescape_xml(&StrSpan::from("&apos;"))?);

        assert_eq!(
            Err(Error::InvalidXmlCharacterReference("bogus".to_string())),
            unescape_xml(&StrSpan::from("&bogus;"))
        );

        Ok(())
    }

    #[test]
    fn escape_xml_numeric_dec_ref() -> Result<()> {
        assert_eq!(" ", unescape_xml(&StrSpan::from("&#32;"))?);
        assert_eq!("☣", unescape_xml(&StrSpan::from("&#9763;"))?);
        assert_eq!("𓂀", unescape_xml(&StrSpan::from("&#77952;"))?);

        assert_eq!(
            Err(Error::InvalidXmlCharacterReference("1114112".to_string())),
            unescape_xml(&StrSpan::from("&#1114112;"))
        );

        Ok(())
    }

    #[test]
    fn escape_xml_numeric_hex_ref() -> Result<()> {
        assert_eq!(" ", unescape_xml(&StrSpan::from("&#x20;"))?);
        assert_eq!("☣", unescape_xml(&StrSpan::from("&#x2623;"))?);
        assert_eq!("𓂀", unescape_xml(&StrSpan::from("&#x13080;"))?);

        assert_eq!(
            Err(Error::InvalidXmlCharacterReference("x110000".to_string())),
            unescape_xml(&StrSpan::from("&#x110000;"))
        );

        Ok(())
    }
}

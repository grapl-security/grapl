use std::{
    borrow::Cow,
    str::FromStr,
};

use chrono::{
    DateTime,
    TimeZone,
    Utc,
};
use xmlparser::{
    StrSpan,
    Token,
};

use crate::error::{
    Error,
    Result,
};

mod eventdata_iterator;
pub(crate) use eventdata_iterator::EventDataIterator;

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

fn get_token_position(token: &Token) -> usize {
    match token {
        Token::Attribute { span, .. }
        | Token::Cdata { span, .. }
        | Token::Comment { span, .. }
        | Token::Declaration { span, .. }
        | Token::DtdEnd { span, .. }
        | Token::DtdStart { span, .. }
        | Token::ElementEnd { span, .. }
        | Token::ElementStart { span, .. }
        | Token::EmptyDtd { span, .. }
        | Token::EntityDeclaration { span, .. }
        | Token::ProcessingInstruction { span, .. } => span.start(),
        Token::Text { text } => text.start(),
    }
}

fn display_token<'a>(token: &'a Token) -> &'a str {
    match token {
        Token::Attribute { span, .. }
        | Token::Cdata { span, .. }
        | Token::Comment { span, .. }
        | Token::Declaration { span, .. }
        | Token::DtdEnd { span, .. }
        | Token::DtdStart { span, .. }
        | Token::ElementEnd { span, .. }
        | Token::ElementStart { span, .. }
        | Token::EmptyDtd { span, .. }
        | Token::EntityDeclaration { span, .. }
        | Token::ProcessingInstruction { span, .. } => span.as_str(),
        Token::Text { text } => text.as_str(),
    }
}

macro_rules! expect_next_token {
    ($tokenizer:ident) => {{
        if let Some(token) = $tokenizer.next() {
            token?
        } else {
            return Err(Error::UnexpectedEndOfStream);
        }
    }};
}

/// Can be used to retrieve XML text from an element that does not include any attributes or other
/// elements.
///
/// To be called after the tokenizer reaches the start of an element.
///
/// Ex: `>some text</Element>` returns the StrSpan for `some text`.
pub(crate) fn get_element_text<'a>(
    tokenizer: &mut xmlparser::Tokenizer<'a>,
    element_name: &str,
) -> Result<Option<StrSpan<'a>>> {
    let token = expect_next_token!(tokenizer);
    match token {
        // empty-element tag: Ex: `<Element />`
        xmlparser::Token::ElementEnd {
            end: xmlparser::ElementEnd::Empty,
            ..
        } => {
            // Sysmon will always return Data elements for event fields that do not have values
            // (such as for RuleName when rule names are not specified in the config), so we'll
            // interpret this None instead of an empty string because we'll treat it the same.
            Ok(None)
        }
        // element is open-tag: Ex: `<Element>`
        xmlparser::Token::ElementEnd {
            end: xmlparser::ElementEnd::Open,
            ..
        } => {
            let token = expect_next_token!(tokenizer);
            match token {
                xmlparser::Token::ElementEnd {
                    end: xmlparser::ElementEnd::Close(_, tagname),
                    ..
                } if tagname.as_str() == element_name => {
                    // In XML this is the same as the empty element <Element />, so we return None
                    // for the same reasons noted for the element-empty case.
                    Ok(None)
                }
                xmlparser::Token::Text { text } => {
                    // before returning advance the tokenizer to the end-tag
                    let token = expect_next_token!(tokenizer);
                    match token {
                        xmlparser::Token::ElementEnd {
                            end: xmlparser::ElementEnd::Close(_, tagname),
                            ..
                        } if tagname.as_str() == element_name => {}
                        _ => {
                            return Err(Error::ParseSysmon {
                                message: format!(
                                    "expected '</{}>', found: {}",
                                    element_name,
                                    display_token(&token)
                                ),
                                position: get_token_position(&token),
                            })
                        }
                    }

                    Ok(Some(text))
                }
                _ => Err(Error::ParseSysmon {
                    message: format!(
                        "expected XML text or '</{}>', found: {}",
                        element_name,
                        display_token(&token)
                    ),
                    position: get_token_position(&token),
                }),
            }
        }
        _ => Err(Error::ParseSysmon {
            message: format!("expected '>' or '/>', found: {}", display_token(&token)),
            position: get_token_position(&token),
        }),
    }
}

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

    #[test]
    fn get_element_text() -> Result<()> {
        let xml = r#"<Foo>Bar</Foo>"#;
        let mut tokenizer = xmlparser::Tokenizer::from(xml);

        // consume start-tag
        tokenizer.next().unwrap()?;

        let text = super::get_element_text(&mut tokenizer, "Foo")?.map(|s| s.as_str());
        assert_eq!(text, Some("Bar"));

        Ok(())
    }

    #[test]
    fn get_element_text_empty() -> Result<()> {
        let xml = r#"<Foo></Foo>"#;
        let mut tokenizer = xmlparser::Tokenizer::from(xml);

        // consume start-tag
        tokenizer.next().unwrap()?;

        let text = super::get_element_text(&mut tokenizer, "Foo")?.map(|s| s.as_str());
        assert_eq!(text, None);

        Ok(())
    }

    #[test]
    fn get_element_text_empty2() -> Result<()> {
        let xml = r#"<Foo/>"#;
        let mut tokenizer = xmlparser::Tokenizer::from(xml);

        // consume start-tag
        tokenizer.next().unwrap()?;

        let text = super::get_element_text(&mut tokenizer, "Foo")?.map(|s| s.as_str());
        assert_eq!(text, None);

        Ok(())
    }

    #[test]
    fn get_element_text_empty_err() -> Result<()> {
        let xml = r#"<Foo>Bar</Baz>"#;
        let mut tokenizer = xmlparser::Tokenizer::from(xml);

        // consume start-tag
        tokenizer.next().unwrap()?;

        let text = super::get_element_text(&mut tokenizer, "Foo").unwrap_err();
        assert!(matches!(text, Error::ParseSysmon { position: 8, .. }));

        let xml = r#"<Foo>Bar<Baz>"#;
        let mut tokenizer = xmlparser::Tokenizer::from(xml);

        // consume start-tag
        tokenizer.next().unwrap()?;

        let text = super::get_element_text(&mut tokenizer, "Foo").unwrap_err();
        assert!(matches!(text, Error::ParseSysmon { position: 8, .. }));

        let xml = r#"<Foo>Bar"#;
        let mut tokenizer = xmlparser::Tokenizer::from(xml);

        // consume start-tag
        tokenizer.next().unwrap()?;

        let text = super::get_element_text(&mut tokenizer, "Foo").unwrap_err();
        assert!(matches!(text, Error::UnexpectedEndOfStream));

        Ok(())
    }
}

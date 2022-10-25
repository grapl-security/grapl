use std::iter::FusedIterator;

use crate::error::{
    Error,
    Result,
};

macro_rules! next_token {
    ($self:ident) => {{
        if let Some(token) = $self.tokenizer.next() {
            match token {
                Ok(token) => token,
                Err(e) => {
                    $self.previous_error = true;

                    return Some(Err(e.into()));
                }
            }
        } else {
            return None;
        }
    }};
}

pub struct EventDataIterator<'a, 'input: 'a> {
    previous_error: bool,
    tokenizer: &'a mut xmlparser::Tokenizer<'input>,
}

impl<'a, 'input: 'a> EventDataIterator<'a, 'input> {
    /// Creates an iterator for walking Data elements under an EventData element.
    ///
    /// This starts by scanning for `<EventData>`. If none is found then an error is returned. From
    /// there on, each iteration returns the key/value pairs from the Data elements.
    ///
    /// If an error occurs, the next iteration should return None, and the iterator should be
    /// fused.
    pub(crate) fn new(tokenizer: &'a mut xmlparser::Tokenizer<'input>) -> Result<Self> {
        // advance tokenizer until `<EventData>` is found
        for token in tokenizer.by_ref() {
            match token? {
                xmlparser::Token::ElementStart { local, .. } if local.as_str() == "EventData" => {
                    break
                }
                _ => {}
            }
        }

        // advance tokenizer to consume end-tag for EventData
        let token = if let Some(token) = tokenizer.next() {
            token?
        } else {
            return Err(Error::UnexpectedEndOfStream);
        };

        match token {
            xmlparser::Token::ElementEnd {
                end: xmlparser::ElementEnd::Open,
                ..
            } => {}
            _ => {
                return Err(Error::ParseSysmon {
                    message: "expected end-tag for EventData".to_string(),
                    position: super::get_token_position(&token),
                })
            }
        }

        Ok(EventDataIterator {
            previous_error: false,
            tokenizer,
        })
    }
}

impl<'a, 'input: 'a> Iterator for EventDataIterator<'a, 'input> {
    type Item = Result<(&'input str, xmlparser::StrSpan<'input>)>;

    fn next(&mut self) -> Option<Self::Item> {
        // If the last item was an error short-circuit and fuse
        if self.previous_error {
            return None;
        }

        // search for next Data element or `</EventData>`
        loop {
            match next_token!(self) {
                xmlparser::Token::ElementStart { local, .. } if local.as_str() == "Data" => break,
                xmlparser::Token::ElementEnd {
                    // we've reached the end
                    end: xmlparser::ElementEnd::Close(_, name),
                    ..
                } if name.as_str() == "EventData" => {
                    return None;
                }
                _ => {}
            }
        }

        // we've found `<Data `
        // get value of the Name attribute
        let token = next_token!(self);
        let name = match token {
            xmlparser::Token::Attribute { local, value, .. } if local.as_str() == "Name" => {
                value.as_str()
            }
            _ => {
                self.previous_error = true;

                return Some(Err(Error::ParseSysmon {
                    message: "expected XML attribute `Name`".to_string(),
                    position: super::get_token_position(&token),
                }));
            }
        };

        let text = match super::get_element_text(self.tokenizer, "Data") {
            Ok(text) => text,
            Err(e) => {
                self.previous_error = true;

                return Some(Err(e));
            }
        };

        // If there's no text to process upstream then just skip it
        if let Some(text) = text {
            Some(Ok((name, text)))
        } else {
            self.next()
        }
    }
}

impl FusedIterator for EventDataIterator<'_, '_> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_data_iterator() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let input = r#"
        <EventData>

          <!-- comment -->
          <Data Name='Foo'>Bar</Data>
          <Data Name='Quux'></Data>
          <Data Name='Quuz' />
          <Data Name='Baz'>Qux</Data>
        </EventData>"#;
        let mut tokenizer = xmlparser::Tokenizer::from(input);

        let mut iterator = EventDataIterator::new(&mut tokenizer)?;

        let (foo, bar) = iterator.next().unwrap()?;
        assert_eq!(foo, "Foo");
        assert_eq!(bar.as_str(), "Bar");

        let (baz, qux) = iterator.next().unwrap()?;
        assert_eq!(baz, "Baz");
        assert_eq!(qux.as_str(), "Qux");

        assert_eq!(iterator.next(), None);
        assert_eq!(iterator.next(), None);

        Ok(())
    }

    #[test]
    fn event_data_iterator_err() -> std::result::Result<(), Box<dyn std::error::Error>> {
        // missing expected <EventData>
        let input = r#"
          <Data Name='Foo'>Bar</Data>
        </EventData>"#;
        let mut tokenizer = xmlparser::Tokenizer::from_fragment(
            input,
            std::ops::Range {
                start: 0,
                end: input.len(),
            },
        );

        let iterator = EventDataIterator::new(&mut tokenizer);

        assert!(matches!(iterator, Err(Error::UnexpectedEndOfStream)));

        // test fuse
        let input = r#"
        <EventData>
          <Data Name='Foo'>Bar</Data>
          <Data Name='Foo'>Bar</BROKEN>
          <Data Name='Baz'>Qux</Data>
        </EventData>"#;
        let mut tokenizer = xmlparser::Tokenizer::from_fragment(
            input,
            std::ops::Range {
                start: 0,
                end: input.len(),
            },
        );

        let mut iterator = EventDataIterator::new(&mut tokenizer)?;

        let (foo, bar) = iterator.next().unwrap()?;
        assert_eq!(foo, "Foo");
        assert_eq!(bar.as_str(), "Bar");
        assert!(matches!(
            iterator.next(),
            Some(Err(Error::ParseSysmon { .. }))
        ));
        assert_eq!(iterator.next(), None);
        assert_eq!(iterator.next(), None);

        Ok(())
    }
}

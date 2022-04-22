use std::iter::FusedIterator;

use super::event::SysmonEvent;
use crate::error::Result;

/// An iterator over results of parsed Sysmon XML events.
///
/// This is created by calling [`sysmon_parser::parse_events`]. See its documentation for more
/// information.
pub struct SysmonEvents<'a> {
    previous_error: bool,
    tokenizer: xmlparser::Tokenizer<'a>,
}

impl<'a> SysmonEvents<'a> {
    pub(super) fn from(input: &'a str) -> Self {
        SysmonEvents {
            previous_error: false,
            tokenizer: xmlparser::Tokenizer::from_fragment(
                input,
                std::ops::Range {
                    start: 0,
                    end: input.len(),
                },
            ),
        }
    }
}

impl<'a> Iterator for SysmonEvents<'a> {
    type Item = Result<SysmonEvent<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.previous_error {
            return None;
        }
        let result_new = super::event::from_tokenizer(&mut self.tokenizer);

        match result_new {
            Ok(_) => Some(result_new),
            Err(crate::error::Error::SysmonEventNotFound) => {
                // This is expected when there's nothing left to process

                // This setting is effectively a no-op because we impl FusedIterator
                // but we'll do it for kicks in case someone removes that.
                self.previous_error = true;

                None
            }
            Err(e) => {
                self.previous_error = true;

                Some(Err(e))
            }
        }
    }
}

impl FusedIterator for SysmonEvents<'_> {}

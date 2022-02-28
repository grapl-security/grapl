use std::iter::FusedIterator;

use super::event::SysmonEvent;
use crate::error::Result;

/// An iterator over results of parsed Sysmon XML events.
///
/// This is created by calling [`sysmon_parser::parse_events`]. See its documentation for more
/// information.
pub struct SysmonEvents<'a> {
    input: &'a str,
    len: usize,
    cursor: usize,
}

impl<'a> SysmonEvents<'a> {
    pub(super) fn new(input: &'a str) -> Self {
        SysmonEvents {
            input,
            len: input.len(),
            cursor: 0,
        }
    }
}

// This is close to being functionally equivelent to
// `input.split_inclusive("</Event>").map(SysmonEvent::try_from)` but it handles trailing data,
// including whitespace in a way that won't result in errors
impl<'a> Iterator for SysmonEvents<'a> {
    type Item = Result<SysmonEvent<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let haystack = &self.input[self.cursor..self.len];
        let needle = b"</Event>";

        let element_end_pos = if let Some(pos) = memchr::memmem::find(haystack.as_bytes(), needle) {
            // add needle length here to have this position cover the end element
            pos + needle.len()
        } else {
            return None;
        };

        let element_substr = &haystack[0..element_end_pos];
        let result = SysmonEvent::from_str(element_substr);

        self.cursor += element_end_pos;

        Some(result)
    }
}

impl FusedIterator for SysmonEvents<'_> {}

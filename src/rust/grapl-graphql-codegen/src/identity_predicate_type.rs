use graphql_parser::schema::Directive;

/// Identity Algorithms take various parameters, and the IdentityPreidcateType
/// represents which of those paramters a given field may be
#[derive(Debug)]
pub enum IdentityPredicateType {
    SessionPseudoKey,
    SessionCreateTime,
    SessionLastSeenTime,
    SessionTerminateTime,
    StaticId,
}

use crate::constants::{
    CREATE_TIME,
    LAST_SEEN_TIME,
    PSEUDO_KEY,
    STATIC_ID,
    TERMINATE_TIME,
};

impl IdentityPredicateType {
    pub fn opt_from<'a>(directive: &Directive<'a, &'a str>) -> Option<Self> {
        match directive.name {
            PSEUDO_KEY => Some(Self::SessionPseudoKey),
            CREATE_TIME => Some(Self::SessionCreateTime),
            LAST_SEEN_TIME => Some(Self::SessionLastSeenTime),
            TERMINATE_TIME => Some(Self::SessionTerminateTime),
            STATIC_ID => Some(Self::StaticId),
            _ => None,
        }
    }
}

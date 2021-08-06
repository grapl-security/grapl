use std::convert::TryFrom;

use graphql_parser::schema::Directive;

use crate::{
    constants::{
        CREATE_TIME,
        DECREMENT_ONLY,
        IMMUTABLE,
        INCREMENT_ONLY,
        LAST_SEEN_TIME,
        PSEUDO_KEY,
        STATIC_ID,
        TERMINATE_TIME,
    },
    errors::CodeGenError,
};

/// ConflictResolution represents how, given two instances of the same predicate, those
/// predicates should be merged together.
#[derive(Clone, Copy, Debug)]
pub enum ConflictResolution {
    /// Immutable can be thought of as a "pick any value", though the most common implementation
    /// is a "First Write Wins".
    Immutable,
    /// Given two values, choose the larger of thet two
    IncrementOnly,
    /// Given two values, choose the lesser of thet two
    DecrementOnly,
}

impl ConflictResolution {
    pub fn implies_cacheable(&self) -> bool {
        match self {
            Self::Immutable => true,
            Self::IncrementOnly => false,
            Self::DecrementOnly => false,
        }
    }

    pub fn from_directive<'a>(directive: &Directive<'a, &'a str>) -> Option<Self> {
        match directive.name {
            PSEUDO_KEY => Some(ConflictResolution::Immutable),
            STATIC_ID => Some(ConflictResolution::Immutable),
            CREATE_TIME => Some(ConflictResolution::Immutable),
            LAST_SEEN_TIME => Some(ConflictResolution::IncrementOnly),
            TERMINATE_TIME => Some(ConflictResolution::Immutable),
            INCREMENT_ONLY => Some(ConflictResolution::IncrementOnly),
            DECREMENT_ONLY => Some(ConflictResolution::DecrementOnly),
            IMMUTABLE => Some(ConflictResolution::Immutable),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implies_cacheable() {
        assert!(ConflictResolution::Immutable.implies_cacheable());
        assert!(!ConflictResolution::IncrementOnly.implies_cacheable());
        assert!(!ConflictResolution::DecrementOnly.implies_cacheable());
    }
}

impl<'a> TryFrom<&[Directive<'a, &'a str>]> for ConflictResolution {
    type Error = CodeGenError<'a>;

    fn try_from(directives: &[Directive<'a, &'a str>]) -> Result<Self, Self::Error> {
        directives
            .iter()
            .find_map(ConflictResolution::from_directive)
            .ok_or_else(|| CodeGenError::UnsupportedConflictResolution {
                directives: directives.to_vec(),
            })
    }
}

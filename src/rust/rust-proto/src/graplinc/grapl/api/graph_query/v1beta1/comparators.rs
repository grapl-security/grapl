use super::messages::{
    AndStringFilters,
    IntFilter,
    IntOperation,
    OrStringFilters,
    StringFilter,
    StringOperation,
};

// Higher level helper for &str
#[derive(Clone, Debug)]
pub enum StrCmp<'a> {
    Eq(&'a str, bool),
    Contains(&'a str, bool),
    Has,
}

impl<'a> StrCmp<'a> {
    pub fn eq(value: &'a str, negated: bool) -> Self {
        StrCmp::Eq(value, negated)
    }
}

impl<'a> From<&'a StringFilter> for StrCmp<'a> {
    fn from(string_filter: &'a StringFilter) -> StrCmp<'a> {
        match string_filter.operation {
            StringOperation::Has => StrCmp::Has,
            StringOperation::Equal => {
                StrCmp::Eq(string_filter.value.as_str(), string_filter.negated)
            }
            StringOperation::Contains => {
                StrCmp::Contains(string_filter.value.as_str(), string_filter.negated)
            }
            StringOperation::Regex => {
                // todo: We don't currently support Regex, but we should
                unimplemented!()
            }
        }
    }
}

// Higher level helper for String (not &str)
#[derive(Clone, Debug)]
pub enum StringCmp {
    Eq(String, bool),
    Contains(String, bool),
    Has,
}

impl StringCmp {
    pub fn eq(value: impl Into<String>, negated: bool) -> Self {
        StringCmp::Eq(value.into(), negated)
    }
}

impl From<StringFilter> for StringCmp {
    fn from(string_filter: StringFilter) -> StringCmp {
        match string_filter.operation {
            StringOperation::Has => StringCmp::Has,
            StringOperation::Equal => StringCmp::Eq(string_filter.value, string_filter.negated),
            StringOperation::Contains => {
                StringCmp::Contains(string_filter.value, string_filter.negated)
            }
            StringOperation::Regex => {
                // todo: We don't currently support Regex, but we should
                unimplemented!()
            }
        }
    }
}

impl From<StringCmp> for StringFilter {
    fn from(string_cmp: StringCmp) -> StringFilter {
        match string_cmp {
            StringCmp::Has => StringFilter {
                operation: StringOperation::Has,
                value: "".to_string(),
                negated: false,
            },
            StringCmp::Eq(value, negated) => StringFilter {
                operation: StringOperation::Equal,
                value,
                negated,
            },
            StringCmp::Contains(value, negated) => StringFilter {
                operation: StringOperation::Contains,
                value,
                negated,
            },
        }
    }
}
impl From<Vec<StringCmp>> for AndStringFilters {
    fn from(cmps: Vec<StringCmp>) -> AndStringFilters {
        AndStringFilters {
            string_filters: cmps.into_iter().map(StringFilter::from).collect(),
        }
    }
}

impl From<OrStringFilters> for Vec<Vec<StringCmp>> {
    fn from(or_string_filters: OrStringFilters) -> Vec<Vec<StringCmp>> {
        // "or filters" are collections of "and" filters
        let mut new_or_filters = Vec::with_capacity(or_string_filters.and_string_filters.len());
        for and_filters in or_string_filters.and_string_filters {
            let and_filters = and_filters.string_filters;
            let mut new_and_filters = Vec::with_capacity(and_filters.len());
            for string_cmp in and_filters {
                new_and_filters.push(string_cmp.into());
            }
            new_or_filters.push(new_and_filters);
        }

        new_or_filters
    }
}

// Higher level helper for ints
#[derive(Clone, Debug)]
pub enum IntCmp {
    // (value, negated)
    Eq(i64, bool),
    Lt(i64, bool),
    Lte(i64, bool),
    Gt(i64, bool),
    Gte(i64, bool),
    Has,
}

impl From<&IntFilter> for IntCmp {
    fn from(int_filter: &IntFilter) -> Self {
        let IntFilter {
            value,
            negated,
            operation,
        } = int_filter;
        match operation {
            IntOperation::Has => IntCmp::Has,
            IntOperation::Equal => IntCmp::Eq(*value, *negated),
            IntOperation::LessThan => IntCmp::Lt(*value, *negated),
            IntOperation::LessThanOrEqual => IntCmp::Lte(*value, *negated),
            IntOperation::GreaterThan => IntCmp::Gt(*value, *negated),
            IntOperation::GreaterThanOrEqual => IntCmp::Gte(*value, *negated),
        }
    }
}

impl IntCmp {
    pub fn matches(&self, other: i64) -> bool {
        match self {
            IntCmp::Eq(value, negated) => (other == *value) ^ negated,
            IntCmp::Lt(value, negated) => (other < *value) ^ negated,
            IntCmp::Lte(value, negated) => (other <= *value) ^ negated,
            IntCmp::Gt(value, negated) => (other > *value) ^ negated,
            IntCmp::Gte(value, negated) => (other >= *value) ^ negated,
            IntCmp::Has => true,
        }
    }
}

impl From<IntCmp> for IntFilter {
    fn from(int_cmp: IntCmp) -> IntFilter {
        match int_cmp {
            IntCmp::Has => IntFilter {
                operation: IntOperation::Has,
                value: 0,
                negated: false,
            },
            IntCmp::Eq(value, negated) => IntFilter {
                operation: IntOperation::Equal,
                value,
                negated,
            },
            IntCmp::Lt(value, negated) => IntFilter {
                operation: IntOperation::LessThan,
                value,
                negated,
            },
            IntCmp::Lte(value, negated) => IntFilter {
                operation: IntOperation::LessThanOrEqual,
                value,
                negated,
            },
            IntCmp::Gt(value, negated) => IntFilter {
                operation: IntOperation::GreaterThan,
                value,
                negated,
            },
            IntCmp::Gte(value, negated) => IntFilter {
                operation: IntOperation::GreaterThanOrEqual,
                value,
                negated,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_cmp() -> eyre::Result<()> {
        eyre::ensure!(
            IntCmp::Eq(3, false).matches(3),
            "Eq: Positive case"
        );

        eyre::ensure!(
            !IntCmp::Eq(3, true).matches(3),
            "Eq: Positive case (negated)"
        );

        eyre::ensure!(
            !IntCmp::Eq(3, false).matches(4),
            "Eq: Negative case"
        );

        eyre::ensure!(
            IntCmp::Eq(3, true).matches(4),
            "Eq: Negative case (negated)"
        );

        // lt
        eyre::ensure!(
            IntCmp::Lt(3, false).matches(2),
            "Lt: Positive case"
        );

        eyre::ensure!(
            IntCmp::Lt(3, true).matches(4),
            "Lt: Negative case"
        );

        eyre::ensure!(
            IntCmp::Lt(3, true).matches(3),
            "Lt: Negative case fencepost"
        );

        // lte
        eyre::ensure!(
            IntCmp::Lte(3, false).matches(2),
            "Lte: Positive case"
        );

        eyre::ensure!(
            IntCmp::Lte(3, false).matches(3),
            "Lte: Positive case fencepost"
        );

        eyre::ensure!(
            IntCmp::Lte(3, true).matches(4),
            "Lte: Negative case"
        );

        // gt 
        eyre::ensure!(
            IntCmp::Gt(3, false).matches(4),
            "Gt: Positive case"
        );

        eyre::ensure!(
            !IntCmp::Gt(3, false).matches(2),
            "Gt: Negative case"
        );

        eyre::ensure!(
            IntCmp::Gt(3, true).matches(3),
            "Gt: Negative case fencepost"
        );

        // gte
        eyre::ensure!(
            IntCmp::Gte(3, false).matches(4),
            "Gte: Positive case"
        );

        eyre::ensure!(
            IntCmp::Gte(3, false).matches(3),
            "Gte: Positive case fencepost"
        );

        eyre::ensure!(
            IntCmp::Gte(3, true).matches(2),
            "Gte: Negative case"
        );

        // has
        eyre::ensure!(
            IntCmp::Has.matches(2),
            "I have no clue how to do this one"
        );

        Ok(())
    }
}
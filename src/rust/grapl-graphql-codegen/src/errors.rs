use graphql_parser::schema::Directive;
use crate::node_type::MergeFailure;

#[derive(Debug, thiserror::Error)]
pub enum CodeGenError<'a> {
    #[error("NodeTypeParseError")]
    NodeTypeParseError,
    #[error("Unsupported ConflictResolution")]
    UnsupportedConflictResolution {
        directives: Vec<Directive<'a, &'a str>>,
    },
    #[error("Missing Node IdentificationAlgorithm")]
    MissingNodeIdentificationAlgorithm {
        directives: Vec<Directive<'a, &'a str>>,
    },
    #[error("Missing Grapl Directive")]
    MissingGraplDirective {
        directives: Vec<Directive<'a, &'a str>>,
    },
    #[error("Missing Arguments for Grapl Directive")]
    MissingGraplDirectiveArguments {
        directives: Vec<Directive<'a, &'a str>>,
    },
    #[error("Failed to extend node schema")]
    MergeFailure(#[from] MergeFailure)
}

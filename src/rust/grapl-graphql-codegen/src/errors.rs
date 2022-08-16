use graphql_parser::schema::Directive;

use crate::node_type::MergeFailure;

#[derive(Debug, thiserror::Error)]
pub enum CodeGenError {
    #[error("NodeTypeParseError")]
    NodeTypeParseError,
    #[error("Unsupported ConflictResolution")]
    UnsupportedConflictResolution {
        directives: Vec<Directive<'static, String>>,
    },
    #[error("Missing Node IdentificationAlgorithm")]
    MissingNodeIdentificationAlgorithm {
        directives: Vec<Directive<'static, String>>,
    },
    #[error("Missing Grapl Directive")]
    MissingGraplDirective {
        directives: Vec<Directive<'static, String>>,
    },
    #[error("Missing Arguments for Grapl Directive")]
    MissingGraplDirectiveArguments {
        directives: Vec<Directive<'static, String>>,
    },
    #[error("Failed to extend node schema")]
    MergeFailure(#[from] MergeFailure),
}

use crate::{editor::Span, source::SourceFile};

/// A type of mutation operation that could be applied to a source file.
#[derive(Debug, Eq, Clone, Copy, PartialEq)]
pub enum MutationOp {
    /// Convert a `pub` to a `pub(crate)`
    ToVisCrate,
    /// Convert a `pub(crate)` to `pub(self)`
    ToVisSelf,
    /// Convert a `pub(crate)` to `pub(super)`
    ToVisSuper,
    /// Convert a `pub`, `pub(crate)`, `pub(self)`, `pub(super)` to inherited
    ToVisInherited,
}

impl MutationOp {
    /// Return the text that replaces the body of the mutated span, without the marker comment.
    fn replacement(&self) -> &'static str {
        use MutationOp::*;
        match self {
            ToVisCrate => "pub(crate)",
            ToVisSelf => "pub(self)",
            ToVisSuper => "pub(super)",
            ToVisInherited => "",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Mutation {
    pub source_file: SourceFile,

    /// The mutated textual region.
    span: Span,

    /// The type of change to apply.
    pub op: MutationOp,
}

impl Mutation {
    pub fn new(source_file: SourceFile, op: MutationOp, span: Span) -> Mutation {
        Mutation {
            source_file,
            op,
            span,
        }
    }
}

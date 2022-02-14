use crate::{
    editor::{replace_region, Span},
    source::SourceFile,
};

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
        // TODO correct editor so we don't have to add awkward whitespace padding
        match self {
            ToVisCrate => " pub(crate) ",
            ToVisSelf => " pub(self) ",
            ToVisSuper => " pub(super) ",
            ToVisInherited => " ",
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

    /// Return text of the whole file with the mutation applied.
    pub fn mutate(&self) -> String {
        replace_region(
            &self.source_file.code,
            &self.span.start,
            &self.span.end,
            self.op.replacement(),
        )
    }
}

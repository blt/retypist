// Copyright 2021 Martin Pool

//! Edit source code.

/// A (line, column) position in a source file.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct LineColumn {
    /// 1-based line number.
    pub line: usize,

    /// 1-based column, measured in chars.
    pub column: usize,
}

impl From<proc_macro2::LineColumn> for LineColumn {
    fn from(l: proc_macro2::LineColumn) -> Self {
        LineColumn {
            line: l.line,
            column: l.column + 1,
        }
    }
}

/// A contiguous text span in a file.
///
/// TODO: Perhaps a semi-open range that can represent an empty span would be more general?
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Span {
    /// The inclusive position where the span starts.
    pub start: LineColumn,
    /// The inclusive position where the span ends.
    pub end: LineColumn,
}

impl From<proc_macro2::Span> for Span {
    fn from(s: proc_macro2::Span) -> Self {
        Span {
            start: s.start().into(),
            end: s.end().into(),
        }
    }
}

impl From<&proc_macro2::Span> for Span {
    fn from(s: &proc_macro2::Span) -> Self {
        Span {
            start: s.start().into(),
            end: s.end().into(),
        }
    }
}

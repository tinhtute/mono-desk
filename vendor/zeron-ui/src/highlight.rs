//! Syntax highlight kinds, ported from zeron's `zeron_syntax` crate as a local
//! enum.
//!
//! Upstream `theme.rs` imports `HighlightKind` from the `zeron_syntax` domain
//! crate (a Tree-sitter/syntax-highlighting stack that mono-desk does not
//! carry). Only the enum itself reaches the theme: it keys [`crate::theme::
//! SyntaxPalette::color`]. The variant list matches zeron @ 8b7afb3 exactly so a
//! future syntax surface ports without renaming anything.

/// The semantic class of a highlighted token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HighlightKind {
    Comment,
    Keyword,
    String,
    StringSpecial,
    Escape,
    Number,
    Boolean,
    Type,
    TypeBuiltin,
    Constructor,
    Function,
    FunctionBuiltin,
    Macro,
    Property,
    Constant,
    Variable,
    VariableSpecial,
    Parameter,
    Operator,
    Punctuation,
    Tag,
    Attribute,
    Label,
    MarkupHeading,
    MarkupRaw,
    MarkupLink,
    MarkupReference,
    MarkupEmphasis,
    MarkupStrong,
    Embedded,
    Invalid,
}

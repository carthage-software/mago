use mago_span::Span;

/// The result of an inference step: a typed node, or the reason inference could
/// not produce one.
pub type InferenceResult<T> = Result<T, InferenceError>;

/// Why inference failed to produce a type for a node.
///
/// Inference is total over the types it supports: it never fabricates a `mixed`
/// to paper over a problem. When it cannot soundly type a node it stops and
/// returns one of these instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceError {
    /// A closure or arrow-function literal had no linked symbol in the symbol
    /// table. The binder assigns every function-like literal a symbol, so this
    /// can only mean the IR was inferred without being bound, or with a symbol
    /// table that was not linked against the code's own definitions.
    UnresolvedItemSymbol { span: Span, kind: &'static str },

    /// Inference does not yet support this construct.
    Unsupported { span: Span, construct: &'static str },
}

impl std::fmt::Display for InferenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnresolvedItemSymbol { kind, .. } => write!(
                formatter,
                "the {kind} has no linked symbol; the IR must be bound and its definitions linked before inference",
            ),
            Self::Unsupported { construct, .. } => {
                write!(formatter, "inference does not yet support {construct}")
            }
        }
    }
}

impl std::error::Error for InferenceError {}

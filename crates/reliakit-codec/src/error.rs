//! Error types returned by canonical encoding and decoding.

use core::fmt;

/// High-level category for a codec error.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CodecErrorKind {
    /// The input ended before the requested bytes could be read.
    UnexpectedEof,
    /// A value used a byte or tag that is not valid for its type.
    InvalidValue,
    /// A decoded length cannot be represented or safely processed.
    LengthOverflow,
    /// A decoded value left trailing bytes in an exact decode operation.
    TrailingBytes,
    /// The writer failed to accept bytes.
    WriteFailed,
    /// The reader failed for a reason other than end of input.
    ReadFailed,
}

/// Error returned by canonical encoding and decoding operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CodecError {
    kind: CodecErrorKind,
    message: &'static str,
}

impl CodecError {
    /// Creates a new codec error with a stable kind and actionable message.
    pub const fn new(kind: CodecErrorKind, message: &'static str) -> Self {
        Self { kind, message }
    }

    /// Returns the stable error category.
    pub const fn kind(&self) -> CodecErrorKind {
        self.kind
    }

    /// Returns a human-readable error message.
    pub const fn message(&self) -> &'static str {
        self.message
    }

    /// Input ended before the requested bytes could be read.
    pub const fn unexpected_eof() -> Self {
        Self::new(
            CodecErrorKind::UnexpectedEof,
            "input ended before the requested bytes could be read",
        )
    }

    /// Value used an invalid byte or tag.
    pub const fn invalid_value(message: &'static str) -> Self {
        Self::new(CodecErrorKind::InvalidValue, message)
    }

    /// Decoded length cannot be represented or safely processed.
    pub const fn length_overflow(message: &'static str) -> Self {
        Self::new(CodecErrorKind::LengthOverflow, message)
    }

    /// Exact decode found bytes after the decoded value.
    pub const fn trailing_bytes() -> Self {
        Self::new(
            CodecErrorKind::TrailingBytes,
            "decode completed but trailing bytes remain",
        )
    }

    /// Writer failed to accept bytes.
    pub const fn write_failed() -> Self {
        Self::new(CodecErrorKind::WriteFailed, "failed to write encoded bytes")
    }

    /// Reader failed for a reason other than end of input.
    pub const fn read_failed() -> Self {
        Self::new(CodecErrorKind::ReadFailed, "failed to read encoded bytes")
    }
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CodecError {}

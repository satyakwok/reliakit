//! Explicit resource limits for parsing untrusted JSON.

/// Resource limits enforced while parsing.
///
/// Limits bound *logical* decoded data (counts and byte lengths), not exact
/// allocator memory — real heap use also depends on `String`/`Vec` capacity and
/// the platform. Parsing untrusted input should always go through limits;
/// [`crate::parse`] applies [`JsonLimits::new`] by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsonLimits {
    /// Maximum size of the whole input, in bytes.
    pub max_input_bytes: usize,
    /// Maximum nesting depth of arrays and objects.
    pub max_depth: usize,
    /// Maximum decoded size of a single string value, in bytes.
    pub max_string_bytes: usize,
    /// Maximum decoded size of a single object key, in bytes.
    pub max_key_bytes: usize,
    /// Maximum length of a single number token, in bytes.
    pub max_number_bytes: usize,
    /// Maximum number of items in a single array.
    pub max_array_items: usize,
    /// Maximum number of members in a single object.
    pub max_object_members: usize,
    /// Maximum total number of values (nodes) in the document.
    pub max_total_nodes: usize,
    /// Maximum total decoded string bytes across the whole document.
    pub max_total_decoded_string_bytes: usize,
}

impl JsonLimits {
    /// The default limits: conservative values suitable for untrusted input.
    ///
    /// `1 MiB` input, depth `64`, `256 KiB` per string, `16 KiB` per key,
    /// `256` bytes per number, `100_000` array items / object members,
    /// `200_000` total nodes, `1 MiB` total decoded string bytes.
    pub const fn new() -> Self {
        Self {
            max_input_bytes: 1 << 20,
            max_depth: 64,
            max_string_bytes: 256 << 10,
            max_key_bytes: 16 << 10,
            max_number_bytes: 256,
            max_array_items: 100_000,
            max_object_members: 100_000,
            max_total_nodes: 200_000,
            max_total_decoded_string_bytes: 1 << 20,
        }
    }

    /// A tighter profile for small, low-trust payloads (e.g. tokens, webhooks).
    ///
    /// `64 KiB` input, depth `32`, `16 KiB` per string, `1 KiB` per key,
    /// `64` bytes per number, `4_096` array items / object members,
    /// `16_384` total nodes, `64 KiB` total decoded string bytes.
    pub const fn conservative() -> Self {
        Self {
            max_input_bytes: 64 << 10,
            max_depth: 32,
            max_string_bytes: 16 << 10,
            max_key_bytes: 1 << 10,
            max_number_bytes: 64,
            max_array_items: 4_096,
            max_object_members: 4_096,
            max_total_nodes: 16_384,
            max_total_decoded_string_bytes: 64 << 10,
        }
    }

    /// A looser profile for larger trusted documents. Still explicit and finite.
    ///
    /// `64 MiB` input, depth `128`, `16 MiB` per string, `256 KiB` per key,
    /// `1_024` bytes per number, `5_000_000` array items / object members,
    /// `10_000_000` total nodes, `64 MiB` total decoded string bytes.
    pub const fn permissive() -> Self {
        Self {
            max_input_bytes: 64 << 20,
            max_depth: 128,
            max_string_bytes: 16 << 20,
            max_key_bytes: 256 << 10,
            max_number_bytes: 1_024,
            max_array_items: 5_000_000,
            max_object_members: 5_000_000,
            max_total_nodes: 10_000_000,
            max_total_decoded_string_bytes: 64 << 20,
        }
    }

    /// Sets [`max_depth`](Self::max_depth).
    pub const fn with_max_depth(mut self, value: usize) -> Self {
        self.max_depth = value;
        self
    }

    /// Sets [`max_input_bytes`](Self::max_input_bytes).
    pub const fn with_max_input_bytes(mut self, value: usize) -> Self {
        self.max_input_bytes = value;
        self
    }

    /// Sets [`max_string_bytes`](Self::max_string_bytes).
    pub const fn with_max_string_bytes(mut self, value: usize) -> Self {
        self.max_string_bytes = value;
        self
    }

    /// Sets [`max_total_nodes`](Self::max_total_nodes).
    pub const fn with_max_total_nodes(mut self, value: usize) -> Self {
        self.max_total_nodes = value;
        self
    }
}

impl Default for JsonLimits {
    fn default() -> Self {
        Self::new()
    }
}

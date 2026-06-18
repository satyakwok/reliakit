//! Explicit resource limits for parsing untrusted JSON.

/// Resource limits enforced while parsing.
///
/// Limits bound *logical* decoded data (counts and byte lengths), not exact
/// allocator memory; real heap use also depends on `String`/`Vec` capacity and
/// the platform. Parsing untrusted input should always go through limits;
/// [`crate::parse`] applies [`JsonLimits::new`] by default.
///
/// Pick a profile with [`new`](Self::new), [`conservative`](Self::conservative),
/// or [`permissive`](Self::permissive), adjust individual limits with the
/// `with_*` builder methods, and read current values with the matching
/// accessors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsonLimits {
    max_input_bytes: usize,
    max_depth: usize,
    max_string_bytes: usize,
    max_key_bytes: usize,
    max_number_bytes: usize,
    max_array_items: usize,
    max_object_members: usize,
    max_total_nodes: usize,
    max_total_decoded_string_bytes: usize,
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

    /// Maximum size of the whole input, in bytes.
    pub const fn max_input_bytes(&self) -> usize {
        self.max_input_bytes
    }

    /// Maximum nesting depth of arrays and objects.
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Maximum decoded size of a single string value, in bytes.
    pub const fn max_string_bytes(&self) -> usize {
        self.max_string_bytes
    }

    /// Maximum decoded size of a single object key, in bytes.
    pub const fn max_key_bytes(&self) -> usize {
        self.max_key_bytes
    }

    /// Maximum length of a single number token, in bytes.
    pub const fn max_number_bytes(&self) -> usize {
        self.max_number_bytes
    }

    /// Maximum number of items in a single array.
    pub const fn max_array_items(&self) -> usize {
        self.max_array_items
    }

    /// Maximum number of members in a single object.
    pub const fn max_object_members(&self) -> usize {
        self.max_object_members
    }

    /// Maximum total number of values (nodes) in the document.
    pub const fn max_total_nodes(&self) -> usize {
        self.max_total_nodes
    }

    /// Maximum total decoded string bytes across the whole document.
    pub const fn max_total_decoded_string_bytes(&self) -> usize {
        self.max_total_decoded_string_bytes
    }

    /// Sets [`max_input_bytes`](Self::max_input_bytes).
    pub const fn with_max_input_bytes(mut self, value: usize) -> Self {
        self.max_input_bytes = value;
        self
    }

    /// Sets [`max_depth`](Self::max_depth).
    pub const fn with_max_depth(mut self, value: usize) -> Self {
        self.max_depth = value;
        self
    }

    /// Sets [`max_string_bytes`](Self::max_string_bytes).
    pub const fn with_max_string_bytes(mut self, value: usize) -> Self {
        self.max_string_bytes = value;
        self
    }

    /// Sets [`max_key_bytes`](Self::max_key_bytes).
    pub const fn with_max_key_bytes(mut self, value: usize) -> Self {
        self.max_key_bytes = value;
        self
    }

    /// Sets [`max_number_bytes`](Self::max_number_bytes).
    pub const fn with_max_number_bytes(mut self, value: usize) -> Self {
        self.max_number_bytes = value;
        self
    }

    /// Sets [`max_array_items`](Self::max_array_items).
    pub const fn with_max_array_items(mut self, value: usize) -> Self {
        self.max_array_items = value;
        self
    }

    /// Sets [`max_object_members`](Self::max_object_members).
    pub const fn with_max_object_members(mut self, value: usize) -> Self {
        self.max_object_members = value;
        self
    }

    /// Sets [`max_total_nodes`](Self::max_total_nodes).
    pub const fn with_max_total_nodes(mut self, value: usize) -> Self {
        self.max_total_nodes = value;
        self
    }

    /// Sets [`max_total_decoded_string_bytes`](Self::max_total_decoded_string_bytes).
    pub const fn with_max_total_decoded_string_bytes(mut self, value: usize) -> Self {
        self.max_total_decoded_string_bytes = value;
        self
    }
}

impl Default for JsonLimits {
    fn default() -> Self {
        Self::new()
    }
}

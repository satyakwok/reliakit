//! Resource limits applied while reading CSV.

/// Bounds applied by [`read_str`](crate::read_str) to defend against
/// adversarial input.
///
/// Every limit is checked while reading; exceeding one stops the read with a
/// [`CsvErrorKind::LimitExceeded`](crate::CsvErrorKind::LimitExceeded). Build a
/// profile with [`conservative`](Self::conservative) (the default) or
/// [`permissive`](Self::permissive), then tune individual limits with the
/// `with_*` methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CsvLimits {
    max_input_bytes: usize,
    max_records: usize,
    max_fields_per_record: usize,
    max_field_bytes: usize,
}

impl CsvLimits {
    /// Conservative limits suitable for untrusted input:
    /// 1 MiB total, 65 536 records, 1 024 fields per record, 65 536 bytes per
    /// field.
    pub const fn conservative() -> Self {
        Self {
            max_input_bytes: 1 << 20,
            max_records: 1 << 16,
            max_fields_per_record: 1 << 10,
            max_field_bytes: 1 << 16,
        }
    }

    /// Permissive limits for trusted input where large documents are expected:
    /// 256 MiB total, ~16.7 M records, 65 536 fields per record, 16 MiB per
    /// field.
    pub const fn permissive() -> Self {
        Self {
            max_input_bytes: 1 << 28,
            max_records: 1 << 24,
            max_fields_per_record: 1 << 16,
            max_field_bytes: 1 << 24,
        }
    }

    /// The maximum number of input bytes.
    pub const fn max_input_bytes(&self) -> usize {
        self.max_input_bytes
    }

    /// The maximum number of records.
    pub const fn max_records(&self) -> usize {
        self.max_records
    }

    /// The maximum number of fields in any one record.
    pub const fn max_fields_per_record(&self) -> usize {
        self.max_fields_per_record
    }

    /// The maximum number of bytes in any one field (after unescaping).
    pub const fn max_field_bytes(&self) -> usize {
        self.max_field_bytes
    }

    /// Returns a copy with `max_input_bytes` set.
    pub const fn with_max_input_bytes(mut self, value: usize) -> Self {
        self.max_input_bytes = value;
        self
    }

    /// Returns a copy with `max_records` set.
    pub const fn with_max_records(mut self, value: usize) -> Self {
        self.max_records = value;
        self
    }

    /// Returns a copy with `max_fields_per_record` set.
    pub const fn with_max_fields_per_record(mut self, value: usize) -> Self {
        self.max_fields_per_record = value;
        self
    }

    /// Returns a copy with `max_field_bytes` set.
    pub const fn with_max_field_bytes(mut self, value: usize) -> Self {
        self.max_field_bytes = value;
        self
    }
}

impl Default for CsvLimits {
    fn default() -> Self {
        Self::conservative()
    }
}

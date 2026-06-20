# CSV import with limits

## Problem

Importing a CSV file or upload means parsing untrusted input. A file can have far
more rows than you expect, absurdly many columns, or a single field megabytes
long. Without bounds, one bad upload can exhaust memory. You want to parse it
strictly, with explicit caps, and get an error when the input is too large
instead of falling over.

## Use

- `reliakit-csv`: strict, deterministic CSV (an RFC 4180 subset) with a
  `CsvLimits` profile for rows, columns, and field size.

## Example

Parse untrusted CSV into rows of strings, under a conservative limit profile:

```rust
use reliakit_csv::{read_str_with_limits, CsvLimits};

fn main() {
    let input = "name,port\napi,8080\nworker,9000\n";
    let limits = CsvLimits::conservative();

    match read_str_with_limits(input, &limits) {
        Ok(rows) => {
            // rows: Vec<Vec<String>>, header first.
            println!("{} row(s)", rows.len());
        }
        Err(err) => {
            // Too many rows/columns, an oversized field, or malformed quoting.
            eprintln!("rejected: {err}");
        }
    }
}
```

For typed records, give a struct a `CsvDecode` impl (hand-written or derived) and
use `from_csv_str_with_limits::<T>(input, &limits)` instead, which applies the
same bounds and then decodes each row.

## Run it

The bounded snippet above is the form to use for untrusted input. These shipped
examples show the general read/write and typed round-trip API; for untrusted
input use the `_with_limits` variants shown above instead of `read_str` /
`from_csv_str`:

```sh
cargo run -p reliakit-csv --example basic
cargo run -p reliakit --example typed_csv --features "csv derive"
```

## Why this works

`CsvLimits` caps the number of records, the fields per record, and the bytes per
field, checked while reading, so an oversized or malformed file is rejected before
it can allocate without bound. Parsing is strict and deterministic: quoting
follows the RFC 4180 subset, and writing the same records always produces the same
text. `from_csv_str_with_limits` layers typed decoding on top of the same bounds.

## Common mistakes

- **Reading without limits.** `read_str` and `from_csv_str` exist for trusted
  input; for anything from outside, use the `_with_limits` form.
- **Setting limits too high "to be safe".** A limit only protects you if it is
  smaller than what would hurt you. Size the profile to your real inputs.
- **Trusting the header blindly.** Validate the columns you require; a present but
  wrong header still produces rows.
- **Assuming every CSV dialect.** This is a strict RFC 4180 subset, not a
  catch-all for semicolon-separated or quoted-newline-heavy exports.

## When not to use this

- It is not a spreadsheet engine or a forgiving "accept anything" importer. If you
  must ingest arbitrary vendor dialects, normalize them first.
- It validates structure and size, not meaning. Combine it with field validation
  (`reliakit-validate`, `reliakit-primitives`) for business rules.
- For nested or richly typed payloads, a CSV is the wrong shape; use
  `reliakit-json` instead.

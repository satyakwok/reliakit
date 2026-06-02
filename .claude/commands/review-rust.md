Review all changed Rust code in this workspace for correctness, safety, and API quality.

## What to check

Run `git diff @{upstream}...HEAD` to get the diff. If empty, run `git diff HEAD~1`.

Then read every changed `.rs` file in full and check:

### Correctness
- Logic errors, wrong conditions, off-by-one, incorrect error variants returned
- Validation order issues (e.g. whitespace check before length check)
- Edge cases not covered: empty input, zero, u64::MAX, MIN==MAX, MIN>MAX
- Any panic path reachable from safe code

### API consistency
- `len()` — does it return bytes or chars? Is it consistent across sibling types?
- `is_empty()` — does it match what `len() == 0` would return?
- `Display` output — does it match what callers would expect?
- `From` vs `TryFrom` — infallible conversions that should be fallible
- Missing trait impls a caller would reasonably expect (TryFrom<u32>, TryFrom<f64>, etc.)

### Safety
- Any `unsafe` block — is it justified and correctly bounded?
- `#![forbid(unsafe_code)]` — still present on crates that require it?
- `saturating_*` used silently where an error should be returned instead

### no_std compatibility
- Any `std::` usage in code that should be `no_std`
- Missing `alloc::` imports where `String`/`Vec` are used under `no_std`

### Test coverage
- New public API without tests
- Tests that encode wrong behavior as expected (assert the bug, not the correct output)
- Missing edge case tests for boundary values

## Output format

Report findings as a numbered list. For each finding include:
- File and line number
- One-sentence description of the bug or issue
- Concrete example of when it fails

If no issues found, say so clearly. Do not pad the output.

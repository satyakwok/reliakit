//! Pure jitter helpers.
//!
//! Jitter spreads retries out in time so that many clients backing off together
//! do not retry in lockstep. These helpers are deterministic given their `rand`
//! argument: the caller supplies a random value (e.g. from `rand`, `getrandom`,
//! or a hardware RNG), keeping this crate dependency-free and the math testable.
//!
//! `rand` is interpreted as a fraction `rand / u32::MAX` in `0.0..=1.0`.

use core::time::Duration;

const NANOS_PER_SEC: u128 = 1_000_000_000;

/// Builds a `Duration` from a `u128` nanosecond count, saturating at
/// [`Duration::MAX`].
fn duration_from_nanos(nanos: u128) -> Duration {
    let secs = nanos / NANOS_PER_SEC;
    if secs > u64::MAX as u128 {
        Duration::MAX
    } else {
        Duration::new(secs as u64, (nanos % NANOS_PER_SEC) as u32)
    }
}

/// Scales `delay` by the fraction `rand / u32::MAX`, saturating.
fn scale(delay: Duration, rand: u32) -> Duration {
    let nanos = delay.as_nanos().saturating_mul(rand as u128) / u32::MAX as u128;
    duration_from_nanos(nanos)
}

/// Full jitter: returns a uniformly random delay in `0 ..= delay`.
///
/// `rand` is a caller-supplied random value spanning `0 ..= u32::MAX`.
///
/// ```
/// use core::time::Duration;
/// use reliakit_backoff::full_jitter;
///
/// let base = Duration::from_millis(800);
/// assert_eq!(full_jitter(base, 0), Duration::ZERO);
/// assert_eq!(full_jitter(base, u32::MAX), base);
/// ```
pub fn full_jitter(delay: Duration, rand: u32) -> Duration {
    scale(delay, rand)
}

/// Equal jitter: keeps half of `delay` fixed and randomizes the other half,
/// returning a delay in `delay/2 ..= delay`.
///
/// `rand` is a caller-supplied random value spanning `0 ..= u32::MAX`.
///
/// ```
/// use core::time::Duration;
/// use reliakit_backoff::equal_jitter;
///
/// let base = Duration::from_millis(800);
/// assert_eq!(equal_jitter(base, 0), Duration::from_millis(400));
/// assert_eq!(equal_jitter(base, u32::MAX), base);
/// ```
pub fn equal_jitter(delay: Duration, rand: u32) -> Duration {
    let half = delay / 2;
    half.saturating_add(scale(half, rand))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_jitter_bounds() {
        let d = Duration::from_millis(1000);
        assert_eq!(full_jitter(d, 0), Duration::ZERO);
        assert_eq!(full_jitter(d, u32::MAX), d);
        // Mid value lands inside the range.
        let mid = full_jitter(d, u32::MAX / 2);
        assert!(mid > Duration::ZERO && mid < d);
    }

    #[test]
    fn equal_jitter_bounds() {
        let d = Duration::from_millis(1000);
        assert_eq!(equal_jitter(d, 0), Duration::from_millis(500));
        assert_eq!(equal_jitter(d, u32::MAX), d);
        let mid = equal_jitter(d, u32::MAX / 2);
        assert!(mid > Duration::from_millis(500) && mid < d);
    }

    #[test]
    fn zero_delay_stays_zero() {
        assert_eq!(full_jitter(Duration::ZERO, u32::MAX), Duration::ZERO);
        assert_eq!(equal_jitter(Duration::ZERO, u32::MAX), Duration::ZERO);
    }

    #[test]
    fn large_delay_does_not_overflow() {
        let huge = Duration::from_secs(u64::MAX);
        // Should not panic; result is within the input.
        let j = full_jitter(huge, u32::MAX / 4);
        assert!(j <= huge);
    }
}

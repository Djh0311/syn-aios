//! Narrow UTC clock for the M2 workflow-state-sidecar reference slice.
//!
//! Receipts, events, audit rows, outbox leases and checkpoints need an RFC3339
//! value derived from the same Unix epoch rather than a build-date prefix.
//! This value object is injectable for crash/restart and cross-day tests; it
//! deliberately is not a cross-stage clock framework.

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct M2UtcClock {
    epoch_ms: i64,
}

impl M2UtcClock {
    pub(crate) fn system() -> Self {
        let epoch_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis() as i64)
            .unwrap_or(0);
        Self { epoch_ms }
    }

    pub(crate) const fn at_epoch_ms(epoch_ms: i64) -> Self {
        Self { epoch_ms }
    }

    pub(crate) const fn epoch_ms(self) -> i64 {
        self.epoch_ms
    }

    pub(crate) fn rfc3339(self) -> String {
        utc_rfc3339_at_epoch_ms(self.epoch_ms)
    }
}

pub(crate) fn utc_now_rfc3339() -> String {
    M2UtcClock::system().rfc3339()
}

/// Generate a UUIDv7 identifier from the same real UTC epoch used by the M2
/// receipts and ledgers. This remains a local M2 helper, not a cross-stage
/// identifier service.
pub(crate) fn uuid_v7() -> String {
    uuid_v7_at_epoch_ms(M2UtcClock::system().epoch_ms())
}

/// Deterministic-clock entry point for tests. The random tail remains random;
/// timestamp/version/variant bits are the UUIDv7 contract under test.
pub(crate) fn uuid_v7_at_epoch_ms(epoch_ms: i64) -> String {
    let mut bytes = [0u8; 16];
    let timestamp_bytes = (epoch_ms.max(0) as u64).to_be_bytes();
    bytes[..6].copy_from_slice(&timestamp_bytes[2..]);
    getrandom::getrandom(&mut bytes[6..]).expect("failed to generate UUIDv7 random bytes");
    bytes[6] = (bytes[6] & 0x0f) | 0x70;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

pub(crate) fn utc_rfc3339_at_epoch_ms(epoch_ms: i64) -> String {
    let seconds = epoch_ms.div_euclid(1_000);
    let millis = epoch_ms.rem_euclid(1_000);
    let days = seconds.div_euclid(86_400);
    let seconds_in_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_in_day / 3_600;
    let minute = (seconds_in_day % 3_600) / 60;
    let second = seconds_in_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}

// Howard Hinnant's public-domain civil-date conversion, expressed with
// Euclidean arithmetic so values before 1970 remain well-defined.
fn civil_from_days(days_since_unix_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::{utc_rfc3339_at_epoch_ms, uuid_v7_at_epoch_ms, M2UtcClock};

    #[test]
    fn injected_clock_formats_epoch_and_cross_day_without_build_date() {
        assert_eq!(utc_rfc3339_at_epoch_ms(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(
            utc_rfc3339_at_epoch_ms(86_400_001),
            "1970-01-02T00:00:00.001Z"
        );
    }

    #[test]
    fn rfc_values_sort_with_their_epoch_values_across_midnight() {
        let before_midnight = M2UtcClock::at_epoch_ms(86_399_999).rfc3339();
        let after_midnight = M2UtcClock::at_epoch_ms(86_400_000).rfc3339();
        assert!(before_midnight < after_midnight);
        assert_eq!(after_midnight, "1970-01-02T00:00:00.000Z");
    }

    #[test]
    fn restart_can_reconstruct_the_same_persisted_clock_value() {
        let persisted_epoch = 1_775_606_400_123_i64;
        let before_restart = M2UtcClock::at_epoch_ms(persisted_epoch);
        let after_restart = M2UtcClock::at_epoch_ms(before_restart.epoch_ms());
        assert_eq!(before_restart.rfc3339(), after_restart.rfc3339());
    }

    #[test]
    fn uuid_v7_carries_the_injected_utc_epoch_and_standard_layout_bits() {
        let id = uuid_v7_at_epoch_ms(1_775_606_400_123);
        let compact = id.replace('-', "");
        assert_eq!(&compact[..12], "019d6a63847b");
        assert_eq!(&compact[12..13], "7");
        assert!(matches!(&compact[16..17], "8" | "9" | "a" | "b"));
    }
}

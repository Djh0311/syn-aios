//! M4C07 的纯本地日程核心。
//!
//! 本模块不持久化、不启动 timer，也不调用模型。它只把后端解析出的 OS
//! timezone、TZif 规则、checkpoint 和事件输入转换为可持久化的确定性计划。
//! 所有配置错误均以静态、脱敏的错误码表达，绝不把 UTC 当作隐式后备。

use crate::m4_secretary_domain::m4_internal_id;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(crate) const M4_SCHEDULER_TICK_SECONDS: i64 = 60;
pub(crate) const M4_DAILY_CLOSE_GRACE_SECONDS: i64 = 5 * 60;
pub(crate) const M4_MAXIMUM_CLOSED_WINDOWS_PER_STARTUP: usize = 7;
pub(crate) const M4_CATCH_UP_TRUNCATED: &str = "CATCH_UP_TRUNCATED";
pub(crate) const M4_EXPLICIT_CATCH_UP_RECOVERED: &str = "EXPLICIT_CATCH_UP_RECOVERED";
pub(crate) const M4_CATCH_UP_RECOVERY_PARTIAL: &str = "CATCH_UP_RECOVERY_PARTIAL";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4SchedulerError {
    ConfigurationInvalid,
    TimezoneUnavailable,
    TimezoneInvalid,
    TzifInvalid,
    TzifVersionUnsupported,
    TzifFutureRulesInvalid,
    LocalDateInvalid,
    UtcTimestampInvalid,
    TimestampOutOfRange,
    LocalBoundaryUnresolvable,
    DeterministicIdFailed,
    SchedulerRunConfigurationMismatch,
    ExplicitCatchUpRangeInvalid,
}

impl M4SchedulerError {
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::ConfigurationInvalid => "m4_scheduler_configuration_invalid",
            Self::TimezoneUnavailable => "m4_scheduler_timezone_unavailable",
            Self::TimezoneInvalid => "m4_scheduler_timezone_invalid",
            Self::TzifInvalid => "m4_scheduler_tzif_invalid",
            Self::TzifVersionUnsupported => "m4_scheduler_tzif_v2_v3_required",
            Self::TzifFutureRulesInvalid => "m4_scheduler_tzif_future_rules_invalid",
            Self::LocalDateInvalid => "m4_scheduler_local_date_invalid",
            Self::UtcTimestampInvalid => "m4_scheduler_utc_timestamp_invalid",
            Self::TimestampOutOfRange => "m4_scheduler_timestamp_out_of_range",
            Self::LocalBoundaryUnresolvable => "m4_scheduler_local_boundary_unresolvable",
            Self::DeterministicIdFailed => "m4_scheduler_deterministic_id_failed",
            Self::SchedulerRunConfigurationMismatch => "m4_scheduler_run_configuration_mismatch",
            Self::ExplicitCatchUpRangeInvalid => "m4_scheduler_explicit_catch_up_range_invalid",
        }
    }
}

/// A proleptic-Gregorian local calendar day. M4 deliberately operates only on
/// ordinary four-digit civil dates; timestamps outside that range fail closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct M4LocalDate {
    year: i32,
    month: u8,
    day: u8,
}

impl M4LocalDate {
    pub(crate) fn new(year: i32, month: u8, day: u8) -> Result<Self, M4SchedulerError> {
        let date = Self { year, month, day };
        date.validate()?;
        Ok(date)
    }

    pub(crate) fn year(self) -> i32 {
        self.year
    }

    pub(crate) fn month(self) -> u8 {
        self.month
    }

    pub(crate) fn day(self) -> u8 {
        self.day
    }

    pub(crate) fn canonical(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    fn validate(self) -> Result<(), M4SchedulerError> {
        if !(0..=9999).contains(&self.year)
            || !(1..=12).contains(&self.month)
            || self.day == 0
            || self.day > m4_days_in_month(self.year, self.month)
        {
            return Err(M4SchedulerError::LocalDateInvalid);
        }
        Ok(())
    }

    fn days_since_unix_epoch(self) -> Result<i64, M4SchedulerError> {
        self.validate()?;
        let mut year = i64::from(self.year);
        let month = i64::from(self.month);
        let day = i64::from(self.day);
        year -= i64::from(month <= 2);
        let era = if year >= 0 { year } else { year - 399 } / 400;
        let year_of_era = year - era * 400;
        let month_from_march = month + if month > 2 { -3 } else { 9 };
        let day_of_year = (153 * month_from_march + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        Ok(era * 146_097 + day_of_era - 719_468)
    }

    fn from_days_since_unix_epoch(days: i64) -> Result<Self, M4SchedulerError> {
        let shifted = days
            .checked_add(719_468)
            .ok_or(M4SchedulerError::TimestampOutOfRange)?;
        let era = if shifted >= 0 {
            shifted
        } else {
            shifted - 146_096
        } / 146_097;
        let day_of_era = shifted - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let mut year = year_of_era + era * 400;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_prime = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
        let month = month_prime + if month_prime < 10 { 3 } else { -9 };
        year += i64::from(month <= 2);
        let date = Self {
            year: i32::try_from(year).map_err(|_| M4SchedulerError::TimestampOutOfRange)?,
            month: u8::try_from(month).map_err(|_| M4SchedulerError::TimestampOutOfRange)?,
            day: u8::try_from(day).map_err(|_| M4SchedulerError::TimestampOutOfRange)?,
        };
        date.validate()?;
        Ok(date)
    }

    fn add_days(self, days: i64) -> Result<Self, M4SchedulerError> {
        let total = self
            .days_since_unix_epoch()?
            .checked_add(days)
            .ok_or(M4SchedulerError::TimestampOutOfRange)?;
        Self::from_days_since_unix_epoch(total)
    }

    fn previous_day(self) -> Result<Self, M4SchedulerError> {
        self.add_days(-1)
    }
}

fn m4_is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn m4_days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if m4_is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn m4_utc_seconds(
    date: M4LocalDate,
    hour: u8,
    minute: u8,
    second: u8,
) -> Result<i64, M4SchedulerError> {
    if hour > 23 || minute > 59 || second > 59 {
        return Err(M4SchedulerError::TimestampOutOfRange);
    }
    date.days_since_unix_epoch()?
        .checked_mul(86_400)
        .and_then(|base| {
            base.checked_add(i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second))
        })
        .ok_or(M4SchedulerError::TimestampOutOfRange)
}

fn m4_utc_date_from_seconds(timestamp: i64) -> Result<M4LocalDate, M4SchedulerError> {
    M4LocalDate::from_days_since_unix_epoch(timestamp.div_euclid(86_400))
}

pub(crate) fn m4_format_utc_seconds(timestamp: i64) -> Result<String, M4SchedulerError> {
    let date = m4_utc_date_from_seconds(timestamp)?;
    let seconds_of_day = timestamp.rem_euclid(86_400);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    Ok(format!(
        "{}T{:02}:{:02}:{:02}Z",
        date.canonical(),
        hour,
        minute,
        second
    ))
}

/// Parse the exact UTC-Z timestamp grammar already admitted by M4:
/// `YYYY-MM-DDTHH:MM:SSZ` with an optional one-to-nine-digit fractional
/// suffix. Fractional input is deliberately mapped to the enclosing UTC
/// second, i.e. scheduling uses `floor(instant_seconds)`.
pub(crate) fn m4_parse_utc_seconds(value: &str) -> Result<i64, M4SchedulerError> {
    let bytes = value.as_bytes();
    let has_fraction = match bytes.len() {
        20 => false,
        22..=30 => bytes.get(19) == Some(&b'.'),
        _ => return Err(M4SchedulerError::UtcTimestampInvalid),
    };
    let fixed_shape = bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && bytes.get(10) == Some(&b'T')
        && bytes.get(13) == Some(&b':')
        && bytes.get(16) == Some(&b':')
        && bytes.last() == Some(&b'Z')
        && m4_all_ascii_digits(bytes.get(0..4))
        && m4_all_ascii_digits(bytes.get(5..7))
        && m4_all_ascii_digits(bytes.get(8..10))
        && m4_all_ascii_digits(bytes.get(11..13))
        && m4_all_ascii_digits(bytes.get(14..16))
        && m4_all_ascii_digits(bytes.get(17..19))
        && (!has_fraction || m4_all_ascii_digits(bytes.get(20..bytes.len() - 1)));
    if !fixed_shape {
        return Err(M4SchedulerError::UtcTimestampInvalid);
    }
    let year = m4_parse_ascii_decimal(bytes.get(0..4))?;
    let month = m4_parse_ascii_decimal(bytes.get(5..7))?;
    let day = m4_parse_ascii_decimal(bytes.get(8..10))?;
    let hour = m4_parse_ascii_decimal(bytes.get(11..13))?;
    let minute = m4_parse_ascii_decimal(bytes.get(14..16))?;
    let second = m4_parse_ascii_decimal(bytes.get(17..19))?;
    let date = M4LocalDate::new(
        i32::try_from(year).map_err(|_| M4SchedulerError::UtcTimestampInvalid)?,
        u8::try_from(month).map_err(|_| M4SchedulerError::UtcTimestampInvalid)?,
        u8::try_from(day).map_err(|_| M4SchedulerError::UtcTimestampInvalid)?,
    )
    .map_err(|_| M4SchedulerError::UtcTimestampInvalid)?;
    let timestamp = m4_utc_seconds(
        date,
        u8::try_from(hour).map_err(|_| M4SchedulerError::UtcTimestampInvalid)?,
        u8::try_from(minute).map_err(|_| M4SchedulerError::UtcTimestampInvalid)?,
        u8::try_from(second).map_err(|_| M4SchedulerError::UtcTimestampInvalid)?,
    )
    .map_err(|_| M4SchedulerError::UtcTimestampInvalid)?;
    let canonical =
        m4_format_utc_seconds(timestamp).map_err(|_| M4SchedulerError::UtcTimestampInvalid)?;
    let expected = if has_fraction {
        format!("{}Z", &value[..19])
    } else {
        value.to_string()
    };
    if canonical != expected {
        return Err(M4SchedulerError::UtcTimestampInvalid);
    }
    Ok(timestamp)
}

fn m4_all_ascii_digits(value: Option<&[u8]>) -> bool {
    value
        .filter(|value| !value.is_empty())
        .is_some_and(|value| value.iter().all(u8::is_ascii_digit))
}

fn m4_parse_ascii_decimal(value: Option<&[u8]>) -> Result<u32, M4SchedulerError> {
    let value = value.ok_or(M4SchedulerError::UtcTimestampInvalid)?;
    if !value.iter().all(u8::is_ascii_digit) {
        return Err(M4SchedulerError::UtcTimestampInvalid);
    }
    value.iter().try_fold(0u32, |accumulator, digit| {
        accumulator
            .checked_mul(10)
            .and_then(|value| value.checked_add(u32::from(*digit - b'0')))
            .ok_or(M4SchedulerError::UtcTimestampInvalid)
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct M4TzifHeader {
    version: u8,
    ttisgmtcnt: usize,
    ttisstdcnt: usize,
    leapcnt: usize,
    timecnt: usize,
    typecnt: usize,
    charcnt: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct M4TimezoneType {
    utc_offset_seconds: i32,
    is_dst: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct M4TimezoneTransition {
    at_utc: i64,
    type_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum M4FutureTimezoneRules {
    None,
    Fixed { utc_offset_seconds: i32 },
    Dst(M4PosixDstRules),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct M4PosixDstRules {
    standard_utc_offset_seconds: i32,
    daylight_utc_offset_seconds: i32,
    start: M4PosixTransitionRule,
    end: M4PosixTransitionRule,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct M4PosixTransitionRule {
    day: M4PosixRuleDay,
    seconds: i32,
    basis: M4PosixTimeBasis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum M4PosixRuleDay {
    JulianNoLeap(u16),
    DayOfYear(u16),
    MonthWeekday { month: u8, week: u8, weekday: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum M4PosixTimeBasis {
    Wall,
    Standard,
    Utc,
}

/// The parsed 64-bit side of a TZif v2/v3 file. Raw file bytes never escape
/// this type; the version is a deterministic opaque reference to them.
#[derive(Clone, Debug)]
pub(crate) struct M4TimezoneRules {
    iana_timezone: String,
    timezone_rules_version: String,
    transitions: Vec<M4TimezoneTransition>,
    types: Vec<M4TimezoneType>,
    default_type_index: usize,
    future_rules: M4FutureTimezoneRules,
}

impl M4TimezoneRules {
    pub(crate) fn iana_timezone(&self) -> &str {
        &self.iana_timezone
    }

    pub(crate) fn timezone_rules_version(&self) -> &str {
        &self.timezone_rules_version
    }

    fn offset_at_utc(&self, timestamp: i64) -> Result<i32, M4SchedulerError> {
        let use_future_rules = self
            .transitions
            .last()
            .map(|transition| timestamp > transition.at_utc)
            .unwrap_or(true);
        if use_future_rules {
            match &self.future_rules {
                M4FutureTimezoneRules::None => {}
                M4FutureTimezoneRules::Fixed { utc_offset_seconds } => {
                    return Ok(*utc_offset_seconds);
                }
                M4FutureTimezoneRules::Dst(rules) => return rules.offset_at_utc(timestamp),
            }
        }

        let mut lower = 0usize;
        let mut upper = self.transitions.len();
        while lower < upper {
            let middle = lower + (upper - lower) / 2;
            if self.transitions[middle].at_utc <= timestamp {
                lower = middle + 1;
            } else {
                upper = middle;
            }
        }
        let type_index = if lower == 0 {
            self.default_type_index
        } else {
            self.transitions[lower - 1].type_index
        };
        self.types
            .get(type_index)
            .map(|kind| kind.utc_offset_seconds)
            .ok_or(M4SchedulerError::TzifInvalid)
    }

    fn local_date_at_utc(&self, timestamp: i64) -> Result<M4LocalDate, M4SchedulerError> {
        let local_timestamp = timestamp
            .checked_add(i64::from(self.offset_at_utc(timestamp)?))
            .ok_or(M4SchedulerError::TimestampOutOfRange)?;
        m4_utc_date_from_seconds(local_timestamp)
    }

    fn resolve_local_midnight(&self, date: M4LocalDate) -> Result<i64, M4SchedulerError> {
        let local_midnight = date
            .days_since_unix_epoch()?
            .checked_mul(86_400)
            .ok_or(M4SchedulerError::TimestampOutOfRange)?;
        let mut candidates = Vec::new();
        for offset in self.known_offsets() {
            let candidate = local_midnight
                .checked_sub(i64::from(offset))
                .ok_or(M4SchedulerError::TimestampOutOfRange)?;
            if self.offset_at_utc(candidate)? == offset {
                candidates.push(candidate);
            }
        }
        if let Some(candidate) = candidates.into_iter().min() {
            // For a repeated midnight, use the first occurrence. This makes
            // the preceding day 25 hours without emitting two windows.
            return Ok(candidate);
        }

        for (transition_utc, before, after) in self.transitions_near(date)? {
            if after <= before {
                continue;
            }
            let first_missing = transition_utc
                .checked_add(i64::from(before))
                .ok_or(M4SchedulerError::TimestampOutOfRange)?;
            let first_valid = transition_utc
                .checked_add(i64::from(after))
                .ok_or(M4SchedulerError::TimestampOutOfRange)?;
            if local_midnight >= first_missing && local_midnight < first_valid {
                // A skipped midnight begins at the first valid instant after
                // the gap, preserving the half-open local-day partition.
                return Ok(transition_utc);
            }
        }
        Err(M4SchedulerError::LocalBoundaryUnresolvable)
    }

    fn known_offsets(&self) -> BTreeSet<i32> {
        let mut offsets = self
            .types
            .iter()
            .map(|kind| kind.utc_offset_seconds)
            .collect::<BTreeSet<_>>();
        match &self.future_rules {
            M4FutureTimezoneRules::None => {}
            M4FutureTimezoneRules::Fixed { utc_offset_seconds } => {
                offsets.insert(*utc_offset_seconds);
            }
            M4FutureTimezoneRules::Dst(rules) => {
                offsets.insert(rules.standard_utc_offset_seconds);
                offsets.insert(rules.daylight_utc_offset_seconds);
            }
        }
        offsets
    }

    fn transitions_near(
        &self,
        local_date: M4LocalDate,
    ) -> Result<Vec<(i64, i32, i32)>, M4SchedulerError> {
        let mut output = Vec::with_capacity(self.transitions.len().saturating_add(6));
        for (index, transition) in self.transitions.iter().enumerate() {
            let before_type = if index == 0 {
                self.default_type_index
            } else {
                self.transitions[index - 1].type_index
            };
            let before = self
                .types
                .get(before_type)
                .ok_or(M4SchedulerError::TzifInvalid)?
                .utc_offset_seconds;
            let after = self
                .types
                .get(transition.type_index)
                .ok_or(M4SchedulerError::TzifInvalid)?
                .utc_offset_seconds;
            output.push((transition.at_utc, before, after));
        }

        if let M4FutureTimezoneRules::Dst(rules) = &self.future_rules {
            let first_year = local_date.year().saturating_sub(1).max(1);
            let last_year = local_date.year().saturating_add(1).min(9999);
            let last_explicit_transition = self.transitions.last().map(|entry| entry.at_utc);
            for year in first_year..=last_year {
                for event in rules.transitions_for_year(year)? {
                    if last_explicit_transition
                        .map(|last| event.0 > last)
                        .unwrap_or(true)
                    {
                        output.push(event);
                    }
                }
            }
        }
        output.sort_by_key(|entry| entry.0);
        Ok(output)
    }
}

impl M4PosixDstRules {
    fn offset_at_utc(&self, timestamp: i64) -> Result<i32, M4SchedulerError> {
        let year = m4_utc_date_from_seconds(timestamp)?.year();
        let start = self.transition_utc(year, &self.start, self.standard_utc_offset_seconds)?;
        let end = self.transition_utc(year, &self.end, self.daylight_utc_offset_seconds)?;
        let in_dst = if start < end {
            timestamp >= start && timestamp < end
        } else {
            timestamp >= start || timestamp < end
        };
        Ok(if in_dst {
            self.daylight_utc_offset_seconds
        } else {
            self.standard_utc_offset_seconds
        })
    }

    fn transitions_for_year(&self, year: i32) -> Result<Vec<(i64, i32, i32)>, M4SchedulerError> {
        Ok(vec![
            (
                self.transition_utc(year, &self.start, self.standard_utc_offset_seconds)?,
                self.standard_utc_offset_seconds,
                self.daylight_utc_offset_seconds,
            ),
            (
                self.transition_utc(year, &self.end, self.daylight_utc_offset_seconds)?,
                self.daylight_utc_offset_seconds,
                self.standard_utc_offset_seconds,
            ),
        ])
    }

    fn transition_utc(
        &self,
        year: i32,
        rule: &M4PosixTransitionRule,
        wall_offset_before: i32,
    ) -> Result<i64, M4SchedulerError> {
        let day = rule.day.date_for_year(year)?;
        let base = day
            .days_since_unix_epoch()?
            .checked_mul(86_400)
            .and_then(|value| value.checked_add(i64::from(rule.seconds)))
            .ok_or(M4SchedulerError::TimestampOutOfRange)?;
        let offset = match rule.basis {
            M4PosixTimeBasis::Wall => wall_offset_before,
            M4PosixTimeBasis::Standard => self.standard_utc_offset_seconds,
            M4PosixTimeBasis::Utc => 0,
        };
        base.checked_sub(i64::from(offset))
            .ok_or(M4SchedulerError::TimestampOutOfRange)
    }
}

impl M4PosixRuleDay {
    fn date_for_year(&self, year: i32) -> Result<M4LocalDate, M4SchedulerError> {
        let first = M4LocalDate::new(year, 1, 1)?;
        match self {
            Self::JulianNoLeap(day) => {
                let mut index = i64::from(*day) - 1;
                if m4_is_leap_year(year) && *day >= 60 {
                    index += 1;
                }
                first.add_days(index)
            }
            Self::DayOfYear(day) => first.add_days(i64::from(*day)),
            Self::MonthWeekday {
                month,
                week,
                weekday,
            } => {
                let first_of_month = M4LocalDate::new(year, *month, 1)?;
                let first_weekday = m4_weekday(first_of_month)?;
                let target = i64::from(*weekday);
                if *week == 5 {
                    let last = M4LocalDate::new(year, *month, m4_days_in_month(year, *month))?;
                    let last_weekday = m4_weekday(last)?;
                    last.add_days(-((last_weekday - target).rem_euclid(7)))
                } else {
                    first_of_month
                        .add_days((target - first_weekday).rem_euclid(7) + 7 * i64::from(*week - 1))
                }
            }
        }
    }
}

fn m4_weekday(date: M4LocalDate) -> Result<i64, M4SchedulerError> {
    // Sunday = 0. 1970-01-01 was Thursday = 4.
    Ok((date.days_since_unix_epoch()? + 4).rem_euclid(7))
}

/// OS paths are injectable only to keep the resolver testable. Production
/// callers use `Default`, which never consults the `TZ` environment variable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SchedulerOsTimezonePaths {
    pub(crate) localtime_path: PathBuf,
    pub(crate) timezone_file_path: PathBuf,
    pub(crate) zoneinfo_roots: Vec<PathBuf>,
}

impl Default for M4SchedulerOsTimezonePaths {
    fn default() -> Self {
        Self {
            localtime_path: PathBuf::from("/etc/localtime"),
            timezone_file_path: PathBuf::from("/etc/timezone"),
            zoneinfo_roots: vec![
                PathBuf::from("/var/db/timezone/zoneinfo"),
                PathBuf::from("/usr/share/zoneinfo"),
                PathBuf::from("/usr/share/lib/zoneinfo"),
            ],
        }
    }
}

/// Resolve a real IANA name from the OS's timezone configuration and parse the
/// TZif payload. Missing files, a copied/unidentifiable `/etc/localtime`, or a
/// malformed name all return a scrubbed error instead of a UTC fallback.
pub(crate) fn m4_resolve_os_timezone(
    paths: &M4SchedulerOsTimezonePaths,
) -> Result<M4TimezoneRules, M4SchedulerError> {
    let roots = m4_canonical_zoneinfo_roots(&paths.zoneinfo_roots);
    if roots.is_empty() {
        return Err(M4SchedulerError::TimezoneUnavailable);
    }

    if let Ok(localtime) = fs::canonicalize(&paths.localtime_path) {
        for root in &roots {
            if let Some(name) = m4_timezone_name_below_root(&localtime, root)? {
                return m4_load_timezone_from_canonical_root(root, &name);
            }
        }
    }

    let name = m4_read_os_timezone_name_file(&paths.timezone_file_path)?;
    for root in &roots {
        match m4_load_timezone_from_canonical_root(root, &name) {
            Ok(rules) => return Ok(rules),
            Err(M4SchedulerError::TimezoneInvalid) => continue,
            Err(error) => return Err(error),
        }
    }
    Err(M4SchedulerError::TimezoneInvalid)
}

/// Load a named zone from a known zoneinfo root. The candidate is canonicalized
/// and required to remain below that root, so text validation alone never
/// turns into path traversal or acceptance of a non-IANA file.
pub(crate) fn m4_load_timezone_from_zoneinfo_root(
    zoneinfo_root: &Path,
    iana_timezone: &str,
) -> Result<M4TimezoneRules, M4SchedulerError> {
    let root =
        fs::canonicalize(zoneinfo_root).map_err(|_| M4SchedulerError::TimezoneUnavailable)?;
    m4_load_timezone_from_canonical_root(&root, iana_timezone)
}

fn m4_canonical_zoneinfo_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut canonical = BTreeSet::new();
    for root in roots {
        if let Ok(root) = fs::canonicalize(root) {
            if root.is_dir() {
                canonical.insert(root);
            }
        }
    }
    canonical.into_iter().collect()
}

fn m4_read_os_timezone_name_file(path: &Path) -> Result<String, M4SchedulerError> {
    let raw = fs::read_to_string(path).map_err(|_| M4SchedulerError::TimezoneUnavailable)?;
    if raw.len() > 256 {
        return Err(M4SchedulerError::TimezoneInvalid);
    }
    let without_lf = raw.strip_suffix('\n').unwrap_or(&raw);
    let name = without_lf.strip_suffix('\r').unwrap_or(without_lf);
    if name.is_empty() || name != name.trim() || name.contains('\n') || name.contains('\r') {
        return Err(M4SchedulerError::TimezoneInvalid);
    }
    m4_validate_iana_timezone_name(name)?;
    Ok(name.to_string())
}

fn m4_timezone_name_below_root(
    timezone_file: &Path,
    canonical_root: &Path,
) -> Result<Option<String>, M4SchedulerError> {
    let Ok(relative) = timezone_file.strip_prefix(canonical_root) else {
        return Ok(None);
    };
    let mut components = Vec::new();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(M4SchedulerError::TimezoneInvalid);
        };
        let segment = component
            .to_str()
            .ok_or(M4SchedulerError::TimezoneInvalid)?;
        components.push(segment);
    }
    if components
        .first()
        .is_some_and(|segment| *segment == "posix" || *segment == "right")
    {
        components.remove(0);
    }
    let name = components.join("/");
    m4_validate_iana_timezone_name(&name)?;
    Ok(Some(name))
}

fn m4_validate_iana_timezone_name(value: &str) -> Result<(), M4SchedulerError> {
    let valid = (3..=128).contains(&value.len())
        && value.contains('/')
        && !value.starts_with('/')
        && !value.ends_with('/')
        && value.split('/').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'+'))
        });
    if valid {
        Ok(())
    } else {
        Err(M4SchedulerError::TimezoneInvalid)
    }
}

fn m4_load_timezone_from_canonical_root(
    canonical_root: &Path,
    iana_timezone: &str,
) -> Result<M4TimezoneRules, M4SchedulerError> {
    m4_validate_iana_timezone_name(iana_timezone)?;
    let candidate = fs::canonicalize(canonical_root.join(iana_timezone))
        .map_err(|_| M4SchedulerError::TimezoneInvalid)?;
    if !candidate.starts_with(canonical_root) || !candidate.is_file() {
        return Err(M4SchedulerError::TimezoneInvalid);
    }
    let bytes = fs::read(candidate).map_err(|_| M4SchedulerError::TimezoneUnavailable)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(M4SchedulerError::TzifInvalid);
    }
    m4_parse_tzif(iana_timezone, &bytes)
}

fn m4_parse_tzif(iana_timezone: &str, bytes: &[u8]) -> Result<M4TimezoneRules, M4SchedulerError> {
    let (first_header, first_data_start) = m4_parse_tzif_header(bytes, 0)?;
    if !matches!(first_header.version, b'2' | b'3') {
        return Err(M4SchedulerError::TzifVersionUnsupported);
    }
    let first_block_length = m4_tzif_block_length(&first_header, 4)?;
    let second_header_offset = first_data_start
        .checked_add(first_block_length)
        .ok_or(M4SchedulerError::TzifInvalid)?;
    let (second_header, second_data_start) = m4_parse_tzif_header(bytes, second_header_offset)?;
    if second_header.version != first_header.version
        || !matches!(second_header.version, b'2' | b'3')
    {
        return Err(M4SchedulerError::TzifVersionUnsupported);
    }
    let second_block_length = m4_tzif_block_length(&second_header, 8)?;
    let second_block_end = second_data_start
        .checked_add(second_block_length)
        .filter(|end| *end <= bytes.len())
        .ok_or(M4SchedulerError::TzifInvalid)?;

    let mut cursor = second_data_start;
    let mut transition_times = Vec::with_capacity(second_header.timecnt);
    for _ in 0..second_header.timecnt {
        let time = m4_take_i64(bytes, &mut cursor)?;
        if transition_times
            .last()
            .is_some_and(|previous| time <= *previous)
        {
            return Err(M4SchedulerError::TzifInvalid);
        }
        transition_times.push(time);
    }
    let type_indices = m4_take_bytes(bytes, &mut cursor, second_header.timecnt)?.to_vec();
    let mut types = Vec::with_capacity(second_header.typecnt);
    let mut abbreviation_indices = Vec::with_capacity(second_header.typecnt);
    for _ in 0..second_header.typecnt {
        let utc_offset_seconds = m4_take_i32(bytes, &mut cursor)?;
        let is_dst = *m4_take_bytes(bytes, &mut cursor, 1)?
            .first()
            .ok_or(M4SchedulerError::TzifInvalid)?
            != 0;
        let abbreviation_index = *m4_take_bytes(bytes, &mut cursor, 1)?
            .first()
            .ok_or(M4SchedulerError::TzifInvalid)?;
        types.push(M4TimezoneType {
            utc_offset_seconds,
            is_dst,
        });
        abbreviation_indices.push(usize::from(abbreviation_index));
    }
    let abbreviations = m4_take_bytes(bytes, &mut cursor, second_header.charcnt)?;
    if types.is_empty()
        || abbreviation_indices
            .iter()
            .any(|index| *index >= abbreviations.len())
        || type_indices
            .iter()
            .any(|index| usize::from(*index) >= types.len())
    {
        return Err(M4SchedulerError::TzifInvalid);
    }
    let transitions = transition_times
        .into_iter()
        .zip(type_indices)
        .map(|(at_utc, index)| M4TimezoneTransition {
            at_utc,
            type_index: usize::from(index),
        })
        .collect::<Vec<_>>();
    if cursor > second_block_end {
        return Err(M4SchedulerError::TzifInvalid);
    }
    let future_rules = m4_parse_tzif_footer(&bytes[second_block_end..])?;
    let default_type_index = types.iter().position(|kind| !kind.is_dst).unwrap_or(0);
    let file_digest = format!("{:x}", Sha256::digest(bytes));
    let timezone_rules_version = m4_internal_id(
        "timezone-rules:",
        "syn.m4.timezone-rules/v1",
        &[iana_timezone, &file_digest],
    )
    .map_err(|_| M4SchedulerError::DeterministicIdFailed)?;
    Ok(M4TimezoneRules {
        iana_timezone: iana_timezone.to_string(),
        timezone_rules_version,
        transitions,
        types,
        default_type_index,
        future_rules,
    })
}

fn m4_parse_tzif_header(
    bytes: &[u8],
    offset: usize,
) -> Result<(M4TzifHeader, usize), M4SchedulerError> {
    let header_end = offset
        .checked_add(44)
        .ok_or(M4SchedulerError::TzifInvalid)?;
    if bytes.get(offset..header_end).is_none()
        || bytes.get(offset..offset + 4) != Some(b"TZif".as_slice())
    {
        return Err(M4SchedulerError::TzifInvalid);
    }
    let version = bytes[offset + 4];
    let mut cursor = offset + 20;
    let header = M4TzifHeader {
        version,
        ttisgmtcnt: usize::try_from(m4_take_u32(bytes, &mut cursor)?)
            .map_err(|_| M4SchedulerError::TzifInvalid)?,
        ttisstdcnt: usize::try_from(m4_take_u32(bytes, &mut cursor)?)
            .map_err(|_| M4SchedulerError::TzifInvalid)?,
        leapcnt: usize::try_from(m4_take_u32(bytes, &mut cursor)?)
            .map_err(|_| M4SchedulerError::TzifInvalid)?,
        timecnt: usize::try_from(m4_take_u32(bytes, &mut cursor)?)
            .map_err(|_| M4SchedulerError::TzifInvalid)?,
        typecnt: usize::try_from(m4_take_u32(bytes, &mut cursor)?)
            .map_err(|_| M4SchedulerError::TzifInvalid)?,
        charcnt: usize::try_from(m4_take_u32(bytes, &mut cursor)?)
            .map_err(|_| M4SchedulerError::TzifInvalid)?,
    };
    if header.typecnt == 0
        || header.ttisstdcnt > header.typecnt
        || header.ttisgmtcnt > header.typecnt
    {
        return Err(M4SchedulerError::TzifInvalid);
    }
    Ok((header, header_end))
}

fn m4_tzif_block_length(
    header: &M4TzifHeader,
    time_size: usize,
) -> Result<usize, M4SchedulerError> {
    let mut total = 0usize;
    for part in [
        header.timecnt.checked_mul(time_size),
        Some(header.timecnt),
        header.typecnt.checked_mul(6),
        Some(header.charcnt),
        header.leapcnt.checked_mul(
            time_size
                .checked_add(4)
                .ok_or(M4SchedulerError::TzifInvalid)?,
        ),
        Some(header.ttisstdcnt),
        Some(header.ttisgmtcnt),
    ] {
        total = total
            .checked_add(part.ok_or(M4SchedulerError::TzifInvalid)?)
            .ok_or(M4SchedulerError::TzifInvalid)?;
    }
    Ok(total)
}

fn m4_take_bytes<'a>(
    bytes: &'a [u8],
    cursor: &mut usize,
    count: usize,
) -> Result<&'a [u8], M4SchedulerError> {
    let end = cursor
        .checked_add(count)
        .ok_or(M4SchedulerError::TzifInvalid)?;
    let output = bytes
        .get(*cursor..end)
        .ok_or(M4SchedulerError::TzifInvalid)?;
    *cursor = end;
    Ok(output)
}

fn m4_take_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, M4SchedulerError> {
    let array: [u8; 4] = m4_take_bytes(bytes, cursor, 4)?
        .try_into()
        .map_err(|_| M4SchedulerError::TzifInvalid)?;
    Ok(u32::from_be_bytes(array))
}

fn m4_take_i32(bytes: &[u8], cursor: &mut usize) -> Result<i32, M4SchedulerError> {
    let array: [u8; 4] = m4_take_bytes(bytes, cursor, 4)?
        .try_into()
        .map_err(|_| M4SchedulerError::TzifInvalid)?;
    Ok(i32::from_be_bytes(array))
}

fn m4_take_i64(bytes: &[u8], cursor: &mut usize) -> Result<i64, M4SchedulerError> {
    let array: [u8; 8] = m4_take_bytes(bytes, cursor, 8)?
        .try_into()
        .map_err(|_| M4SchedulerError::TzifInvalid)?;
    Ok(i64::from_be_bytes(array))
}

fn m4_parse_tzif_footer(bytes: &[u8]) -> Result<M4FutureTimezoneRules, M4SchedulerError> {
    if bytes.is_empty() {
        return Ok(M4FutureTimezoneRules::None);
    }
    if bytes.len() < 2 || bytes[0] != b'\n' || *bytes.last().unwrap_or(&0) != b'\n' {
        return Err(M4SchedulerError::TzifInvalid);
    }
    let value = &bytes[1..bytes.len() - 1];
    if value.is_empty() {
        return Ok(M4FutureTimezoneRules::None);
    }
    let mut parser = M4PosixParser::new(value);
    let _standard_name = parser
        .parse_name()
        .ok_or(M4SchedulerError::TzifFutureRulesInvalid)?;
    let standard_utc_offset_seconds = parser
        .parse_utc_offset()
        .ok_or(M4SchedulerError::TzifFutureRulesInvalid)?;
    if parser.finished() {
        return Ok(M4FutureTimezoneRules::Fixed {
            utc_offset_seconds: standard_utc_offset_seconds,
        });
    }
    let _daylight_name = parser
        .parse_name()
        .ok_or(M4SchedulerError::TzifFutureRulesInvalid)?;
    let daylight_utc_offset_seconds = if parser.peek() == Some(b',') {
        standard_utc_offset_seconds
            .checked_add(3_600)
            .ok_or(M4SchedulerError::TzifFutureRulesInvalid)?
    } else {
        parser
            .parse_utc_offset()
            .ok_or(M4SchedulerError::TzifFutureRulesInvalid)?
    };
    parser
        .expect(b',')
        .ok_or(M4SchedulerError::TzifFutureRulesInvalid)?;
    let start = parser
        .parse_transition_rule()
        .ok_or(M4SchedulerError::TzifFutureRulesInvalid)?;
    parser
        .expect(b',')
        .ok_or(M4SchedulerError::TzifFutureRulesInvalid)?;
    let end = parser
        .parse_transition_rule()
        .ok_or(M4SchedulerError::TzifFutureRulesInvalid)?;
    if !parser.finished() {
        return Err(M4SchedulerError::TzifFutureRulesInvalid);
    }
    Ok(M4FutureTimezoneRules::Dst(M4PosixDstRules {
        standard_utc_offset_seconds,
        daylight_utc_offset_seconds,
        start,
        end,
    }))
}

struct M4PosixParser<'a> {
    input: &'a [u8],
    cursor: usize,
}

impl<'a> M4PosixParser<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, cursor: 0 }
    }

    fn finished(&self) -> bool {
        self.cursor == self.input.len()
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.cursor).copied()
    }

    fn expect(&mut self, expected: u8) -> Option<()> {
        if self.peek() == Some(expected) {
            self.cursor += 1;
            Some(())
        } else {
            None
        }
    }

    fn parse_name(&mut self) -> Option<()> {
        if self.expect(b'<').is_some() {
            let start = self.cursor;
            while self.peek().is_some_and(|byte| byte != b'>') {
                self.cursor += 1;
            }
            if self.cursor == start || self.expect(b'>').is_none() {
                return None;
            }
            return Some(());
        }
        let start = self.cursor;
        while self.peek().is_some_and(|byte| byte.is_ascii_alphabetic()) {
            self.cursor += 1;
        }
        (self.cursor - start >= 3).then_some(())
    }

    fn parse_number(&mut self) -> Option<u32> {
        let start = self.cursor;
        let mut value = 0u32;
        while let Some(byte) = self.peek() {
            if !byte.is_ascii_digit() {
                break;
            }
            value = value.checked_mul(10)?.checked_add(u32::from(byte - b'0'))?;
            self.cursor += 1;
        }
        (self.cursor > start).then_some(value)
    }

    fn parse_utc_offset(&mut self) -> Option<i32> {
        let sign = match self.peek() {
            Some(b'-') => {
                self.cursor += 1;
                -1i64
            }
            Some(b'+') => {
                self.cursor += 1;
                1i64
            }
            _ => 1i64,
        };
        let hour = self.parse_number()?;
        let minute = if self.expect(b':').is_some() {
            self.parse_number()?
        } else {
            0
        };
        let second = if self.expect(b':').is_some() {
            self.parse_number()?
        } else {
            0
        };
        if hour > 24 || minute > 59 || second > 59 {
            return None;
        }
        let seconds_west =
            sign * (i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second));
        i32::try_from(-seconds_west).ok()
    }

    fn parse_transition_rule(&mut self) -> Option<M4PosixTransitionRule> {
        let day = match self.peek()? {
            b'J' => {
                self.cursor += 1;
                let value = self.parse_number()?;
                if !(1..=365).contains(&value) {
                    return None;
                }
                M4PosixRuleDay::JulianNoLeap(value as u16)
            }
            b'M' => {
                self.cursor += 1;
                let month = self.parse_number()?;
                self.expect(b'.')?;
                let week = self.parse_number()?;
                self.expect(b'.')?;
                let weekday = self.parse_number()?;
                if !(1..=12).contains(&month) || !(1..=5).contains(&week) || weekday > 6 {
                    return None;
                }
                M4PosixRuleDay::MonthWeekday {
                    month: month as u8,
                    week: week as u8,
                    weekday: weekday as u8,
                }
            }
            byte if byte.is_ascii_digit() => {
                let value = self.parse_number()?;
                if value > 365 {
                    return None;
                }
                M4PosixRuleDay::DayOfYear(value as u16)
            }
            _ => return None,
        };
        let (seconds, basis) = if self.expect(b'/').is_some() {
            self.parse_transition_time()?
        } else {
            (7_200, M4PosixTimeBasis::Wall)
        };
        Some(M4PosixTransitionRule {
            day,
            seconds,
            basis,
        })
    }

    fn parse_transition_time(&mut self) -> Option<(i32, M4PosixTimeBasis)> {
        let sign = match self.peek() {
            Some(b'-') => {
                self.cursor += 1;
                -1i64
            }
            Some(b'+') => {
                self.cursor += 1;
                1i64
            }
            _ => 1i64,
        };
        let hour = self.parse_number()?;
        let minute = if self.expect(b':').is_some() {
            self.parse_number()?
        } else {
            0
        };
        let second = if self.expect(b':').is_some() {
            self.parse_number()?
        } else {
            0
        };
        if hour > 167 || minute > 59 || second > 59 {
            return None;
        }
        let seconds = i32::try_from(
            sign * (i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second)),
        )
        .ok()?;
        let basis = match self.peek() {
            Some(b'w') => {
                self.cursor += 1;
                M4PosixTimeBasis::Wall
            }
            Some(b's') => {
                self.cursor += 1;
                M4PosixTimeBasis::Standard
            }
            Some(b'u' | b'g' | b'z') => {
                self.cursor += 1;
                M4PosixTimeBasis::Utc
            }
            _ => M4PosixTimeBasis::Wall,
        };
        Some((seconds, basis))
    }
}

/// Immutable server-owned scheduling configuration. A new OS timezone
/// resolution must be assigned a new revision by the composition owner; this
/// type never mutates an existing configuration or an existing window.
#[derive(Clone, Debug)]
pub(crate) struct M4SchedulerConfiguration {
    configuration_revision: u64,
    scope_id: String,
    timezone: M4TimezoneRules,
}

impl M4SchedulerConfiguration {
    pub(crate) fn configuration_revision(&self) -> u64 {
        self.configuration_revision
    }

    pub(crate) fn scope_id(&self) -> &str {
        &self.scope_id
    }

    pub(crate) fn timezone(&self) -> &M4TimezoneRules {
        &self.timezone
    }
}

pub(crate) fn m4_scheduler_configuration(
    configuration_revision: u64,
    scope_id: &str,
    timezone: M4TimezoneRules,
) -> Result<M4SchedulerConfiguration, M4SchedulerError> {
    if scope_id.is_empty()
        || scope_id.len() > 512
        || scope_id != scope_id.trim()
        || scope_id.chars().any(char::is_control)
    {
        return Err(M4SchedulerError::ConfigurationInvalid);
    }
    Ok(M4SchedulerConfiguration {
        configuration_revision,
        scope_id: scope_id.to_string(),
        timezone,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SchedulerDisabledConfiguration {
    configuration_revision: u64,
    error_code: &'static str,
}

impl M4SchedulerDisabledConfiguration {
    pub(crate) fn configuration_revision(&self) -> u64 {
        self.configuration_revision
    }

    pub(crate) fn error_code(&self) -> &'static str {
        self.error_code
    }
}

#[derive(Clone, Debug)]
pub(crate) enum M4SchedulerConfigurationResolution {
    Enabled(M4SchedulerConfiguration),
    Disabled(M4SchedulerDisabledConfiguration),
}

impl M4SchedulerConfigurationResolution {
    pub(crate) fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled(_))
    }
}

/// Resolve configuration from OS state without ever turning a failed lookup
/// into UTC. Consumers persist the returned disabled state and static code.
pub(crate) fn m4_resolve_os_scheduler_configuration(
    configuration_revision: u64,
    scope_id: &str,
    paths: &M4SchedulerOsTimezonePaths,
) -> M4SchedulerConfigurationResolution {
    match m4_resolve_os_timezone(paths)
        .and_then(|timezone| m4_scheduler_configuration(configuration_revision, scope_id, timezone))
    {
        Ok(configuration) => M4SchedulerConfigurationResolution::Enabled(configuration),
        Err(error) => {
            M4SchedulerConfigurationResolution::Disabled(M4SchedulerDisabledConfiguration {
                configuration_revision,
                error_code: error.code(),
            })
        }
    }
}

/// Immutable materialized local-day interval. `daily_window_id` intentionally
/// excludes configuration revision, matching the frozen contract components.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4DailyWindow {
    configuration_revision: u64,
    scope_id: String,
    iana_timezone: String,
    local_date: M4LocalDate,
    window_start_utc: i64,
    window_end_utc: i64,
    utc_offset_at_start_seconds: i32,
    utc_offset_at_end_seconds: i32,
    timezone_rules_version: String,
    daily_window_id: String,
}

impl M4DailyWindow {
    pub(crate) fn configuration_revision(&self) -> u64 {
        self.configuration_revision
    }

    pub(crate) fn scope_id(&self) -> &str {
        &self.scope_id
    }

    pub(crate) fn iana_timezone(&self) -> &str {
        &self.iana_timezone
    }

    pub(crate) fn local_date(&self) -> M4LocalDate {
        self.local_date
    }

    pub(crate) fn window_start_utc(&self) -> i64 {
        self.window_start_utc
    }

    pub(crate) fn window_end_utc(&self) -> i64 {
        self.window_end_utc
    }

    pub(crate) fn utc_offset_at_start_seconds(&self) -> i32 {
        self.utc_offset_at_start_seconds
    }

    pub(crate) fn utc_offset_at_end_seconds(&self) -> i32 {
        self.utc_offset_at_end_seconds
    }

    pub(crate) fn timezone_rules_version(&self) -> &str {
        &self.timezone_rules_version
    }

    pub(crate) fn daily_window_id(&self) -> &str {
        &self.daily_window_id
    }

    pub(crate) fn duration_seconds(&self) -> i64 {
        self.window_end_utc - self.window_start_utc
    }
}

pub(crate) fn m4_daily_window_for_local_date(
    configuration: &M4SchedulerConfiguration,
    local_date: M4LocalDate,
) -> Result<M4DailyWindow, M4SchedulerError> {
    let timezone = configuration.timezone();
    let window_start_utc = timezone.resolve_local_midnight(local_date)?;
    let next_date = local_date.add_days(1)?;
    let window_end_utc = timezone.resolve_local_midnight(next_date)?;
    if window_end_utc <= window_start_utc {
        return Err(M4SchedulerError::LocalBoundaryUnresolvable);
    }
    let start_text = m4_format_utc_seconds(window_start_utc)?;
    let end_text = m4_format_utc_seconds(window_end_utc)?;
    let local_date_text = local_date.canonical();
    let daily_window_id = m4_internal_id(
        "daily-window:",
        "syn.m4.daily-window/v1",
        &[
            configuration.scope_id(),
            timezone.iana_timezone(),
            &local_date_text,
            &start_text,
            &end_text,
            timezone.timezone_rules_version(),
        ],
    )
    .map_err(|_| M4SchedulerError::DeterministicIdFailed)?;
    Ok(M4DailyWindow {
        configuration_revision: configuration.configuration_revision(),
        scope_id: configuration.scope_id().to_string(),
        iana_timezone: timezone.iana_timezone().to_string(),
        local_date,
        window_start_utc,
        window_end_utc,
        utc_offset_at_start_seconds: timezone.offset_at_utc(window_start_utc)?,
        utc_offset_at_end_seconds: timezone.offset_at_utc(window_end_utc)?,
        timezone_rules_version: timezone.timezone_rules_version().to_string(),
        daily_window_id,
    })
}

/// Resolve the timestamp's local calendar date with the frozen timezone rules,
/// then delegate to the sole daily-window constructor. Repository code can use
/// this for current-window reads without copying TZif or DST logic.
pub(crate) fn m4_daily_window_at_utc(
    configuration: &M4SchedulerConfiguration,
    timestamp: i64,
) -> Result<M4DailyWindow, M4SchedulerError> {
    let local_date = configuration.timezone().local_date_at_utc(timestamp)?;
    m4_daily_window_for_local_date(configuration, local_date)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4SchedulerTrigger {
    TimerTick,
    StartupRecovery,
    InternalFailureRecovery,
    CoordinationOnly,
    /// A user-directed request to materialize a previously recorded,
    /// unmaterialized range.  It is deliberately never a model trigger.
    ExplicitCatchUpRecovery,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SchedulerCheckpoint {
    // This is the highest window automatically materialized by the scheduler.
    // It deliberately does not claim that every older date was materialized:
    // a CATCH_UP_TRUNCATED receipt can preserve an older unmaterialized range.
    latest_automatically_materialized_local_date: Option<M4LocalDate>,
    last_tick_utc: Option<i64>,
}

impl M4SchedulerCheckpoint {
    pub(crate) fn new(
        latest_automatically_materialized_local_date: Option<M4LocalDate>,
        last_tick_utc: Option<i64>,
    ) -> Self {
        Self {
            latest_automatically_materialized_local_date,
            last_tick_utc,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct M4SchedulerPlanningInput {
    configuration: M4SchedulerConfiguration,
    trigger: M4SchedulerTrigger,
    now_utc: i64,
    checkpoint: M4SchedulerCheckpoint,
}

impl M4SchedulerPlanningInput {
    pub(crate) fn new(
        configuration: M4SchedulerConfiguration,
        trigger: M4SchedulerTrigger,
        now_utc: i64,
        checkpoint: M4SchedulerCheckpoint,
    ) -> Self {
        Self {
            configuration,
            trigger,
            now_utc,
            checkpoint,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4CatchUpTruncation {
    receipt_code: &'static str,
    unmaterialized_from_local_date: M4LocalDate,
    unmaterialized_through_local_date: M4LocalDate,
    omitted_window_count: u64,
}

impl M4CatchUpTruncation {
    pub(crate) fn receipt_code(&self) -> &'static str {
        self.receipt_code
    }

    pub(crate) fn unmaterialized_from_local_date(&self) -> M4LocalDate {
        self.unmaterialized_from_local_date
    }

    pub(crate) fn unmaterialized_through_local_date(&self) -> M4LocalDate {
        self.unmaterialized_through_local_date
    }

    pub(crate) fn omitted_window_count(&self) -> u64 {
        self.omitted_window_count
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SchedulerRunPlan {
    configuration_revision: u64,
    trigger: M4SchedulerTrigger,
    next_tick_after_seconds: i64,
    windows: Vec<M4DailyWindow>,
    outcome_code: &'static str,
    catch_up_truncation: Option<M4CatchUpTruncation>,
}

impl M4SchedulerRunPlan {
    pub(crate) fn configuration_revision(&self) -> u64 {
        self.configuration_revision
    }

    pub(crate) fn trigger(&self) -> M4SchedulerTrigger {
        self.trigger
    }

    pub(crate) fn next_tick_after_seconds(&self) -> i64 {
        self.next_tick_after_seconds
    }

    pub(crate) fn windows(&self) -> &[M4DailyWindow] {
        &self.windows
    }

    pub(crate) fn outcome_code(&self) -> &'static str {
        self.outcome_code
    }

    pub(crate) fn catch_up_truncation(&self) -> Option<&M4CatchUpTruncation> {
        self.catch_up_truncation.as_ref()
    }
}

/// A server-authorized batch from an already persisted unmaterialized local
/// date range.  Unlike automatic startup recovery, this always starts at the
/// oldest pending date and exposes the next pending date for a later explicit
/// request; it never silently skips older windows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4ExplicitCatchUpRecoveryPlan {
    windows: Vec<M4DailyWindow>,
    next_unmaterialized_local_date: Option<M4LocalDate>,
    remaining_window_count: u64,
    outcome_code: &'static str,
}

impl M4ExplicitCatchUpRecoveryPlan {
    pub(crate) fn windows(&self) -> &[M4DailyWindow] {
        &self.windows
    }

    pub(crate) fn next_unmaterialized_local_date(&self) -> Option<M4LocalDate> {
        self.next_unmaterialized_local_date
    }

    pub(crate) fn remaining_window_count(&self) -> u64 {
        self.remaining_window_count
    }

    pub(crate) fn outcome_code(&self) -> &'static str {
        self.outcome_code
    }
}

pub(crate) fn m4_scheduler_tick_due(last_tick_utc: Option<i64>, now_utc: i64) -> bool {
    last_tick_utc
        .map(|last_tick| now_utc.saturating_sub(last_tick) >= M4_SCHEDULER_TICK_SECONDS)
        .unwrap_or(true)
}

/// Plan one explicitly requested catch-up batch from server-owned pending
/// bounds.  The date range is inclusive and every successful batch contains
/// at most the frozen seven local windows in oldest-first order.
pub(crate) fn m4_plan_explicit_catch_up_recovery(
    configuration: &M4SchedulerConfiguration,
    next_unmaterialized_local_date: M4LocalDate,
    unmaterialized_through_local_date: M4LocalDate,
) -> Result<M4ExplicitCatchUpRecoveryPlan, M4SchedulerError> {
    let first_day = next_unmaterialized_local_date.days_since_unix_epoch()?;
    let last_day = unmaterialized_through_local_date.days_since_unix_epoch()?;
    let inclusive_window_count = last_day
        .checked_sub(first_day)
        .and_then(|count| count.checked_add(1))
        .ok_or(M4SchedulerError::TimestampOutOfRange)?;
    if inclusive_window_count <= 0 {
        return Err(M4SchedulerError::ExplicitCatchUpRangeInvalid);
    }
    let inclusive_window_count =
        u64::try_from(inclusive_window_count).map_err(|_| M4SchedulerError::TimestampOutOfRange)?;
    let maximum_batch_size = u64::try_from(M4_MAXIMUM_CLOSED_WINDOWS_PER_STARTUP)
        .map_err(|_| M4SchedulerError::TimestampOutOfRange)?;
    let batch_count = inclusive_window_count.min(maximum_batch_size);
    let batch_capacity =
        usize::try_from(batch_count).map_err(|_| M4SchedulerError::TimestampOutOfRange)?;
    let mut windows = Vec::with_capacity(batch_capacity);
    for offset in 0..batch_count {
        let offset = i64::try_from(offset).map_err(|_| M4SchedulerError::TimestampOutOfRange)?;
        let local_date = next_unmaterialized_local_date.add_days(offset)?;
        windows.push(m4_daily_window_for_local_date(configuration, local_date)?);
    }
    let remaining_window_count = inclusive_window_count
        .checked_sub(batch_count)
        .ok_or(M4SchedulerError::TimestampOutOfRange)?;
    let next_unmaterialized_local_date = if remaining_window_count == 0 {
        None
    } else {
        let batch_count =
            i64::try_from(batch_count).map_err(|_| M4SchedulerError::TimestampOutOfRange)?;
        Some(next_unmaterialized_local_date.add_days(batch_count)?)
    };
    Ok(M4ExplicitCatchUpRecoveryPlan {
        windows,
        next_unmaterialized_local_date,
        remaining_window_count,
        outcome_code: if remaining_window_count == 0 {
            M4_EXPLICIT_CATCH_UP_RECOVERED
        } else {
            M4_CATCH_UP_RECOVERY_PARTIAL
        },
    })
}

/// Pure scheduler planner. A timer waits for five minutes after the current
/// local midnight before closing the prior day. Startup/recovery materializes
/// at most the most recent seven eligible windows, ordered from old to new
/// within that selected set. Earlier eligible windows remain unmaterialized and
/// are represented by the mandatory `CATCH_UP_TRUNCATED` receipt directive.
pub(crate) fn m4_plan_scheduler_run(
    input: &M4SchedulerPlanningInput,
) -> Result<M4SchedulerRunPlan, M4SchedulerError> {
    let configuration = &input.configuration;
    let empty = |outcome_code| M4SchedulerRunPlan {
        configuration_revision: configuration.configuration_revision(),
        trigger: input.trigger,
        next_tick_after_seconds: M4_SCHEDULER_TICK_SECONDS,
        windows: Vec::new(),
        outcome_code,
        catch_up_truncation: None,
    };
    if input.trigger == M4SchedulerTrigger::CoordinationOnly {
        return Ok(empty("COORDINATION_ONLY"));
    }
    if input.trigger == M4SchedulerTrigger::TimerTick
        && !m4_scheduler_tick_due(input.checkpoint.last_tick_utc, input.now_utc)
    {
        return Ok(empty("TICK_NOT_DUE"));
    }

    let current_window = m4_daily_window_at_utc(configuration, input.now_utc)?;
    let current_local_date = current_window.local_date();
    let grace_ends_at = current_window
        .window_start_utc()
        .checked_add(M4_DAILY_CLOSE_GRACE_SECONDS)
        .ok_or(M4SchedulerError::TimestampOutOfRange)?;
    let latest_eligible_local_date = if input.now_utc >= grace_ends_at {
        current_local_date.previous_day()?
    } else {
        current_local_date.previous_day()?.previous_day()?
    };

    let first_unclosed_local_date = match input
        .checkpoint
        .latest_automatically_materialized_local_date
    {
        Some(date) if date >= latest_eligible_local_date => return Ok(empty("NO_ELIGIBLE_WINDOW")),
        Some(date) => date.add_days(1)?,
        // With no persisted checkpoint there is no evidence for an older
        // pending interval. Schedule only the latest eligible window.
        None => latest_eligible_local_date,
    };
    let total_window_count = latest_eligible_local_date
        .days_since_unix_epoch()?
        .checked_sub(first_unclosed_local_date.days_since_unix_epoch()?)
        .and_then(|value| value.checked_add(1))
        .ok_or(M4SchedulerError::TimestampOutOfRange)?;
    if total_window_count <= 0 {
        return Ok(empty("NO_ELIGIBLE_WINDOW"));
    }
    let total_window_count =
        u64::try_from(total_window_count).map_err(|_| M4SchedulerError::TimestampOutOfRange)?;
    let materialized_count = usize::try_from(total_window_count)
        .unwrap_or(usize::MAX)
        .min(M4_MAXIMUM_CLOSED_WINDOWS_PER_STARTUP);
    let omitted_window_count = total_window_count - materialized_count as u64;
    let first_materialized_local_date = first_unclosed_local_date.add_days(
        i64::try_from(omitted_window_count).map_err(|_| M4SchedulerError::TimestampOutOfRange)?,
    )?;
    let mut windows = Vec::with_capacity(materialized_count);
    for offset in 0..materialized_count {
        let date = first_materialized_local_date
            .add_days(i64::try_from(offset).map_err(|_| M4SchedulerError::TimestampOutOfRange)?)?;
        windows.push(m4_daily_window_for_local_date(configuration, date)?);
    }
    let catch_up_truncation = if omitted_window_count > 0 {
        Some(M4CatchUpTruncation {
            receipt_code: M4_CATCH_UP_TRUNCATED,
            unmaterialized_from_local_date: first_unclosed_local_date,
            unmaterialized_through_local_date: first_materialized_local_date.previous_day()?,
            omitted_window_count,
        })
    } else {
        None
    };
    Ok(M4SchedulerRunPlan {
        configuration_revision: configuration.configuration_revision(),
        trigger: input.trigger,
        next_tick_after_seconds: M4_SCHEDULER_TICK_SECONDS,
        windows,
        outcome_code: if catch_up_truncation.is_some() {
            M4_CATCH_UP_TRUNCATED
        } else {
            "WINDOWS_PLANNED"
        },
        catch_up_truncation,
    })
}

/// Input snapshot for determining whether a model *may* be asked for a named
/// enhancement. It is not an invocation request and contains no provider data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4MaterialEventInput {
    trigger: M4SchedulerTrigger,
    explicit_user_message: bool,
    admitted_material_event_count: u64,
    scope_source_watermark_before: String,
    scope_source_watermark_after: String,
}

impl M4MaterialEventInput {
    pub(crate) fn new(
        trigger: M4SchedulerTrigger,
        explicit_user_message: bool,
        admitted_material_event_count: u64,
        scope_source_watermark_before: &str,
        scope_source_watermark_after: &str,
    ) -> Self {
        Self {
            trigger,
            explicit_user_message,
            admitted_material_event_count,
            scope_source_watermark_before: scope_source_watermark_before.to_string(),
            scope_source_watermark_after: scope_source_watermark_after.to_string(),
        }
    }

    fn has_material_source_change(&self) -> bool {
        self.admitted_material_event_count > 0
            && self.scope_source_watermark_before != self.scope_source_watermark_after
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4ModelEligibilityInput {
    material_event: M4MaterialEventInput,
    named_enhancement_purpose: Option<String>,
}

impl M4ModelEligibilityInput {
    pub(crate) fn new(
        material_event: M4MaterialEventInput,
        named_enhancement_purpose: Option<&str>,
    ) -> Self {
        Self {
            material_event,
            named_enhancement_purpose: named_enhancement_purpose.map(str::to_string),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4ModelEligibilityBasis {
    ExplicitUserMessage,
    MaterialAdmittedSourceEvent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M4ModelEligibility {
    Ineligible {
        code: &'static str,
    },
    Eligible {
        basis: M4ModelEligibilityBasis,
        named_enhancement_purpose: String,
    },
}

impl M4ModelEligibility {
    pub(crate) fn is_eligible(&self) -> bool {
        matches!(self, Self::Eligible { .. })
    }
}

pub(crate) fn m4_decide_model_eligibility(input: &M4ModelEligibilityInput) -> M4ModelEligibility {
    let material = &input.material_event;
    if material.trigger == M4SchedulerTrigger::ExplicitCatchUpRecovery {
        return M4ModelEligibility::Ineligible {
            code: m4_model_ineligible_trigger_code(material.trigger),
        };
    }
    let Some(purpose) = input.named_enhancement_purpose.as_deref() else {
        return M4ModelEligibility::Ineligible {
            code: m4_model_ineligible_trigger_code(material.trigger),
        };
    };
    if !m4_is_named_enhancement_purpose(purpose) {
        return M4ModelEligibility::Ineligible {
            code: "m4_model_enhancement_purpose_invalid",
        };
    }
    if material.explicit_user_message {
        return M4ModelEligibility::Eligible {
            basis: M4ModelEligibilityBasis::ExplicitUserMessage,
            named_enhancement_purpose: purpose.to_string(),
        };
    }
    if material.has_material_source_change() {
        return M4ModelEligibility::Eligible {
            basis: M4ModelEligibilityBasis::MaterialAdmittedSourceEvent,
            named_enhancement_purpose: purpose.to_string(),
        };
    }
    M4ModelEligibility::Ineligible {
        code: m4_model_ineligible_trigger_code(material.trigger),
    }
}

fn m4_model_ineligible_trigger_code(trigger: M4SchedulerTrigger) -> &'static str {
    match trigger {
        M4SchedulerTrigger::TimerTick => "m4_model_ineligible_timer_only",
        M4SchedulerTrigger::StartupRecovery | M4SchedulerTrigger::InternalFailureRecovery => {
            "m4_model_ineligible_recovery_only"
        }
        M4SchedulerTrigger::CoordinationOnly => "m4_model_ineligible_coordination_only",
        M4SchedulerTrigger::ExplicitCatchUpRecovery => {
            "m4_model_ineligible_explicit_catch_up_recovery"
        }
    }
}

fn m4_is_named_enhancement_purpose(value: &str) -> bool {
    (1..=96).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':'))
}

/// A ledger-ready, pure decision. Eligible means a separate service may make
/// one named enhancement turn; this module still does not make that call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SchedulerRunDecision {
    configuration_revision: u64,
    window: M4DailyWindow,
    material_event: M4MaterialEventInput,
    model_eligibility: M4ModelEligibility,
    agent_turn_count: u64,
    model_invocation_count: u64,
}

impl M4SchedulerRunDecision {
    pub(crate) fn configuration_revision(&self) -> u64 {
        self.configuration_revision
    }

    pub(crate) fn window(&self) -> &M4DailyWindow {
        &self.window
    }

    pub(crate) fn material_event(&self) -> &M4MaterialEventInput {
        &self.material_event
    }

    pub(crate) fn model_eligibility(&self) -> &M4ModelEligibility {
        &self.model_eligibility
    }

    pub(crate) fn agent_turn_count(&self) -> u64 {
        self.agent_turn_count
    }

    pub(crate) fn model_invocation_count(&self) -> u64 {
        self.model_invocation_count
    }
}

pub(crate) fn m4_scheduler_run_decision(
    configuration: &M4SchedulerConfiguration,
    window: M4DailyWindow,
    eligibility_input: M4ModelEligibilityInput,
) -> Result<M4SchedulerRunDecision, M4SchedulerError> {
    if configuration.configuration_revision() != window.configuration_revision() {
        return Err(M4SchedulerError::SchedulerRunConfigurationMismatch);
    }
    let material_event = eligibility_input.material_event.clone();
    let model_eligibility = m4_decide_model_eligibility(&eligibility_input);
    let model_count = u64::from(model_eligibility.is_eligible());
    Ok(M4SchedulerRunDecision {
        configuration_revision: configuration.configuration_revision(),
        window,
        material_event,
        model_eligibility,
        agent_turn_count: model_count,
        model_invocation_count: model_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn system_zoneinfo_root() -> PathBuf {
        [
            "/var/db/timezone/zoneinfo",
            "/usr/share/zoneinfo",
            "/usr/share/lib/zoneinfo",
        ]
        .iter()
        .map(PathBuf::from)
        .find(|root| {
            root.join("Asia/Shanghai").is_file() && root.join("America/New_York").is_file()
        })
        .expect("local system zoneinfo must include Asia/Shanghai and America/New_York")
    }

    fn timezone(name: &str) -> M4TimezoneRules {
        m4_load_timezone_from_zoneinfo_root(&system_zoneinfo_root(), name)
            .expect("load local system TZif")
    }

    fn config(name: &str) -> M4SchedulerConfiguration {
        m4_scheduler_configuration(41, "scope:personal:primary", timezone(name))
            .expect("build synthetic scheduler configuration")
    }

    fn date(year: i32, month: u8, day: u8) -> M4LocalDate {
        M4LocalDate::new(year, month, day).expect("valid local date")
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos();
        let path = env::temp_dir().join(format!("syn-m4c07-{label}-{}-{nanos}", process::id()));
        fs::create_dir_all(&path).expect("create test temp directory");
        path
    }

    fn synthetic_tzif_v2_or_v3(version: u8) -> Vec<u8> {
        fn append_header(output: &mut Vec<u8>, version: u8, counts: [u32; 6]) {
            output.extend_from_slice(b"TZif");
            output.push(version);
            output.extend_from_slice(&[0; 15]);
            for count in counts {
                output.extend_from_slice(&count.to_be_bytes());
            }
        }
        fn append_type(output: &mut Vec<u8>, offset: i32, is_dst: u8, abbreviation: u8) {
            output.extend_from_slice(&offset.to_be_bytes());
            output.push(is_dst);
            output.push(abbreviation);
        }

        let mut output = Vec::new();
        // The v1 block deliberately has no transitions. A parser that ignores
        // the v2/v3 64-bit block will therefore fail the assertion below.
        append_header(&mut output, version, [0, 0, 0, 0, 1, 4]);
        append_type(&mut output, 0, 0, 0);
        output.extend_from_slice(b"UTC\0");

        append_header(&mut output, version, [0, 0, 0, 1, 2, 8]);
        output.extend_from_slice(&2_200_000_000i64.to_be_bytes());
        output.push(1);
        append_type(&mut output, 0, 0, 0);
        append_type(&mut output, 3_600, 1, 4);
        output.extend_from_slice(b"UTC\0DST\0");
        output.extend_from_slice(b"\nFOO-1\n");
        output
    }

    #[test]
    fn parses_v2_and_v3_64_bit_transition_blocks() {
        for version in [b'2', b'3'] {
            let directory = unique_temp_dir("tzif");
            let root = directory.join("zoneinfo");
            let zone = root.join("Test/Zone");
            fs::create_dir_all(zone.parent().expect("zone parent")).expect("create zone parent");
            fs::write(&zone, synthetic_tzif_v2_or_v3(version)).expect("write synthetic TZif");
            let rules = m4_load_timezone_from_zoneinfo_root(&root, "Test/Zone")
                .expect("parse synthetic TZif");
            assert_eq!(rules.offset_at_utc(2_200_000_000).unwrap(), 3_600);
            let _ = fs::remove_dir_all(directory);
        }
    }

    #[test]
    fn asia_shanghai_has_one_24_hour_local_day_and_stable_rules_version() {
        let configuration = config("Asia/Shanghai");
        let window = m4_daily_window_for_local_date(&configuration, date(2024, 6, 1)).unwrap();
        assert_eq!(window.duration_seconds(), 86_400);
        assert_eq!(window.iana_timezone(), "Asia/Shanghai");
        assert_eq!(
            configuration.timezone().timezone_rules_version(),
            timezone("Asia/Shanghai").timezone_rules_version()
        );
    }

    #[test]
    fn new_york_dst_spring_and_fall_days_are_23_and_25_hours() {
        let configuration = config("America/New_York");
        let spring = m4_daily_window_for_local_date(&configuration, date(2024, 3, 10)).unwrap();
        let fall = m4_daily_window_for_local_date(&configuration, date(2024, 11, 3)).unwrap();
        assert_eq!(spring.duration_seconds(), 23 * 3_600);
        assert_eq!(fall.duration_seconds(), 25 * 3_600);
        assert_eq!(
            m4_format_utc_seconds(spring.window_start_utc()).unwrap(),
            "2024-03-10T05:00:00Z"
        );
        assert_eq!(
            m4_format_utc_seconds(spring.window_end_utc()).unwrap(),
            "2024-03-11T04:00:00Z"
        );
        assert_eq!(
            m4_format_utc_seconds(fall.window_start_utc()).unwrap(),
            "2024-11-03T04:00:00Z"
        );
        assert_eq!(
            m4_format_utc_seconds(fall.window_end_utc()).unwrap(),
            "2024-11-04T05:00:00Z"
        );
    }

    #[test]
    fn same_window_inputs_reuse_the_contract_daily_window_id() {
        let configuration = config("America/New_York");
        let left = m4_daily_window_for_local_date(&configuration, date(2024, 11, 3)).unwrap();
        let right = m4_daily_window_for_local_date(&configuration, date(2024, 11, 3)).unwrap();
        assert_eq!(left.daily_window_id(), right.daily_window_id());
        assert!(left.daily_window_id().starts_with("daily-window:"));
    }

    #[test]
    fn utc_window_entry_reuses_the_local_day_constructor_and_canonical_formatter() {
        let configuration = config("America/New_York");
        let timestamp = m4_utc_seconds(date(2024, 3, 10), 12, 34, 56).unwrap();
        let from_utc = m4_daily_window_at_utc(&configuration, timestamp).unwrap();
        let from_local = m4_daily_window_for_local_date(&configuration, date(2024, 3, 10)).unwrap();
        assert_eq!(from_utc.daily_window_id(), from_local.daily_window_id());
        assert_eq!(
            m4_format_utc_seconds(timestamp).unwrap(),
            "2024-03-10T12:34:56Z"
        );
    }

    #[test]
    fn parse_utc_seconds_strictly_accepts_m4_utc_z_and_floors_fractional_seconds() {
        let canonical = "2024-02-29T23:59:59Z";
        let expected = m4_utc_seconds(date(2024, 2, 29), 23, 59, 59).unwrap();
        assert_eq!(m4_parse_utc_seconds(canonical).unwrap(), expected);
        assert_eq!(
            m4_parse_utc_seconds("2024-02-29T23:59:59.1Z").unwrap(),
            expected
        );
        assert_eq!(
            m4_parse_utc_seconds("2024-02-29T23:59:59.123456789Z").unwrap(),
            expected
        );
        assert_eq!(m4_format_utc_seconds(expected).unwrap(), canonical);
        assert_eq!(m4_parse_utc_seconds("1969-12-31T23:59:59.9Z").unwrap(), -1);

        for invalid in [
            "2026-02-29T00:00:00Z",
            "2024-01-01T24:00:00Z",
            "2024-01-01T00:00:60Z",
            "2024-01-01T00:00:00.Z",
            "2024-01-01T00:00:00.1234567890Z",
            "2024-01-01T00:00:00+08:00",
        ] {
            assert_eq!(
                m4_parse_utc_seconds(invalid).unwrap_err().code(),
                "m4_scheduler_utc_timestamp_invalid"
            );
        }
    }

    #[test]
    fn startup_catch_up_keeps_older_windows_unmaterialized_and_runs_recent_seven_oldest_first() {
        let configuration = config("Asia/Shanghai");
        // 2024-06-10 00:06 in Asia/Shanghai: the June 9 window is eligible.
        let now_utc = m4_utc_seconds(date(2024, 6, 9), 16, 6, 0).unwrap();
        let input = M4SchedulerPlanningInput::new(
            configuration,
            M4SchedulerTrigger::StartupRecovery,
            now_utc,
            M4SchedulerCheckpoint::new(Some(date(2024, 5, 30)), None),
        );
        let plan = m4_plan_scheduler_run(&input).unwrap();
        assert_eq!(plan.windows().len(), M4_MAXIMUM_CLOSED_WINDOWS_PER_STARTUP);
        assert_eq!(
            plan.windows().first().unwrap().local_date(),
            date(2024, 6, 3)
        );
        assert_eq!(
            plan.windows().last().unwrap().local_date(),
            date(2024, 6, 9)
        );
        assert_eq!(plan.outcome_code(), M4_CATCH_UP_TRUNCATED);
        let truncation = plan.catch_up_truncation().expect("truncation receipt");
        assert_eq!(truncation.receipt_code(), M4_CATCH_UP_TRUNCATED);
        assert_eq!(
            truncation.unmaterialized_from_local_date(),
            date(2024, 5, 31)
        );
        assert_eq!(
            truncation.unmaterialized_through_local_date(),
            date(2024, 6, 2)
        );
        assert_eq!(truncation.omitted_window_count(), 3);
    }

    #[test]
    fn explicit_catch_up_recovery_materializes_a_short_range_oldest_first() {
        let configuration = config("Asia/Shanghai");
        let plan =
            m4_plan_explicit_catch_up_recovery(&configuration, date(2024, 6, 1), date(2024, 6, 3))
                .expect("plan three explicit catch-up windows");
        assert_eq!(
            plan.windows()
                .iter()
                .map(|window| window.local_date())
                .collect::<Vec<_>>(),
            vec![date(2024, 6, 1), date(2024, 6, 2), date(2024, 6, 3)]
        );
        assert_eq!(plan.next_unmaterialized_local_date(), None);
        assert_eq!(plan.remaining_window_count(), 0);
        assert_eq!(plan.outcome_code(), M4_EXPLICIT_CATCH_UP_RECOVERED);
    }

    #[test]
    fn explicit_catch_up_recovery_resumes_a_ten_window_range_in_two_batches() {
        let configuration = config("Asia/Shanghai");
        let through = date(2024, 6, 10);
        let first = m4_plan_explicit_catch_up_recovery(&configuration, date(2024, 6, 1), through)
            .expect("plan first explicit seven-window batch");
        assert_eq!(first.windows().len(), M4_MAXIMUM_CLOSED_WINDOWS_PER_STARTUP);
        assert_eq!(
            first.windows().first().unwrap().local_date(),
            date(2024, 6, 1)
        );
        assert_eq!(
            first.windows().last().unwrap().local_date(),
            date(2024, 6, 7)
        );
        assert_eq!(
            first.next_unmaterialized_local_date(),
            Some(date(2024, 6, 8))
        );
        assert_eq!(first.remaining_window_count(), 3);
        assert_eq!(first.outcome_code(), M4_CATCH_UP_RECOVERY_PARTIAL);

        let second = m4_plan_explicit_catch_up_recovery(
            &configuration,
            first
                .next_unmaterialized_local_date()
                .expect("partial batch supplies next date"),
            through,
        )
        .expect("plan second explicit catch-up batch");
        assert_eq!(
            second
                .windows()
                .iter()
                .map(|window| window.local_date())
                .collect::<Vec<_>>(),
            vec![date(2024, 6, 8), date(2024, 6, 9), date(2024, 6, 10)]
        );
        assert_eq!(second.next_unmaterialized_local_date(), None);
        assert_eq!(second.remaining_window_count(), 0);
        assert_eq!(second.outcome_code(), M4_EXPLICIT_CATCH_UP_RECOVERED);
    }

    #[test]
    fn explicit_catch_up_recovery_keeps_dst_window_ids_and_boundaries_stable() {
        let configuration = config("America/New_York");
        let plan =
            m4_plan_explicit_catch_up_recovery(&configuration, date(2024, 3, 9), date(2024, 3, 11))
                .expect("plan range spanning spring-forward day");
        assert_eq!(
            plan.windows()
                .iter()
                .map(|window| window.local_date())
                .collect::<Vec<_>>(),
            vec![date(2024, 3, 9), date(2024, 3, 10), date(2024, 3, 11)]
        );
        assert_eq!(plan.windows()[0].duration_seconds(), 24 * 3_600);
        assert_eq!(plan.windows()[1].duration_seconds(), 23 * 3_600);
        assert_eq!(plan.windows()[2].duration_seconds(), 24 * 3_600);
        for window in plan.windows() {
            assert_eq!(
                window.daily_window_id(),
                m4_daily_window_for_local_date(&configuration, window.local_date())
                    .expect("rebuild exact DST daily window")
                    .daily_window_id()
            );
        }
    }

    #[test]
    fn explicit_catch_up_recovery_rejects_reversed_ranges_and_is_always_model_ineligible() {
        let configuration = config("Asia/Shanghai");
        assert_eq!(
            m4_plan_explicit_catch_up_recovery(&configuration, date(2024, 6, 2), date(2024, 6, 1),)
                .unwrap_err(),
            M4SchedulerError::ExplicitCatchUpRangeInvalid
        );

        let window = m4_daily_window_for_local_date(&configuration, date(2024, 6, 1))
            .expect("build explicit catch-up decision window");
        let material = M4MaterialEventInput::new(
            M4SchedulerTrigger::ExplicitCatchUpRecovery,
            true,
            1,
            "watermark:before",
            "watermark:after",
        );
        let decision = m4_scheduler_run_decision(
            &configuration,
            window,
            M4ModelEligibilityInput::new(material, Some("EXPLAIN_ATTENTION_REASON")),
        )
        .expect("build explicit catch-up decision");
        assert_eq!(decision.agent_turn_count(), 0);
        assert_eq!(decision.model_invocation_count(), 0);
        assert_eq!(
            decision.model_eligibility(),
            &M4ModelEligibility::Ineligible {
                code: "m4_model_ineligible_explicit_catch_up_recovery"
            }
        );
    }

    #[test]
    fn timer_uses_a_60_second_cadence_and_waits_for_the_five_minute_midnight_grace() {
        assert!(!m4_scheduler_tick_due(Some(1_000), 1_059));
        assert!(m4_scheduler_tick_due(Some(1_000), 1_060));

        let configuration = config("Asia/Shanghai");
        let checkpoint = M4SchedulerCheckpoint::new(Some(date(2024, 6, 8)), None);
        let before_grace = m4_utc_seconds(date(2024, 6, 9), 16, 4, 0).unwrap();
        let before_plan = m4_plan_scheduler_run(&M4SchedulerPlanningInput::new(
            configuration.clone(),
            M4SchedulerTrigger::TimerTick,
            before_grace,
            M4SchedulerCheckpoint::new(
                checkpoint.latest_automatically_materialized_local_date,
                Some(before_grace - M4_SCHEDULER_TICK_SECONDS),
            ),
        ))
        .unwrap();
        assert!(before_plan.windows().is_empty());

        let at_grace = m4_utc_seconds(date(2024, 6, 9), 16, 5, 0).unwrap();
        let at_grace_plan = m4_plan_scheduler_run(&M4SchedulerPlanningInput::new(
            configuration,
            M4SchedulerTrigger::TimerTick,
            at_grace,
            M4SchedulerCheckpoint::new(
                checkpoint.latest_automatically_materialized_local_date,
                Some(at_grace - M4_SCHEDULER_TICK_SECONDS),
            ),
        ))
        .unwrap();
        assert_eq!(at_grace_plan.windows().len(), 1);
        assert_eq!(at_grace_plan.windows()[0].local_date(), date(2024, 6, 9));
    }

    #[test]
    fn empty_timer_window_has_exactly_zero_agent_turns_and_model_invocations() {
        let configuration = config("Asia/Shanghai");
        let window = m4_daily_window_for_local_date(&configuration, date(2024, 6, 1)).unwrap();
        let material = M4MaterialEventInput::new(
            M4SchedulerTrigger::TimerTick,
            false,
            0,
            "watermark:unchanged",
            "watermark:unchanged",
        );
        let decision = m4_scheduler_run_decision(
            &configuration,
            window,
            M4ModelEligibilityInput::new(material, None),
        )
        .unwrap();
        assert_eq!(decision.agent_turn_count(), 0);
        assert_eq!(decision.model_invocation_count(), 0);
        assert_eq!(
            decision.model_eligibility(),
            &M4ModelEligibility::Ineligible {
                code: "m4_model_ineligible_timer_only"
            }
        );
    }

    #[test]
    fn invalid_os_timezone_disables_scheduler_without_utc_substitution() {
        let directory = unique_temp_dir("invalid-zone");
        let root = directory.join("zoneinfo");
        fs::create_dir_all(&root).expect("create empty zoneinfo root");
        let timezone_file = directory.join("timezone");
        fs::write(&timezone_file, "UTC\n").expect("write invalid non-IANA timezone");
        let resolution = m4_resolve_os_scheduler_configuration(
            9,
            "scope:personal:primary",
            &M4SchedulerOsTimezonePaths {
                localtime_path: directory.join("missing-localtime"),
                timezone_file_path: timezone_file,
                zoneinfo_roots: vec![root],
            },
        );
        assert!(!resolution.is_enabled());
        match resolution {
            M4SchedulerConfigurationResolution::Disabled(disabled) => {
                assert_eq!(disabled.configuration_revision(), 9);
                assert_eq!(disabled.error_code(), "m4_scheduler_timezone_invalid");
            }
            M4SchedulerConfigurationResolution::Enabled(_) => {
                panic!("invalid timezone must disable")
            }
        }
        let _ = fs::remove_dir_all(directory);
    }
}

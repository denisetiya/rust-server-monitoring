use chrono::{DateTime, FixedOffset, NaiveDate, Utc};

// WIB (Western Indonesia Time) is UTC+7
const WIB_OFFSET_HOURS: i32 = 7;

/// Get current time in WIB (Western Indonesia Time - UTC+7)
pub fn now_wib() -> DateTime<FixedOffset> {
    let utc_now = Utc::now();
    let wib_offset = FixedOffset::east_opt(WIB_OFFSET_HOURS * 3600).unwrap();
    utc_now.with_timezone(&wib_offset)
}

/// Get current date in WIB
pub fn today_wib() -> NaiveDate {
    now_wib().date_naive()
}

/// Convert UTC DateTime to WIB
pub fn utc_to_wib(utc_time: DateTime<Utc>) -> DateTime<FixedOffset> {
    let wib_offset = FixedOffset::east_opt(WIB_OFFSET_HOURS * 3600).unwrap();
    utc_time.with_timezone(&wib_offset)
}

/// Format current WIB time as string
pub fn format_wib_now(format: &str) -> String {
    now_wib().format(format).to_string()
}

/// Format WIB time with default format for logging
pub fn format_wib_log() -> String {
    format_wib_now("%Y-%m-%d %H:%M:%S WIB")
}

/// Format WIB time for email subjects and alerts
pub fn format_wib_alert() -> String {
    format_wib_now("%Y-%m-%d %H:%M:%S")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wib_offset() {
        let wib_time = now_wib();

        // WIB should have +7 hours offset from UTC
        assert_eq!(wib_time.offset().local_minus_utc(), 7 * 3600);

        // Test that the offset is correctly applied
        let utc_time = Utc::now();
        let wib_from_utc = utc_to_wib(utc_time);
        assert_eq!(wib_from_utc.offset().local_minus_utc(), 7 * 3600);

        // The timestamps should be the same (same instant in time)
        assert_eq!(
            wib_time.timestamp(),
            wib_time.with_timezone(&Utc).timestamp()
        );
    }
}

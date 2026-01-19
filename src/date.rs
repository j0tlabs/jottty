use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Local, NaiveDate};

///today_date returns the current date in "YYYY-MM-DD" format.
pub fn today_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

/// now_nanos returns the current time in nanoseconds since the UNIX epoch.
pub fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

/// page_id_for generates a page ID string for a given date.
pub fn page_id_for(date: &str) -> String {
    format!("page:{}", date)
}

pub fn today_date_formatted() -> String {
    Local::now().format("%B %d, %Y").to_string()
}

pub fn date_str_format(date: &str) -> String {
    if let Ok(naive_date) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        naive_date.format("%B %d, %Y").to_string()
    } else {
        date.to_string()
    }
}

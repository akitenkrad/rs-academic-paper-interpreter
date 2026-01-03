use chrono::{DateTime, Local, TimeZone};
use indicatif::{ProgressBar, ProgressStyle};

/// Creates and configures a new `ProgressBar` with a custom style and optional message.
///
/// This function initializes a progress bar with a spinner, percentage display,
/// and customizable message. The progress bar uses Unicode block characters
/// for visual representation.
///
/// # Arguments
///
/// * `total` - The total number of steps for the progress bar.
/// * `msg` - An optional message to display alongside the progress bar.
///           Defaults to "Processing" if `None` is provided.
///
/// # Returns
///
/// A configured `ProgressBar` instance ready for use.
///
/// # Examples
///
/// ```ignore
/// let pb = generate_progress_bar(100, Some("Downloading...".to_string()));
/// for _ in 0..100 {
///     pb.inc(1);
/// }
/// pb.finish();
/// ```
pub fn generate_progress_bar(total: usize, msg: Option<String>) -> ProgressBar {
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{wide_bar}] {percent}% ({msg})")
            .unwrap()
            .progress_chars("█▓▒░")
            .tick_chars("⠋⠙⠚⠉"),
    );
    if let Some(m) = msg {
        pb.set_message(m);
    } else {
        pb.set_message("Processing".to_string());
    }
    pb
}

/// Parses a date string into a `DateTime<Local>` object.
///
/// This function attempts to parse the input string using multiple date formats:
/// - RFC 2822 format (e.g., "Tue, 1 Jul 2003 10:52:37 +0200")
/// - RFC 3339 format (e.g., "2003-07-01T10:52:37+02:00")
/// - ISO 8601 date only (e.g., "2003-07-01")
/// - ISO 8601 date and time (e.g., "2003-07-01 10:52:37")
/// - ISO 8601 with timezone offset (e.g., "2003-07-01 10:52:37+0200")
///
/// # Arguments
///
/// * `date_str` - A string slice containing the date to parse.
///
/// # Returns
///
/// A `DateTime<Local>` representing the parsed date. If parsing fails or the input
/// is empty, returns Unix epoch (1970-01-01 00:00:00) as a fallback.
///
/// # Examples
///
/// ```ignore
/// let date = datetime_from_str("2023-12-01");
/// let date = datetime_from_str("2023-12-01 15:30:00");
/// let date = datetime_from_str(""); // Returns Unix epoch
/// ```
pub fn datetime_from_str(date_str: &str) -> DateTime<Local> {
    if date_str.is_empty() {
        return Local.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap(); // Fallback to a default date
    }
    if let Ok(parsed) = DateTime::parse_from_rfc2822(date_str) {
        return parsed.with_timezone(&Local);
    } else if let Ok(parsed) = DateTime::parse_from_rfc3339(date_str) {
        return parsed.with_timezone(&Local);
    }

    let mut date_str = date_str.to_string();
    if regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$")
        .unwrap()
        .is_match(&date_str)
    {
        date_str.push_str(" 00:00:00+0000");
    } else if regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$")
        .unwrap()
        .is_match(&date_str)
    {
        date_str.push_str("+0000");
    } else if !date_str.ends_with('+') && !date_str.ends_with('-') {
        date_str.push_str("+0000");
    } else if regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\+\d{4}$")
        .unwrap()
        .is_match(&date_str)
    {
        // Already in the correct format
    } else {
        eprintln!(
            "WARNING: Date string does not match expected formats: {}",
            date_str
        );
        return Local.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap(); // Fallback to a default date
    }
    match DateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S%z") {
        Ok(date) => date.with_timezone(&Local),
        Err(e) => {
            eprintln!(
                "WARNING: Failed to parse date string: {}. Error: {}",
                date_str, e
            );
            Local.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap() // Fallback to a default date
        }
    }
}

//! Utilities that can be shared between commands and components.

pub mod constants;
pub mod time_parsers;

/// Converts a time in seconds to a string.
pub fn time_to_string(seconds: u64) -> String {
    if seconds < 60 {
        return format!("00:{:02}", seconds);
    } else if seconds < 60 * 60 {
        let time = seconds as f32;
        let minutes = (time / 60.0).floor();
        let seconds = time - minutes * 60.0;
        return format!("{:02}:{:02}", minutes as u32, seconds as u32);
    }

    let time = seconds as f32;
    let hours = (time / 60.0 / 60.0).floor();
    let minutes = (time - hours * 60.0 * 60.0).floor();
    let seconds = time - minutes * 60.0 - hours * 60.0 * 60.0;
    format!(
        "{:02}:{:02}:{:02}",
        hours as u32, minutes as u32, seconds as u32
    )
}

/// Creates a progress bar.
pub fn progress_bar(current: u64, total: u64) -> String {
    let item_total = 30usize;
    let item_count = (current as f32 / (total as f32 / item_total as f32)).round();
    let bar = "▓".repeat(item_count as usize);
    format!("╣{:░<width$.width$}╠", bar, width = item_total)
}

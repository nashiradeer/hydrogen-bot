//! Time parsers used to parse different time syntaxes.

use std::sync::LazyLock;

use regex::Regex;

/// Regex for the `00s`, `00m`, and `00h` syntaxes.
static _TIME_SUFFIX_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(([0-9]{1,3})[sS]?|([0-9]{1,3})[mM]|([0-9]{1,3})[hH])$")
        .expect("failed to compile the regex for time suffix parser")
});

/// Parses the `00s`, `00m`, and `00h` syntaxes.
pub fn _suffix_syntax(data: &str) -> Option<u32> {
    let captures = _TIME_SUFFIX_REGEX.captures(data)?;

    if let Some(seconds) = captures.get(2) {
        // `00s` syntax.
        let seconds = seconds.as_str().parse::<u32>().ok()?;

        Some(seconds * 1000)
    } else if let Some(minutes) = captures.get(3) {
        // `00m` syntax.
        let minutes = minutes.as_str().parse::<u32>().ok()?;

        Some(minutes * 60 * 1000)
    } else if let Some(hours) = captures.get(4) {
        // `00h` syntax.
        let hours = hours.as_str().parse::<u32>().ok()?;

        Some(hours * 60 * 60 * 1000)
    } else {
        None
    }
}

/// Regex for the `00:00:00` and `00:00` syntaxes.
static _TIME_SEMICOLON_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^((([0-9]{1,3}):([0-5][0-9])|([0-9]{1,3})):([0-5][0-9]))$")
        .expect("failed to compile the regex for time semicolon parser")
});

/// Parses the `00:00:00` and `00:00` syntaxes.
pub fn _semicolon_syntax(data: &str) -> Option<u32> {
    let captures = _TIME_SEMICOLON_REGEX.captures(data)?;

    let hours_minutes = match captures.get(3) {
        Some(x) => {
            // `00:00:00` syntax.
            let hours = x.as_str().parse::<u32>().ok()?;
            let minutes = captures.get(4)?.as_str().parse::<u32>().ok()?;

            (hours * 60 * 60 * 1000) + (minutes * 60 * 1000)
        }
        None => {
            // `00:00` syntax.
            let minutes = captures.get(5)?.as_str().parse::<u32>().ok()?;

            minutes * 60 * 1000
        }
    };

    let seconds = captures.get(6)?.as_str().parse::<u32>().ok()?;

    Some(hours_minutes + (seconds * 1000))
}

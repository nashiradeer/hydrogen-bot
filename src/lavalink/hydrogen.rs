//! Helpers used by Hydrogen to interact with Hydrolink.

use regex::Regex;

use super::Rest;

/// Hydrogen's Lavalink configuration parser.
pub struct ConfigParser {
    /// The regex engine to parse the configuration.
    single_string_regex: Regex,
}

impl ConfigParser {
    /// Creates a new instance of the parser.
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Self {
            single_string_regex: Regex::new(
                r"((?:\[.+\]|[^;:\n]+):[0-9]{1,5})@([^/;\n]+)(?:/([^;\n]+))?;?",
            )?,
        })
    }

    /// Parses the configuration string into a list of [`Rest`] instances.
    pub fn parse(&self, value: String) -> Vec<Rest> {
        self.single_string_regex
            .captures_iter(&value)
            .filter_map(|cap| {
                let host = cap.get(1)?;
                let password = cap.get(2)?;

                if let Some(query) = cap.get(3) {
                    Rest::new(host.as_str(), password.as_str(), query.as_str() == "tls").ok()
                } else {
                    Rest::new(host.as_str(), password.as_str(), false).ok()
                }
            })
            .collect()
    }
}

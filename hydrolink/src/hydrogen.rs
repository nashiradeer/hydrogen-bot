//! Helpers used by Hydrogen to interact with Hydrolink.

use regex::Regex;

use super::Rest;

/// Hydrogen's Lavalink configuration parser.
pub struct ConfigParser<'a> {
    /// The regex engine to parse the configuration.
    single_string_regex: Regex,

    /// User agent.
    user_agent: &'a str,
}

impl<'a> ConfigParser<'a> {
    /// Creates a new instance of the parser.
    pub fn new(user_agent: &'a str) -> Result<Self, regex::Error> {
        Ok(Self {
            single_string_regex: Regex::new(
                r"((?:\[.+]|[^;:\n]+):[0-9]{1,5})@([^/;\n]+)(?:/([^;\n]+))?;?",
            )?,
            user_agent,
        })
    }

    /// Parses the configuration string into a list of [`Rest`] instances.
    pub fn parse(&self, value: &str) -> Vec<Rest> {
        self.single_string_regex
            .captures_iter(value)
            .filter_map(|cap| {
                let host = cap.get(1)?;
                let password = cap.get(2)?;

                if let Some(query) = cap.get(3) {
                    Rest::new(
                        host.as_str(),
                        password.as_str(),
                        self.user_agent,
                        query.as_str() == "tls",
                    )
                    .ok()
                } else {
                    Rest::new(host.as_str(), password.as_str(), self.user_agent, false).ok()
                }
            })
            .collect()
    }
}

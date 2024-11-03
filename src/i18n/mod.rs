//! Internationalization module.

use phf::Map;
use serenity::all::{CreateCommand, CreateCommandOption};

mod en_us;
mod pt_br;

pub static AVAILABLE_LANGS: &[(&str, &Map<&str, &str>); 2] = &[
    ("en_US", &en_us::TRANSLATIONS),
    ("pt_BR", &pt_br::TRANSLATIONS),
];

/// Translate a key to a specific language.
pub fn t<'a>(lang: &str, key: &'a str) -> &'a str {
    let lang_content = match lang {
        "pt_BR" => &pt_br::TRANSLATIONS,
        _ => &en_us::TRANSLATIONS,
    };

    lang_content
        .get(key)
        .or(en_us::TRANSLATIONS.get(key))
        .unwrap_or(&key)
}

/// Translate a key to a specific language with variables.
pub fn t_vars<'a, S: AsRef<str>, T: IntoIterator<Item = (&'a str, S)>>(
    lang: &str,
    key: &str,
    vars: T,
) -> String {
    let mut content = t(lang, key).to_owned();

    for (k, v) in vars.into_iter() {
        content = content.replace(&format!("{{{}}}", k), v.as_ref());
    }

    content
}

/// Translate a key to all available languages.
pub fn t_all(key: &str) -> Iter<'_> {
    Iter { key, index: 0 }
}

/// An iterator over the available languages for a specific key.
pub struct Iter<'a> {
    /// The key to translate.
    key: &'a str,
    /// The current index.
    index: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= AVAILABLE_LANGS.len() {
            return None;
        }

        let lang = AVAILABLE_LANGS[self.index];
        self.index += 1;

        Some((lang.0, lang.1.get(self.key)?))
    }
}

/// Inserts all the translations of a key into a [CreateCommand] as localized names.
pub fn serenity_command_name(key: &str, mut command: CreateCommand) -> CreateCommand {
    for (locale, name) in t_all(key) {
        command = command.name_localized(locale, name);
    }

    command
}

/// Inserts all the translations of a key into a [CreateCommand] as localized descriptions.
pub fn serenity_command_description(key: &str, mut command: CreateCommand) -> CreateCommand {
    for (locale, description) in t_all(key) {
        command = command.description_localized(locale, description);
    }

    command
}

/// Inserts all the translations of a key into a [CreateCommandOption] as localized names.
pub fn serenity_command_option_name(
    key: &str,
    mut option: CreateCommandOption,
) -> CreateCommandOption {
    for (locale, name) in t_all(key) {
        option = option.name_localized(locale, name);
    }

    option
}

/// Inserts all the translations of a key into a [CreateCommandOption] as localized descriptions.
pub fn serenity_command_option_description(
    key: &str,
    mut option: CreateCommandOption,
) -> CreateCommandOption {
    for (locale, description) in t_all(key) {
        option = option.description_localized(locale, description);
    }

    option
}

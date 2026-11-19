//! Language resolution, translated text and locale-aware number formatting.
//!
//! Text comes from `rust-i18n` catalogs embedded at compile time. Numbers and
//! plurals come from ICU4X, so the decimal separator, grouping separator and
//! plural category follow CLDR instead of being hard-coded per language.

use std::cell::RefCell;
use std::collections::HashMap;

use icu_decimal::DecimalFormatter;
use icu_decimal::input::Decimal as IcuDecimal;
use icu_locale_core::Locale;
use icu_plurals::{PluralCategory, PluralRuleType, PluralRules};
use rust_i18n::t;

use crate::config::Config;

/// Languages shipped with the application. The first tag is the fallback.
pub const SUPPORTED_LOCALES: &[(&str, &str)] = &[
    ("en", "English"),
    ("es", "Español"),
    ("fr", "Français"),
    ("pt", "Português"),
    ("it", "Italiano"),
    ("de", "Deutsch"),
    ("ca", "Català"),
    ("gl", "Galego"),
    ("eu", "Euskara"),
];

/// Normalizes a BCP 47 tag such as `es-ES` to one of the supported catalogs.
pub fn normalize_tag(tag: &str) -> Option<&'static str> {
    let language = tag.split(['-', '_']).next()?.to_ascii_lowercase();
    SUPPORTED_LOCALES
        .iter()
        .find_map(|(code, _)| (*code == language.as_str()).then_some(*code))
}

/// Chooses the locale to use, in priority order: explicit configuration,
/// system language, English fallback.
pub fn resolve_locale(config: Option<&str>, system: Option<&str>) -> &'static str {
    config
        .and_then(normalize_tag)
        .or_else(|| system.and_then(normalize_tag))
        .unwrap_or("en")
}

/// Resolves the locale and makes it the global `rust-i18n` locale.
///
/// `STRING_ART_LANG` overrides the configuration on native builds, which is
/// useful for tests and screenshots without touching the user's system.
pub fn init(config: &Config) -> &'static str {
    #[cfg(not(target_family = "wasm"))]
    let override_tag = std::env::var("STRING_ART_LANG").ok();
    #[cfg(target_family = "wasm")]
    let override_tag: Option<String> = None;

    let system = sys_locale::get_locale();
    // An unsupported override must not hide a valid configuration value.
    let configured = override_tag
        .as_deref()
        .filter(|tag| normalize_tag(tag).is_some())
        .or(config.language.as_deref());
    let locale = resolve_locale(configured, system.as_deref());
    rust_i18n::set_locale(locale);
    locale
}

/// The locale currently shown by the UI.
pub fn locale() -> String {
    rust_i18n::locale().to_string()
}

/// Changes the language. The caller is responsible for refreshing the windows
/// and for rebuilding widgets whose labels were resolved at construction time.
#[allow(dead_code)]
pub fn set_locale(tag: &str) {
    rust_i18n::set_locale(tag);
}

thread_local! {
    static DECIMAL_FORMATTERS: RefCell<HashMap<String, DecimalFormatter>> =
        RefCell::new(HashMap::new());
    static DECIMAL_SEPARATORS: RefCell<HashMap<String, char>> = RefCell::new(HashMap::new());
}

fn formatter_for(tag: &str) -> Option<DecimalFormatter> {
    DECIMAL_FORMATTERS.with(|formatters| {
        let mut formatters = formatters.borrow_mut();
        if let Some(formatter) = formatters.get(tag) {
            return Some(formatter.clone());
        }
        let locale: Locale = tag.parse().ok()?;
        let formatter = DecimalFormatter::try_new(locale.into(), Default::default()).ok()?;
        formatters.insert(tag.to_owned(), formatter.clone());
        Some(formatter)
    })
}

/// Decimal separator used by an editable number, without grouping.
pub(crate) fn decimal_separator_for(tag: &str) -> char {
    DECIMAL_SEPARATORS.with(|separators| {
        let mut separators = separators.borrow_mut();
        if let Some(separator) = separators.get(tag) {
            return *separator;
        }

        let separator = formatter_for(tag)
            .and_then(|formatter| {
                let mut sample = IcuDecimal::from(15);
                sample.multiply_pow10(-1);
                formatter
                    .format(&sample)
                    .to_string()
                    .chars()
                    .find(|character| !character.is_numeric() && *character != '-')
            })
            .unwrap_or_else(|| if tag.starts_with("es") { ',' } else { '.' });
        separators.insert(tag.to_owned(), separator);
        separator
    })
}

/// Formats a number for a read-only label, with locale grouping separators.
pub fn format_display_number(value: f64) -> String {
    format_display_number_for(&locale(), value)
}

fn format_display_number_for(tag: &str, value: f64) -> String {
    let rounded = value.round() as i64;
    formatter_for(tag)
        .map(|formatter| formatter.format(&IcuDecimal::from(rounded)).to_string())
        .unwrap_or_else(|| rounded.to_string())
}

/// Formats a number for an editable field: locale decimal separator, no
/// grouping, exactly `decimals` fractional digits.
pub fn format_editable_number(value: f64, decimals: u8) -> String {
    format_editable_number_for(&locale(), value, decimals)
}

fn format_editable_number_for(tag: &str, value: f64, decimals: u8) -> String {
    let factor = 10f64.powi(i32::from(decimals));
    let scaled = (value * factor).round() as i64;
    if decimals == 0 {
        return scaled.to_string();
    }

    let negative = scaled < 0;
    let digits = scaled.unsigned_abs().to_string();
    let decimals = usize::from(decimals);
    let padded = if digits.len() <= decimals {
        format!("{digits:0>width$}", width = decimals + 1)
    } else {
        digits
    };
    let (integer, fraction) = padded.split_at(padded.len() - decimals);
    let separator = decimal_separator_for(tag);
    format!(
        "{}{integer}{separator}{fraction}",
        if negative { "-" } else { "" }
    )
}

/// Parses a number typed by the user. Both `.` and `,` are accepted as the
/// decimal separator; earlier separators are treated as grouping.
pub fn parse_number(text: &str) -> Option<f64> {
    let mut digits = String::new();
    let mut separators: Vec<(usize, char)> = Vec::new();

    for character in text.chars() {
        if character.is_ascii_digit() {
            digits.push(character);
        } else if character == '.' || character == ',' {
            separators.push((digits.len(), character));
        } else if character.is_whitespace() || character == '\u{00a0}' || character == '\'' {
            continue;
        } else {
            return None;
        }
    }

    if digits.is_empty() {
        return None;
    }

    // One separator is the decimal separator. Several separators are only
    // accepted when they mix `.` and `,`, so a pasted `1,024.5` works while
    // a malformed `1,2,3` does not silently become a different number.
    if separators.len() > 1
        && separators
            .iter()
            .all(|(_, separator)| *separator == separators[0].1)
    {
        return None;
    }
    let decimal_at = separators.last().map(|(position, _)| *position);
    let mut normalized = String::with_capacity(digits.len() + 1);
    for (index, digit) in digits.chars().enumerate() {
        if Some(index) == decimal_at {
            normalized.push('.');
        }
        normalized.push(digit);
    }
    if decimal_at == Some(digits.len()) {
        normalized.push('.');
    }
    normalized.parse().ok()
}

fn plural_category_for(tag: &str, count: i64) -> PluralCategory {
    let fallback = if count == 1 {
        PluralCategory::One
    } else {
        PluralCategory::Other
    };
    let Ok(locale) = tag.parse::<Locale>() else {
        return fallback;
    };
    match PluralRules::try_new(locale.into(), PluralRuleType::Cardinal.into()) {
        Ok(rules) => rules.category_for(&IcuDecimal::from(count)),
        Err(_) => fallback,
    }
}

fn plural_suffix(category: PluralCategory) -> &'static str {
    match category {
        PluralCategory::Zero => "zero",
        PluralCategory::One => "one",
        PluralCategory::Two => "two",
        PluralCategory::Few => "few",
        PluralCategory::Many => "many",
        PluralCategory::Other => "other",
    }
}

/// Translates a count-dependent message stored as `<base>.<category>`.
pub fn tr_plural(base: &str, count: usize) -> String {
    tr_plural_for(&locale(), base, count)
}

fn tr_plural_for(tag: &str, base: &str, count: usize) -> String {
    let formatted = format_display_number_for(tag, count as f64);
    let key = format!(
        "{base}.{}",
        plural_suffix(plural_category_for(tag, count as i64))
    );
    let translated = t!(key.as_str(), locale = tag, count = formatted.as_str());
    if translated.as_ref() == key.as_str() {
        let fallback = format!("{base}.other");
        t!(fallback.as_str(), locale = tag, count = formatted.as_str()).to_string()
    } else {
        translated.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;

    fn load_catalog(yaml: &str) -> BTreeMap<String, String> {
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).expect("catalog should parse");
        let mut messages = BTreeMap::new();
        flatten("", &value, &mut messages);
        messages
    }

    fn flatten(prefix: &str, value: &serde_yaml::Value, messages: &mut BTreeMap<String, String>) {
        let serde_yaml::Value::Mapping(mapping) = value else {
            return;
        };
        for (key, value) in mapping {
            let serde_yaml::Value::String(key) = key else {
                continue;
            };
            let full = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{prefix}.{key}")
            };
            match value {
                serde_yaml::Value::Mapping(_) => flatten(&full, value, messages),
                serde_yaml::Value::String(message) if !key.starts_with('_') => {
                    messages.insert(full, message.clone());
                }
                _ => {}
            }
        }
    }

    fn placeholders(message: &str) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        let mut rest = message;
        while let Some(start) = rest.find("%{") {
            rest = &rest[start + 2..];
            let Some(end) = rest.find('}') else {
                break;
            };
            result.insert(rest[..end].to_owned());
            rest = &rest[end + 1..];
        }
        result
    }

    #[test]
    fn catalogues_have_the_same_keys_and_placeholders() {
        let en = load_catalog(include_str!("../locales/en.yml"));
        let catalogues = [
            ("es", include_str!("../locales/es.yml")),
            ("fr", include_str!("../locales/fr.yml")),
            ("pt", include_str!("../locales/pt.yml")),
            ("it", include_str!("../locales/it.yml")),
            ("de", include_str!("../locales/de.yml")),
            ("ca", include_str!("../locales/ca.yml")),
            ("gl", include_str!("../locales/gl.yml")),
            ("eu", include_str!("../locales/eu.yml")),
        ];

        for (tag, yaml) in catalogues {
            let catalogue = load_catalog(yaml);
            assert_eq!(
                en.keys().collect::<Vec<_>>(),
                catalogue.keys().collect::<Vec<_>>(),
                "{tag} must define the same keys as English"
            );
            for (key, english) in &en {
                assert_eq!(
                    placeholders(english),
                    placeholders(&catalogue[key]),
                    "placeholders differ for {key} in {tag}"
                );
            }
        }
    }

    #[test]
    fn catalogues_are_embedded_and_pluralized() {
        assert_eq!(
            rust_i18n::t!("settings.title", locale = "es"),
            "Configuración"
        );
        assert_eq!(
            rust_i18n::t!("import.choose", locale = "en"),
            "Choose an image"
        );
        for (locale, close) in [
            ("fr", "Fermer"),
            ("pt", "Fechar"),
            ("it", "Chiudi"),
            ("de", "Schließen"),
            ("ca", "Tanca"),
            ("gl", "Pechar"),
            ("eu", "Itxi"),
        ] {
            assert_eq!(rust_i18n::t!("common.close", locale = locale), close);
        }

        assert_eq!(
            tr_plural_for("en", "sidebar.computations", 1),
            "1 computation"
        );
        assert_eq!(
            tr_plural_for("en", "sidebar.computations", 2),
            "2 computations"
        );
        assert_eq!(
            tr_plural_for("es", "sidebar.computations", 1),
            "1 computación"
        );
        assert_eq!(
            tr_plural_for("es", "sidebar.computations", 2),
            "2 computaciones"
        );
        // French treats zero as singular.
        assert_eq!(tr_plural_for("fr", "sidebar.computations", 0), "0 calcul");
        assert_eq!(tr_plural_for("fr", "sidebar.computations", 2), "2 calculs");
        assert_eq!(tr_plural_for("pt", "sidebar.computations", 2), "2 cálculos");
        assert_eq!(
            tr_plural_for("de", "sidebar.computations", 1),
            "1 Berechnung"
        );
        assert_eq!(tr_plural_for("eu", "sidebar.computations", 2), "2 kalkulu");
    }

    #[test]
    fn resolves_configuration_before_the_system() {
        assert_eq!(resolve_locale(Some("es"), Some("en-US")), "es");
        assert_eq!(resolve_locale(Some("es-ES"), Some("en-US")), "es");
        assert_eq!(resolve_locale(None, Some("es-MX")), "es");
        assert_eq!(resolve_locale(None, Some("fr-FR")), "fr");
        assert_eq!(resolve_locale(None, Some("pt-BR")), "pt");
        assert_eq!(resolve_locale(Some("ca-ES"), Some("es")), "ca");
        assert_eq!(resolve_locale(None, Some("de-DE")), "de");
        assert_eq!(resolve_locale(None, Some("ja-JP")), "en");
        assert_eq!(resolve_locale(Some("xx"), Some("es")), "es");
        assert_eq!(resolve_locale(None, None), "en");
    }

    #[test]
    fn normalizes_regional_tags() {
        assert_eq!(normalize_tag("fr-CA"), Some("fr"));
        assert_eq!(normalize_tag("pt_BR"), Some("pt"));
        assert_eq!(normalize_tag("it-IT"), Some("it"));
        assert_eq!(normalize_tag("de-AT"), Some("de"));
        assert_eq!(normalize_tag("ca-ES"), Some("ca"));
        assert_eq!(normalize_tag("gl-ES"), Some("gl"));
        assert_eq!(normalize_tag("eu-ES"), Some("eu"));
        assert_eq!(normalize_tag("ja-JP"), None);
    }

    #[test]
    fn formats_editable_numbers_without_grouping() {
        assert_eq!(format_editable_number_for("en", 1.5, 1), "1.5");
        assert_eq!(format_editable_number_for("es", 1.5, 1), "1,5");
        assert_eq!(format_editable_number_for("en", 4096.0, 0), "4096");
        assert_eq!(format_editable_number_for("es", 4096.0, 0), "4096");
        assert_eq!(format_editable_number_for("es", 0.05, 1), "0,1");
    }

    #[test]
    fn formats_display_numbers_with_grouping() {
        assert_eq!(format_display_number_for("en", 10_240.0), "10,240");
        assert_eq!(format_display_number_for("es", 10_240.0), "10.240");
        // Spanish CLDR does not group four-digit integers.
        assert_eq!(format_display_number_for("es", 1024.0), "1024");
    }

    #[test]
    fn parses_both_decimal_separators() {
        assert_eq!(parse_number("1.5"), Some(1.5));
        assert_eq!(parse_number("1,5"), Some(1.5));
        assert_eq!(parse_number(" 1 024,5 "), Some(1024.5));
        assert_eq!(parse_number("1,024.5"), Some(1024.5));
        assert_eq!(parse_number("12"), Some(12.0));
        assert_eq!(parse_number("1,2,3"), None);
        assert_eq!(parse_number("abc"), None);
        assert_eq!(parse_number(""), None);
    }

    #[test]
    fn plural_categories_follow_the_locale() {
        assert_eq!(plural_category_for("en", 1), PluralCategory::One);
        assert_eq!(plural_category_for("en", 2), PluralCategory::Other);
        assert_eq!(plural_category_for("es", 1), PluralCategory::One);
        assert_eq!(plural_category_for("es", 0), PluralCategory::Other);
    }
}

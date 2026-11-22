//! Pure helpers with no Zed host calls, so they can be unit-tested natively.

use zed_extension_api::serde_json::Value;

/// Options read from `context_servers.markdown-pdf.settings` in Zed's settings.
///
/// Every export option (look, fonts, header, footer, ...) lives in the shared
/// markdown-pdf-export settings file, not here: these only tell the server
/// where that file is and which browser to use.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ServerOptions {
    pub settings_file: Option<String>,
    pub browser: Option<String>,
    pub debug: bool,
}

impl ServerOptions {
    /// Read options from the `settings` object. Unknown keys, wrong types and
    /// blank strings are ignored, so a half-filled template never breaks startup.
    pub fn from_settings(settings: Option<&Value>) -> Self {
        let text = |key: &str| {
            settings
                .and_then(|value| value.get(key))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        };
        Self {
            settings_file: text("settings_file"),
            browser: text("browser"),
            debug: settings
                .and_then(|value| value.get("debug"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        }
    }

    /// Environment variables understood by the markdown-pdf-export server.
    pub fn env(&self) -> Vec<(String, String)> {
        let mut env = Vec::new();
        if let Some(file) = &self.settings_file {
            env.push(("MARKDOWN_PDF_SETTINGS".to_owned(), file.clone()));
        }
        if let Some(browser) = &self.browser {
            env.push(("MARKDOWN_PDF_BROWSER".to_owned(), browser.clone()));
        }
        if self.debug {
            env.push(("MARKDOWN_PDF_DEBUG".to_owned(), "1".to_owned()));
        }
        env
    }
}

/// `true` when `version` is at least `minimum` (both `major.minor.patch`).
///
/// A pre-release suffix (`-beta.1`) is ignored. A version that cannot be read
/// is accepted: refusing to start over an odd version string helps nobody.
pub fn version_at_least(version: &str, minimum: &str) -> bool {
    match (parse(version), parse(minimum)) {
        (Some(version), Some(minimum)) => version >= minimum,
        _ => true,
    }
}

fn parse(version: &str) -> Option<(u64, u64, u64)> {
    let core = version.trim().trim_start_matches('v');
    let core = core.split(['-', '+']).next()?;
    let mut parts = core.split('.').map(|part| part.parse::<u64>().ok());
    Some((parts.next()??, parts.next()??, parts.next()??))
}

#[cfg(test)]
mod tests {
    use super::*;
    use zed_extension_api::serde_json::json;

    #[test]
    fn no_settings_means_no_env() {
        let options = ServerOptions::from_settings(None);
        assert_eq!(options, ServerOptions::default());
        assert!(options.env().is_empty());
    }

    #[test]
    fn maps_settings_to_env() {
        let settings = json!({
            "settings_file": " /home/me/pdf.json ",
            "browser": "msedge",
            "debug": true,
        });
        let env = ServerOptions::from_settings(Some(&settings)).env();
        assert_eq!(
            env,
            vec![
                (
                    "MARKDOWN_PDF_SETTINGS".to_owned(),
                    "/home/me/pdf.json".to_owned()
                ),
                ("MARKDOWN_PDF_BROWSER".to_owned(), "msedge".to_owned()),
                ("MARKDOWN_PDF_DEBUG".to_owned(), "1".to_owned()),
            ]
        );
    }

    #[test]
    fn ignores_blank_and_mistyped_values() {
        let settings = json!({
            "settings_file": "   ",
            "browser": 42,
            "debug": "yes",
            "page_format": "A5",
        });
        let options = ServerOptions::from_settings(Some(&settings));
        assert_eq!(options, ServerOptions::default());
    }

    #[test]
    fn compares_versions() {
        assert!(version_at_least("2.2.0", "2.2.0"));
        assert!(version_at_least("2.10.0", "2.2.0"));
        assert!(version_at_least("3.0.0-beta.1", "2.2.0"));
        assert!(version_at_least("v2.2.1", "2.2.0"));
        assert!(!version_at_least("2.1.1", "2.2.0"));
        assert!(!version_at_least("1.99.99", "2.2.0"));
    }

    #[test]
    fn accepts_unreadable_versions() {
        assert!(version_at_least("latest", "2.2.0"));
        assert!(version_at_least("2.2", "2.2.0"));
    }
}

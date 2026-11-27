//! Zed extension for markdown-pdf-export.
//!
//! Registers the `markdown-pdf` context server (MCP). The server itself is the
//! published npm package `markdown-pdf-export`, installed and kept up to date
//! by Zed and run with Zed's own Node, so users install nothing by hand.

mod config;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use zed::settings::ContextServerSettings;
use zed_extension_api::{
    self as zed, ContextServerConfiguration, ContextServerId, Project, Result,
};

use crate::config::{ServerOptions, major_of, update_check_due, version_at_least};

/// Context server id. Same key as the manual `context_servers.markdown-pdf`
/// entry the package's `setup` command writes, so a hand-made entry with a
/// `command` replaces this one instead of running twice.
const SERVER_ID: &str = "markdown-pdf";
const PACKAGE: &str = "markdown-pdf-export";
const ENTRY_POINT: &str = "node_modules/markdown-pdf-export/cli.mjs";
/// Oldest exporter the server starts with: 2.2.0 brought the setup tools
/// (`install_browser_markdown_pdf`, `open_studio_markdown_pdf`) and settings
/// reload the extension relies on.
const MIN_VERSION: &str = "2.2.0";
/// Below this version an update is tried at once, without waiting for the
/// daily check; if it fails, the older copy (at least MIN_VERSION) still
/// runs. 2.10.2 has the MCP tools ask for an absolute Markdown path, since
/// Zed may start the server outside the project folder; 2.9.1 fixed maths
/// printing in Times instead of KaTeX's fonts. Earlier: 2.8.0 took a default
/// `author` from the settings file (and `author: none` in a document); 2.6.0
/// fixed opening the PDF on Windows when a file name holds `&`; 2.5.0 drew
/// Mermaid diagrams and rendered task lists, footnotes and GitHub alerts; 2.4.0
/// stopped raw HTML from running scripts while the PDF is printed; 2.3.1
/// expanded `~` and `$HOME` in `settings_file`, which Zed gives the extension
/// no HOME to do.
const WANTED_VERSION: &str = "2.10.2";
/// Only this major version of the package is installed automatically: a new
/// major may change the MCP surface this extension was written against.
const PINNED_MAJOR: u64 = 2;
/// File in the extension work dir holding the unix time of the last npm
/// registry check, so normal starts skip the network entirely.
const UPDATE_CHECK_FILE: &str = ".last-update-check";

#[derive(Default)]
struct MarkdownPdfExtension {
    /// Entry point found on an earlier start; skips the npm check on restarts.
    cli_path: Option<PathBuf>,
}

impl MarkdownPdfExtension {
    fn cli_path(&mut self) -> Result<PathBuf> {
        if let Some(path) = &self.cli_path
            && is_file(path)
        {
            return Ok(path.clone());
        }

        let work_dir = env::current_dir()
            .map_err(|error| format!("cannot find the extension folder: {error}"))?;
        let entry = work_dir.join(ENTRY_POINT);
        let installed = zed::npm_package_installed_version(PACKAGE)?;
        let at_least = |minimum| {
            installed
                .as_deref()
                .is_some_and(|version| version_at_least(version, minimum))
                && is_file(&entry)
        };
        let has_usable_copy = at_least(MIN_VERSION);

        // The extension API has no async or timers, so an update check blocks
        // the server start. With an up-to-date copy it runs at most once a day;
        // every other start returns here without touching the network.
        let stamp = work_dir.join(UPDATE_CHECK_FILE);
        if at_least(WANTED_VERSION) && !update_check_due(now_secs(), read_unix_secs(&stamp)) {
            self.cli_path = Some(entry.clone());
            return Ok(entry);
        }

        check_for_update(&entry, installed.as_deref(), has_usable_copy)?;
        // Stamped even after an offline or failed check, so a machine without
        // npm access is not slowed down again on every start.
        if let Err(error) = fs::write(&stamp, now_secs().to_string()) {
            eprintln!("[markdown-pdf] could not record the update check time: {error}");
        }

        // Re-read: the check may have installed, replaced or removed the copy.
        let version = zed::npm_package_installed_version(PACKAGE)?
            .ok_or_else(|| format!("{PACKAGE} is not installed"))?;
        if !version_at_least(&version, MIN_VERSION) {
            return Err(format!(
                "{PACKAGE} {version} is installed but this extension needs {MIN_VERSION} or \
                 later. Check the internet connection, then restart the server to update it."
            ));
        }
        if !is_file(&entry) {
            return Err(format!(
                "{PACKAGE} {version} is installed but {} is missing",
                entry.display()
            ));
        }

        self.cli_path = Some(entry.clone());
        Ok(entry)
    }
}

/// Ask npm for the newest version and install it when it is an update within
/// [`PINNED_MAJOR`]. Never fails while a working copy exists — problems go to
/// stderr instead, which Zed shows only when started with `zed --foreground`
/// (not in Zed.log); with no copy at all the error is returned.
fn check_for_update(entry: &Path, installed: Option<&str>, has_usable_copy: bool) -> Result<()> {
    // Any copy at all, even one older than MIN_VERSION: a stale server beats
    // none. Re-checked after a failed install: npm may have removed the copy.
    let has_copy = || installed.is_some() && is_file(entry);

    let latest = match zed::npm_package_latest_version(PACKAGE) {
        Ok(latest) => latest,
        Err(error) => {
            if has_copy() {
                eprintln!("[markdown-pdf] could not reach npm to check for updates: {error}");
                return Ok(());
            }
            return Err(format!(
                "could not reach npm to install {PACKAGE}: {error}. \
                 Check the internet connection, then restart the server."
            ));
        }
    };

    let target = match major_of(&latest) {
        Some(major) if major == PINNED_MAJOR => latest,
        // Nothing usable installed and `latest` out of reach (newer major or
        // unreadable): ask npm for the newest release of the pinned major.
        _ if !has_usable_copy => PINNED_MAJOR.to_string(),
        _ => {
            eprintln!(
                "[markdown-pdf] not updating to {PACKAGE} {latest}: this extension only \
                 installs version {PINNED_MAJOR}.x. A newer extension may support it."
            );
            return Ok(());
        }
    };

    if installed == Some(target.as_str()) && is_file(entry) {
        return Ok(());
    }
    if let Err(error) = zed::npm_install_package(PACKAGE, &target) {
        if has_copy() {
            // A failed update keeps a working older copy rather than stop the server.
            eprintln!(
                "[markdown-pdf] could not update {PACKAGE} to {target}, keeping {}: {error}",
                installed.unwrap_or("the current copy")
            );
            return Ok(());
        }
        return Err(format!(
            "could not install {PACKAGE} {target} from npm: {error}"
        ));
    }
    Ok(())
}

/// Seconds since the unix epoch; 0 when the clock is unreadable, which makes
/// the next update check due.
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

fn read_unix_secs(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn is_file(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|meta| meta.is_file())
}

fn server_options(project: &Project) -> ServerOptions {
    let settings = ContextServerSettings::for_project(SERVER_ID, project)
        .ok()
        .and_then(|settings| settings.settings);
    // Passed as written: the server expands `~` and `$HOME` and warns about
    // a missing file (the wasm sandbox sees neither HOME nor that file).
    ServerOptions::from_settings(settings.as_ref())
}

impl zed::Extension for MarkdownPdfExtension {
    fn new() -> Self {
        Self::default()
    }

    fn context_server_command(
        &mut self,
        context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<zed::Command> {
        if context_server_id.as_ref() != SERVER_ID {
            return Err(format!("unknown context server: {context_server_id}"));
        }
        let cli = self.cli_path()?;
        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: vec![cli.to_string_lossy().into_owned(), "mcp".to_owned()],
            env: server_options(project).env(),
        })
    }

    fn context_server_configuration(
        &mut self,
        context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        if context_server_id.as_ref() != SERVER_ID {
            return Ok(None);
        }
        Ok(Some(ContextServerConfiguration {
            installation_instructions: include_str!(
                "../configuration/installation_instructions.md"
            )
            .to_owned(),
            settings_schema: include_str!("../configuration/settings_schema.json").to_owned(),
            default_settings: include_str!("../configuration/default_settings.json").to_owned(),
        }))
    }
}

zed::register_extension!(MarkdownPdfExtension);

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

use crate::config::{ServerOptions, version_at_least};

/// Context server id. Same key as the manual `context_servers.markdown-pdf`
/// entry the package's `setup` command writes, so a hand-made entry with a
/// `command` replaces this one instead of running twice.
const SERVER_ID: &str = "markdown-pdf";
const PACKAGE: &str = "markdown-pdf-export";
const ENTRY_POINT: &str = "node_modules/markdown-pdf-export/cli.mjs";
/// First release with the setup tools (`install_browser_markdown_pdf`,
/// `open_studio_markdown_pdf`) and settings reload the extension relies on.
const MIN_VERSION: &str = "2.2.0";

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

        let entry = env::current_dir()
            .map_err(|error| format!("cannot find the extension folder: {error}"))?
            .join(ENTRY_POINT);
        let installed = zed::npm_package_installed_version(PACKAGE)?;

        match zed::npm_package_latest_version(PACKAGE) {
            Ok(latest) => {
                let outdated = installed.as_deref() != Some(latest.as_str()) || !is_file(&entry);
                if outdated
                    && let Err(error) = zed::npm_install_package(PACKAGE, &latest)
                    // Keep a working older copy rather than fail on a bad network.
                    && (installed.is_none() || !is_file(&entry))
                {
                    return Err(format!(
                        "could not install {PACKAGE} {latest} from npm: {error}"
                    ));
                }
            }
            Err(error) => {
                if installed.is_none() || !is_file(&entry) {
                    return Err(format!(
                        "could not reach npm to install {PACKAGE}: {error}. \
                         Check the internet connection, then restart the server."
                    ));
                }
            }
        }

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

fn is_file(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|meta| meta.is_file())
}

fn server_options(project: &Project) -> ServerOptions {
    let settings = ContextServerSettings::for_project(SERVER_ID, project)
        .ok()
        .and_then(|settings| settings.settings);
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

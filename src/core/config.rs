use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::cli::{MdvArgs, Theme};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppConfig {
    pub path: PathBuf,
    pub watch: bool,
    pub theme: Theme,
    pub mermaid_mode: MermaidMode,
    /// Multiplier of the backend's native raster density, independent of document zoom.
    pub snapshot_scale: u8,
}

impl TryFrom<MdvArgs> for AppConfig {
    type Error = anyhow::Error;

    fn try_from(value: MdvArgs) -> Result<Self> {
        let MdvArgs { command, path, watch, theme, no_mermaid, snapshot_scale, .. } = value;

        if command.is_some() {
            bail!("view configuration cannot be built from a subcommand")
        }

        let Some(path) = path else { bail!("mdv requires a Markdown file path") };

        let mermaid_mode = if no_mermaid { MermaidMode::Disabled } else { MermaidMode::Enabled };

        Ok(Self { path, watch, theme: theme.resolve(), mermaid_mode, snapshot_scale })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MermaidMode {
    Enabled,
    Disabled,
}

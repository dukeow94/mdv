//! Platform-selected HTML rasterization for the interactive graphics pipeline.

use std::path::Path;

use anyhow::Result;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use anyhow::bail;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) const NATIVE_IMAGE_SCRIPT: &str = include_str!("snapshot_images.js");

/// Renders an HTML document into the lossless raster consumed by `GraphicPage`.
/// `width` is the unchanged CSS viewport; `snapshot_scale` multiplies backend raster density.
pub fn render_html_to_raster(
    html: &str,
    base_dir: &Path,
    width: u32,
    snapshot_scale: u8,
) -> Result<Vec<u8>> {
    anyhow::ensure!((1..=3).contains(&snapshot_scale), "snapshot scale must be 1, 2, or 3");
    #[cfg(target_os = "macos")]
    {
        crate::io::webkit_snapshot::render_html_to_tiff(html, base_dir, width, snapshot_scale)
    }

    #[cfg(target_os = "windows")]
    {
        crate::io::chromium_snapshot::render_html_to_png(html, base_dir, width, snapshot_scale)
            .map(|result| result.png_bytes)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = (html, base_dir, width, snapshot_scale);
        bail!("interactive HTML rasterization is only supported on macOS and Windows");
    }
}

use std::path::Path;

use super::{
    SnapshotDiagnostics,
    paths::{cleanup_workspace, create_workspace, snapshot_asset_root},
};

#[cfg(target_os = "macos")]
#[test]
fn platform_raster_facade_preserves_lossless_graphics_input() {
    let workspace = tempfile::tempdir().unwrap_or_else(|error| panic!("workspace: {error}"));
    let raster = crate::io::snapshot::render_html_to_raster(
        "<body style='margin:0;background:#243b53'><h1>Raster</h1></body>",
        workspace.path(),
        320,
        3,
    )
    .unwrap_or_else(|error| panic!("platform raster facade should render: {error:#}"));

    let page = image::load_from_memory(&raster)
        .unwrap_or_else(|error| panic!("lossless raster should decode: {error}"));
    let native = super::render_html_to_png(
        "<body style='margin:0;background:#243b53'><h1>Raster</h1></body>",
        workspace.path(),
        320,
    )
    .unwrap_or_else(|error| panic!("native capture: {error}"));
    let native = image::load_from_memory(&native.png_bytes)
        .unwrap_or_else(|error| panic!("native raster: {error}"));
    assert_eq!(page.width(), native.width() * 3);
    assert_eq!(page.height(), native.height() * 3);
}

#[cfg(target_os = "macos")]
#[test]
fn scaled_snapshot_resolves_subpixel_vector_detail() {
    let workspace = tempfile::tempdir().unwrap_or_else(|error| panic!("workspace: {error}"));
    let html = "<body style='margin:0;background:white'>\
        <svg width='320' height='12' style='display:block'>\
        <path d='M 0.1666667 0 V 12' stroke='black' stroke-width='0.3333333'/></svg></body>";
    let capture = |scale| {
        let snapshot = super::render_html_to_png_with_scale(html, workspace.path(), 320, scale)
            .unwrap_or_else(|error| panic!("vector capture at {scale}x: {error}"));
        image::load_from_memory(&snapshot.png_bytes)
            .unwrap_or_else(|error| panic!("vector raster: {error}"))
            .to_rgba8()
    };
    let native = capture(1);
    let scaled = capture(3);
    assert_eq!(scaled.width(), native.width() * 3);
    // A one-third-CSS-pixel stroke resolves into a black pixel at high density,
    // not three copies of the native anti-aliased gray pixel.
    assert!(native.get_pixel(0, 1)[0] > 50, "native stroke should be subpixel gray");
    assert!(scaled.get_pixel(0, 3)[0] < 10, "browser must rasterize fresh vector detail");
    assert!(scaled.get_pixel(2, 3)[0] > 245, "adjacent pixels must remain white");
}

#[cfg(target_os = "macos")]
#[test]
fn img_density_stays_native_while_inline_vectors_gain_detail() {
    let workspace = tempfile::tempdir().unwrap_or_else(|error| panic!("workspace: {error}"));
    let stripes = image::RgbaImage::from_fn(960, 12, |x, _| {
        let value = if x % 3 == 0 { 0 } else { 255 };
        image::Rgba([value, value, value, 255])
    });
    stripes
        .save(workspace.path().join("stripes.png"))
        .unwrap_or_else(|error| panic!("striped source: {error}"));
    let base = url::Url::from_directory_path(workspace.path())
        .unwrap_or_else(|()| panic!("fixture directory should become a file URL"));
    let html = format!(
        "<base href='{base}'><body style='margin:0;background:white'>\
        <img src='stripes.png' width='320' height='4' style='display:block'>\
        <svg width='320' height='12' style='display:block'>\
        <path d='M 0.1666667 0 V 12' stroke='black' stroke-width='0.3333333'/></svg></body>"
    );
    let mut baseline = None;
    for scale in [1, 2, 3] {
        let snapshot = super::render_html_to_png_with_scale(&html, workspace.path(), 320, scale)
            .unwrap_or_else(|error| panic!("capture at {scale}x: {error}"));
        let image = image::load_from_memory(&snapshot.png_bytes)
            .unwrap_or_else(|error| panic!("raster: {error}"))
            .to_rgba8();
        let density = image.width() / 320;
        let image_pixel = image.get_pixel(10 * density, density)[0];
        assert!(
            (100..=230).contains(&image_pixel),
            "img must retain native-density blended pixels, not resolve source stripes at {scale}x: {image_pixel}"
        );
        let native = baseline.get_or_insert(snapshot.diagnostics.clone());
        assert_eq!(
            &snapshot.diagnostics, native,
            "original image diagnostics and layout must be preserved"
        );
        if scale == 3 {
            assert!(
                image.get_pixel(0, 5 * density)[0] < 10,
                "inline vectors must still receive high-density rasterization"
            );
        }
    }
}

#[cfg(target_os = "macos")]
#[test]
fn native_density_images_preserve_gif_svg_picture_and_object_fit_geometry() {
    let workspace = tempfile::tempdir().unwrap_or_else(|error| panic!("workspace: {error}"));
    image::RgbaImage::from_pixel(640, 64, image::Rgba([0, 80, 220, 180]))
        .save(workspace.path().join("source.gif"))
        .unwrap_or_else(|error| panic!("GIF fixture: {error}"));
    std::fs::write(workspace.path().join("source.svg"),
        "<svg xmlns='http://www.w3.org/2000/svg' width='240' height='60'><rect width='240' height='60' fill='red'/></svg>")
        .unwrap_or_else(|error| panic!("SVG fixture: {error}"));
    let base =
        url::Url::from_directory_path(workspace.path()).unwrap_or_else(|()| panic!("fixture URL"));
    let html = format!(
        "<base href='{base}'><style>body{{margin:0;font:16px/24px sans-serif}}\
        .frame{{text-align:center}}img{{vertical-align:middle}}\
        .box{{box-sizing:border-box;border:2px solid #000;padding:3px;margin:5px}}\
        </style><body><p>Before images</p><div class='frame'>\
        <picture><source media='(min-width:200px)' srcset='source.svg 1x, source.svg 2x'>\
        <img class='box' src='source.gif' width='120' height='60' style='object-fit:contain;object-position:right bottom'></picture></div>\
        <p>Inline <img src='source.gif' width='90' height='30'> tail</p>\
        <img class='box' src='source.gif' width='120' height='40' style='object-fit:cover;object-position:left top;display:block'>\
        <img class='box' src='source.svg' width='90' height='40' style='object-fit:scale-down;display:block'>\
        <img src='source.svg' width='90' height='40' style='object-fit:none;display:block'>\
        <p>After images</p></body>"
    );
    let mut baseline = None;
    for scale in [1, 2, 3] {
        let snapshot = super::render_html_to_png_with_scale(&html, workspace.path(), 320, scale)
            .unwrap_or_else(|error| panic!("image geometry at {scale}x: {error}"));
        let page = crate::ui::page_graphics::build_graphic_page(&snapshot.png_bytes, 80, 496)
            .unwrap_or_else(|error| panic!("page: {error}"));
        let reference =
            baseline.get_or_insert((snapshot.diagnostics.clone(), page.display_height_px));
        assert_eq!(
            snapshot.diagnostics, reference.0,
            "original sources and display dimensions must stay fixed"
        );
        assert_eq!(page.display_height_px, reference.1);
        assert_eq!(snapshot.diagnostics.images.len(), 5);
    }
}

#[cfg(target_os = "macos")]
#[test]
fn native_density_image_minification_does_not_introduce_checkerboard_moire() {
    let workspace = tempfile::tempdir().unwrap_or_else(|error| panic!("workspace: {error}"));
    image::RgbaImage::from_fn(1920, 1080, |x, y| {
        let value = if (x + y) % 2 == 0 { 0 } else { 255 };
        image::Rgba([value, value, value, 255])
    })
    .save(workspace.path().join("checker.png"))
    .unwrap_or_else(|error| panic!("checker fixture: {error}"));
    let base =
        url::Url::from_directory_path(workspace.path()).unwrap_or_else(|()| panic!("fixture URL"));
    let html = format!(
        "<base href='{base}'><body style='margin:0'><img src='checker.png' width='317' height='179' style='display:block'></body>"
    );
    let mut native_contrast = None;
    for scale in [1, 3] {
        let snapshot = super::render_html_to_png_with_scale(&html, workspace.path(), 320, scale)
            .unwrap_or_else(|error| panic!("checker capture: {error}"));
        let image = image::load_from_memory(&snapshot.png_bytes)
            .unwrap_or_else(|error| panic!("checker raster: {error}"))
            .to_rgba8();
        let density = image.width() / 320;
        let mut minimum = 255_u8;
        let mut maximum = 0_u8;
        for y in 5..174 {
            for x in 5..312 {
                let value =
                    image.get_pixel(x * density + density / 2, y * density + density / 2)[0];
                minimum = minimum.min(value);
                maximum = maximum.max(value);
            }
        }
        let contrast = u16::from(maximum) - u16::from(minimum);
        let native = *native_contrast.get_or_insert(contrast);
        assert!(
            contrast <= native + 10,
            "image preparation must not add minification moire at {scale}x: contrast={contrast}, native={native}"
        );
    }
}

#[test]
fn diagnostics_default_to_empty_assets() {
    let diagnostics = SnapshotDiagnostics::default();

    assert!(diagnostics.images.is_empty());
    assert!(diagnostics.mermaids.is_empty());
}

#[test]
fn snapshot_asset_root_expands_to_cover_relative_parent_assets() {
    let html = r#"<html><body><img src="../../../../examples/pixel.png" /></body></html>"#;
    let project = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("project fixture should be created: {error}"));
    let base_dir = project.path().join("tests/fixtures/gfm/image");
    std::fs::create_dir_all(&base_dir)
        .unwrap_or_else(|error| panic!("fixture base directory should be created: {error}"));

    let root = snapshot_asset_root(html, &base_dir);

    assert_eq!(root, project.path());
}

#[test]
fn snapshot_workspace_uses_temp_dir() {
    let temp_dir = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("temporary workspace root should be created: {error}"));
    let workspace = create_workspace(temp_dir.path(), temp_dir.path())
        .unwrap_or_else(|error| panic!("workspace should be created: {error}"));

    assert!(workspace.starts_with(temp_dir.path()));
    assert_eq!(
        workspace.parent().and_then(Path::file_name),
        Some(std::ffi::OsStr::new(".mdv-webkit"))
    );
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn cleanup_workspace_removes_empty_mdv_webkit_parent() {
    let base = std::env::temp_dir().join(format!(
        "mdv-webkit-cleanup-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&base)
        .unwrap_or_else(|error| panic!("base directory should be created: {error}"));
    let workspace = create_workspace(&base, &base)
        .unwrap_or_else(|error| panic!("workspace should be created: {error}"));
    let parent = workspace
        .parent()
        .unwrap_or_else(|| panic!("workspace should have a parent"))
        .to_path_buf();

    cleanup_workspace(&workspace)
        .unwrap_or_else(|error| panic!("workspace cleanup should succeed: {error}"));

    assert!(!parent.exists(), "empty .mdv-webkit parent should be removed");
    let _ = std::fs::remove_dir_all(base);
}

#[cfg(target_os = "macos")]
#[test]
fn worker_renders_fresh_documents_recovers_and_scales_without_reflow() {
    use anyhow::{Context, Result, ensure};
    use std::{
        io::{BufRead, BufReader, Write},
        os::unix::{fs::PermissionsExt, net::UnixStream},
        process::{Command, Stdio},
        time::{Duration, Instant},
    };

    let dir = tempfile::Builder::new()
        .prefix("mdv-worker-test-")
        .tempdir_in("/tmp")
        .unwrap_or_else(|error| panic!("worker workspace: {error}"));
    let helper = dir.path().join("helper");
    std::fs::write(&helper, super::SNAPSHOT_HELPER)
        .unwrap_or_else(|error| panic!("write helper: {error}"));
    std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o700))
        .unwrap_or_else(|error| panic!("helper permissions: {error}"));
    let socket = dir.path().join("worker.sock");
    let mut child = Command::new(&helper)
        .arg("--serve")
        .arg(&socket)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| panic!("start worker: {error}"));

    let result = (|| -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if UnixStream::connect(&socket).is_ok() {
                break;
            }
            ensure!(
                child.try_wait()?.is_none(),
                "worker exited before accepting rendering requests"
            );
            ensure!(Instant::now() < deadline, "worker did not start");
            std::thread::sleep(Duration::from_millis(10));
        }
        let html = dir.path().join("document.html");
        let output = dir.path().join("snapshot.png");
        let report = dir.path().join("report.json");
        let request = serde_json::json!({
            "html_path": html, "read_access_root": dir.path(),
            "output_path": output, "report_path": report, "width": 320,
            "image_preparation_script": crate::io::snapshot::NATIVE_IMAGE_SCRIPT,
        });
        let render_scaled =
            |format: &str, timeout_ms: u32, scale: u8| -> Result<serde_json::Value> {
                let mut request = request.clone();
                request["output_format"] = format.into();
                request["timeout_ms"] = timeout_ms.into();
                request["snapshot_scale"] = scale.into();
                let mut stream = UnixStream::connect(&socket)?;
                stream.set_read_timeout(Some(Duration::from_secs(15)))?;
                serde_json::to_writer(&mut stream, &request)?;
                stream.write_all(b"\n")?;
                let mut line = String::new();
                BufReader::new(stream).read_line(&mut line)?;
                serde_json::from_str(&line).context("worker response must be JSON")
            };
        let render = |format: &str, timeout_ms: u32| render_scaled(format, timeout_ms, 1);
        for color in ["#ff0000", "#0000ff"] {
            std::fs::write(
                &html,
                format!("<body style='margin:0;background:{color}'><h1>Fresh</h1></body>"),
            )?;
            ensure!(render("png", 25_000)?["ok"] == true, "worker should render updated HTML");
            let actual = image::open(&output)?.to_rgba8();
            let reference = dir.path().join("reference.png");
            let status = Command::new(&helper)
                .args([
                    html.as_os_str(),
                    dir.path().as_os_str(),
                    reference.as_os_str(),
                    report.as_os_str(),
                    std::ffi::OsStr::new("320"),
                ])
                .status()?;
            ensure!(status.success(), "reference render should succeed");
            ensure!(
                actual == image::open(reference)?.to_rgba8(),
                "worker must preserve exact reference pixels after document changes"
            );
        }
        ensure!(
            render("tiff", 25_000)?["ok"] == true,
            "worker should return lossless raster transport"
        );
        let tiff = std::fs::read(&output)?;
        ensure!(
            image::guess_format(&tiff)? == image::ImageFormat::Tiff,
            "worker should avoid full-page PNG compression when TIFF is requested"
        );
        ensure!(
            image::load_from_memory(&tiff)?.to_rgba8()
                == image::open(dir.path().join("reference.png"))?.to_rgba8(),
            "lossless transport must preserve reference RGBA pixels"
        );
        std::fs::write(&html, "<img src='missing.png'>")?;
        let failure = render("png", 25_000)?;
        ensure!(
            failure["ok"] == false
                && failure["error"]
                    .as_str()
                    .is_some_and(|error| error.contains("failed to render")),
            "broken image diagnostics must survive worker mode: {failure}"
        );
        std::fs::write(&html, "<body><h1>Recovered</h1></body>")?;
        ensure!(
            render("png", 25_000)?["ok"] == true,
            "worker should recover after a failed document"
        );
        let timed_out = render("png", 1)?;
        ensure!(
            timed_out["ok"] == false
                && timed_out["error"].as_str().is_some_and(|error| error.contains("timed out")),
            "a render deadline must cancel active work: {timed_out}"
        );
        ensure!(
            render("png", 25_000)?["ok"] == true,
            "worker must render the next document after cancellation"
        );

        // Text wraps naturally in the fixed CSS viewport. Solid bands make crop
        // position and full-page orientation observable independently of glyph AA.
        std::fs::write(
            &html,
            format!(
                "<style>body{{margin:0;background:white}}p{{font:16px/24px monospace;margin:0}}.band{{height:48px}}</style>\
             <div class='band' style='background:#f00'></div><p><strong>{}</strong></p>\
             <div style='height:1200px'></div><div class='band' style='background:#00f'></div>",
                "wrapping text stays the same across capture densities. ".repeat(30)
            ),
        )?;
        let mut baseline = None;
        for scale in [1_u8, 2, 3, 1] {
            for format in ["png", "tiff"] {
                ensure!(
                    render_scaled(format, 25_000, scale)?["ok"] == true,
                    "scaled capture failed"
                );
                let bytes = std::fs::read(&output)?;
                let page = crate::ui::page_graphics::build_graphic_page(&bytes, 80, 496)?;
                let report: serde_json::Value = serde_json::from_slice(&std::fs::read(&report)?)?;
                let (reference, reference_report) =
                    baseline.get_or_insert_with(|| (page.clone(), report.clone()));
                ensure!(
                    page.image_width_px == reference.image_width_px * u32::from(scale),
                    "snapshot scale {scale} must increase actual browser raster width"
                );
                ensure!(
                    page.image_height_px == reference.image_height_px * u32::from(scale),
                    "snapshot scale {scale} must increase actual browser raster height"
                );
                ensure!(
                    page.display_width_px == reference.display_width_px
                        && page.display_height_px == reference.display_height_px,
                    "capture density must not change terminal display size"
                );
                ensure!(
                    report == *reference_report,
                    "CSS layout, wrapping height and typography must be unchanged"
                );
                let total = crate::ui::page_graphics::total_rows(&page, 16.0);
                for scroll in [0.0, 1.25, total as f64 / 2.0, (total - 2) as f64] {
                    let crop = crate::ui::page_graphics::viewport_slice_at(&page, scroll, 20, 16.0);
                    let original =
                        crate::ui::page_graphics::viewport_slice_at(reference, scroll, 20, 16.0);
                    ensure!(crop.rows == original.rows, "crop must reserve the same terminal rows");
                    ensure!(
                        crop.source_y_px.abs_diff(original.source_y_px * u32::from(scale))
                            <= u32::from(scale),
                        "crop origin must track raster density"
                    );
                    ensure!(
                        crop.source_height_px
                            .abs_diff(original.source_height_px * u32::from(scale))
                            <= u32::from(scale),
                        "crop height must track raster density"
                    );
                }
                ensure!(
                    page.image.get_pixel(0, 0).0 == [255, 0, 0, 255],
                    "top band must remain at the top"
                );
                ensure!(
                    page.image.get_pixel(0, page.image_height_px - 1).0 == [0, 0, 255, 255],
                    "tail band must remain at the bottom"
                );
            }
        }
        for invalid in [0, 4] {
            ensure!(
                render_scaled("png", 25_000, invalid)?["ok"] == false,
                "worker must reject invalid scale {invalid}"
            );
        }
        Ok(())
    })();
    let _ = child.kill();
    let _ = child.wait();
    result.unwrap_or_else(|error| panic!("persistent renderer contract failed: {error:#}"));
}

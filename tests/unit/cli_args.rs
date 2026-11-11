use mdv::cli::{MdvArgs, MdvCommand, Theme};

#[test]
fn parses_watch_theme_and_mermaid_flags() {
    let args =
        MdvArgs::parse_from(["mdv", "docs/PRD.md", "--watch", "--theme", "dark", "--no-mermaid"]);

    assert_eq!(args.theme, Theme::Dark);
    assert!(args.watch);
    assert!(args.no_mermaid);
    assert_eq!(args.path.as_deref(), Some(std::path::Path::new("docs/PRD.md")));
}

#[test]
fn parses_update_subcommand_without_document_path() {
    let args = MdvArgs::parse_from(["mdv", "update"]);

    assert_eq!(args.command, Some(MdvCommand::Update));
    assert_eq!(args.path, None);
}

#[test]
fn snapshot_scale_defaults_to_one_and_reaches_view_configuration() {
    use mdv::core::config::AppConfig;

    let args = MdvArgs::parse_from(["mdv", "README.md"]);
    assert_eq!(args.snapshot_scale, 1);
    for scale in ["1", "2", "3"] {
        let args = MdvArgs::parse_from([
            "mdv",
            "README.md",
            "--theme",
            "light",
            "--snapshot-scale",
            scale,
        ]);
        let config = AppConfig::try_from(args)
            .unwrap_or_else(|error| panic!("view config should be valid: {error}"));
        assert_eq!(config.snapshot_scale.to_string(), scale);
    }
}

#[test]
fn defaults_theme_to_system() {
    let args = MdvArgs::parse_from(["mdv", "docs/PRD.md"]);

    assert_eq!(args.theme, Theme::System);
}

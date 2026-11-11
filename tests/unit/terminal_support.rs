use mdv::ui::terminal::is_supported_terminal;

#[test]
fn accepts_ghostty_and_kitty() {
    assert!(is_supported_terminal(Some("ghostty"), None));
    assert!(is_supported_terminal(None, Some("xterm-kitty")));
}

#[test]
fn accepts_rex_without_spoofing_another_terminal() {
    for program in ["rex", "Rex", "REX"] {
        assert!(is_supported_terminal(Some(program), Some("xterm-256color")));
    }
    assert!(!is_supported_terminal(Some("rex-other"), Some("xterm-256color")));
}

#[test]
fn rejects_unknown_terminal() {
    assert!(!is_supported_terminal(Some("iterm"), Some("xterm-256color")));
}

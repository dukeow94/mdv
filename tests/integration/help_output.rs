use assert_cmd::Command;
use predicates::str::{contains, diff};

#[test]
fn help_mentions_core_flags() {
    let mut command = match Command::cargo_bin("mdv") {
        Ok(command) => command,
        Err(error) => panic!("binary should build: {error}"),
    };

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("--watch"))
        .stdout(contains("--theme"))
        .stdout(contains("--no-mermaid"))
        .stdout(contains("--snapshot-scale"))
        .stdout(contains("--version"))
        .stdout(contains("system"))
        .stdout(contains("update"));
}

#[test]
fn snapshot_scale_accepts_only_one_two_or_three() {
    let baseline = Command::cargo_bin("mdv")
        .unwrap_or_else(|error| panic!("binary should build: {error}"))
        .args(["README.md", "--no-mermaid"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    for scale in ["1", "2", "3"] {
        let mut command = Command::cargo_bin("mdv")
            .unwrap_or_else(|error| panic!("binary should build: {error}"));
        command
            .args(["README.md", "--no-mermaid", "--snapshot-scale", scale])
            .assert()
            .success()
            .stdout(baseline.clone());
    }
    for scale in ["0", "4", "-1", "1.5", "NaN", "256"] {
        let mut command = Command::cargo_bin("mdv")
            .unwrap_or_else(|error| panic!("binary should build: {error}"));
        command.args(["README.md", "--snapshot-scale", scale]).assert().code(2);
    }
    let mut command =
        Command::cargo_bin("mdv").unwrap_or_else(|error| panic!("binary should build: {error}"));
    command.args(["README.md", "--snapshot-scale"]).assert().code(2);
}

#[test]
fn update_help_mentions_main_binary_replacement() {
    let mut command = match Command::cargo_bin("mdv") {
        Ok(command) => command,
        Err(error) => panic!("binary should build: {error}"),
    };

    command
        .args(["update", "--help"])
        .assert()
        .success()
        .stdout(contains("GitHub main"))
        .stdout(contains("bin/mdv"));
}

#[test]
fn version_flag_prints_the_package_version() {
    let mut command = match Command::cargo_bin("mdv") {
        Ok(command) => command,
        Err(error) => panic!("binary should build: {error}"),
    };

    command
        .arg("--version")
        .assert()
        .success()
        .stdout(diff(format!("{}\n", env!("CARGO_PKG_VERSION"))));
}

#[test]
fn version_flag_stays_numeric_only_when_combined_with_a_path() {
    let mut command = match Command::cargo_bin("mdv") {
        Ok(command) => command,
        Err(error) => panic!("binary should build: {error}"),
    };

    command
        .args(["README.md", "--version"])
        .assert()
        .success()
        .stdout(diff(format!("{}\n", env!("CARGO_PKG_VERSION"))));
}

#[test]
fn version_flag_stays_numeric_only_when_combined_with_a_subcommand() {
    let mut command = match Command::cargo_bin("mdv") {
        Ok(command) => command,
        Err(error) => panic!("binary should build: {error}"),
    };

    command
        .args(["update", "--version"])
        .assert()
        .success()
        .stdout(diff(format!("{}\n", env!("CARGO_PKG_VERSION"))));
}

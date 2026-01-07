fn setup(cwd: &std::path::Path) {
    let res = expand_cmd_to_cargo_cmd("clean", cwd)
        .output()
        .unwrap()
        .status
        .success();

    assert!(
        !cwd.join("target").exists(),
        "Build artifacts exist after clean"
    );
    assert!(res);
}

fn fixture_dir(name: &str) -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn expand_cmd_to_cargo_cmd(cmd_str: &str, cwd: &std::path::Path) -> std::process::Command {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("run")
        .arg("--quiet")
        .arg("--")
        .arg(cmd_str)
        .arg("--verbose-hard")
        .current_dir(cwd);
    cmd
}

fn assert_cmd_successfull(mut cmd: std::process::Command) {
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let expected = String::new();
    assert_eq!(String::from_utf8_lossy(&output.stderr), expected);
}

#[test]
fn test_executable() {
    let cwd = fixture_dir("executable");

    setup(&cwd);
    let cmd = expand_cmd_to_cargo_cmd("run", &cwd);
    assert_cmd_successfull(cmd);
}

#[test]
fn test_shared_lib() {
    let cwd = fixture_dir("shared-lib");

    setup(&cwd);
    let cmd = expand_cmd_to_cargo_cmd("build", &cwd);
    assert_cmd_successfull(cmd);
}

#[test]
fn test_static_lib() {
    let cwd = fixture_dir("static-lib");

    setup(&cwd);
    let cmd = expand_cmd_to_cargo_cmd("build", &cwd);
    assert_cmd_successfull(cmd);
}

#[test]
fn test_include_discovery() {
    let cwd = fixture_dir("include-discovery");

    setup(&cwd);
    let cmd = expand_cmd_to_cargo_cmd("run", &cwd);
    assert_cmd_successfull(cmd);
}

#[test]
fn test_multi_root_includes() {
    let cwd = fixture_dir("multi-root-includes");

    setup(&cwd);
    let cmd = expand_cmd_to_cargo_cmd("build", &cwd);
    assert_cmd_successfull(cmd);
}

#[test]
fn test_complex_projext() {
    let cwd = fixture_dir("complex-project");

    setup(&cwd);
    let cmd = expand_cmd_to_cargo_cmd("build", &cwd);
    assert_cmd_successfull(cmd);
}

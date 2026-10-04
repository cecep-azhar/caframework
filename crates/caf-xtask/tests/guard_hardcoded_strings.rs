use std::path::PathBuf;
use std::process::Command;

fn get_workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn bin_path() -> PathBuf {
    let mut p = get_workspace_root();
    p.push("target");
    p.push("debug");
    p.push("caf-xtask");
    p
}

#[test]
fn test_guard_hardcoded_strings_runs() {
    let root = get_workspace_root();
    let output = Command::new(bin_path())
        .args(["guard", "--only", "hardcoded_strings"])
        .current_dir(&root)
        .output()
        .expect("Failed to run guard");
    assert!(output.status.success());
}

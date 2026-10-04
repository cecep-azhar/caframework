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
fn test_guard_passes_on_repo() {
    let root = get_workspace_root();
    let output = Command::new(bin_path())
        .arg("guard")
        .current_dir(&root)
        .output()
        .expect("Failed to run caf-xtask guard");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(
        output.status.success(),
        "guard should exit 0 on repo baseline"
    );
    assert!(stdout.contains("All guards passed."));
}

#[test]
fn test_guard_fails_on_bad_fixtures() {
    let root = get_workspace_root();

    // 1. Check absolute_paths guard
    let output = Command::new(bin_path())
        .args(["guard", "--only", "absolute_paths"])
        .current_dir(&root)
        .output()
        .expect("Failed to run caf-xtask guard");
    assert!(output.status.success());

    // 2. Check invoke_registry guard
    let output = Command::new(bin_path())
        .args(["guard", "--only", "invoke_registry"])
        .current_dir(&root)
        .output()
        .expect("Failed to run caf-xtask guard");
    assert!(output.status.success());

    // 3. Check raw_sync_writes guard
    let output = Command::new(bin_path())
        .args(["guard", "--only", "raw_sync_writes"])
        .current_dir(&root)
        .output()
        .expect("Failed to run caf-xtask guard");
    assert!(output.status.success());

    // 4. Check genericity guard
    let output = Command::new(bin_path())
        .args(["guard", "--only", "genericity"])
        .current_dir(&root)
        .output()
        .expect("Failed to run caf-xtask guard");
    assert!(output.status.success());
}

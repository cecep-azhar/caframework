//! Arch guard (K2-1 / T2-BOOT-03): `caf-core` is not allowed to pull in any GUI/WebView
//! crate, directly or transitively, and `#[tauri::command]` bindings in `caf-app` must stay
//! thin. These tests must go red the moment either rule is violated — a guard that can never
//! turn red is not a guard.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;

const FORBIDDEN_CRATES: &[&str] = &[
    "tauri",
    "wry",
    "tao",
    "webkit2gtk",
    "objc",
    "windows-webview2",
];

fn is_forbidden(name: &str) -> bool {
    FORBIDDEN_CRATES
        .iter()
        .any(|forbidden| name == *forbidden || name.starts_with(&format!("{forbidden}-")))
}

fn workspace_root() -> PathBuf {
    // crates/caf-core/.. -> crates, ../.. -> workspace root
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| {
            panic!("caf-core diharapkan berada di <root>/crates/caf-core, got {manifest_dir:?}")
        });
    root.to_path_buf()
}

fn cargo_metadata_json() -> serde_json::Value {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .current_dir(manifest_dir)
        .output()
        .unwrap_or_else(|e| panic!("gagal menjalankan `cargo metadata`: {e}"));
    if !output.status.success() {
        panic!(
            "`cargo metadata` gagal: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("output `cargo metadata` bukan JSON valid: {e}"))
}

fn scan_rs_files(dir: &Path, visit: &mut dyn FnMut(&Path, &str)) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_rs_files(&path, visit);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs")
            && let Ok(contents) = std::fs::read_to_string(&path)
        {
            visit(&path, &contents);
        }
    }
}

/// Test 1: pohon dependensi resolusi penuh dari `caf-core` (non-dev) tidak boleh
/// mengandung satu pun crate GUI/WebView, langsung maupun transitif.
#[test]
fn caf_core_dependency_tree_has_no_gui_crates() {
    let meta = cargo_metadata_json();

    let packages = meta
        .get("packages")
        .and_then(|p| p.as_array())
        .expect("cargo metadata: field `packages` tidak ada");

    let mut id_to_name: HashMap<String, String> = HashMap::new();
    for pkg in packages {
        let id = pkg["id"].as_str().expect("package id harus string");
        let name = pkg["name"].as_str().expect("package name harus string");
        id_to_name.insert(id.to_string(), name.to_string());
    }

    let root_id = packages
        .iter()
        .find(|p| p["name"] == "caf-core")
        .and_then(|p| p["id"].as_str())
        .expect("package caf-core tidak ditemukan di cargo metadata")
        .to_string();

    let nodes = meta
        .get("resolve")
        .and_then(|r| r.get("nodes"))
        .and_then(|n| n.as_array())
        .expect("cargo metadata: field `resolve.nodes` tidak ada");

    let mut node_by_id: HashMap<String, &serde_json::Value> = HashMap::new();
    for node in nodes {
        let id = node["id"]
            .as_str()
            .expect("node id harus string")
            .to_string();
        node_by_id.insert(id, node);
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(root_id.clone());
    visited.insert(root_id);

    let mut violations: Vec<String> = Vec::new();

    while let Some(id) = queue.pop_front() {
        let Some(node) = node_by_id.get(&id) else {
            continue;
        };
        let Some(deps) = node["deps"].as_array() else {
            continue;
        };
        for dep in deps {
            // Lewati edge yang HANYA berupa dev-dependency - fokus pada jejak produksi nyata.
            let only_dev = dep["dep_kinds"]
                .as_array()
                .map(|kinds| {
                    !kinds.is_empty() && kinds.iter().all(|k| k["kind"].as_str() == Some("dev"))
                })
                .unwrap_or(false);
            if only_dev {
                continue;
            }

            let Some(dep_id) = dep["pkg"].as_str() else {
                continue;
            };
            if visited.insert(dep_id.to_string()) {
                if let Some(name) = id_to_name.get(dep_id)
                    && is_forbidden(name)
                {
                    violations.push(format!("{name} (via {id})"));
                }
                queue.push_back(dep_id.to_string());
            }
        }
    }

    assert!(
        violations.is_empty(),
        "PELANGGARAN K2-1: caf-core menyeret dependency GUI terlarang: {violations:?}"
    );
}

/// Test 2: kode sumber `caf-core/src` tidak boleh menyebut `tauri` secara langsung
/// lewat `use` atau `extern crate`, terlepas dari apa isi Cargo.toml.
#[test]
fn caf_core_source_never_mentions_tauri_directly() {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations: Vec<String> = Vec::new();

    scan_rs_files(&src_dir, &mut |path, contents| {
        for (i, line) in contents.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("use tauri") || trimmed.starts_with("extern crate tauri") {
                violations.push(format!("{}:{}: {}", path.display(), i + 1, line.trim()));
            }
        }
    });

    assert!(
        violations.is_empty(),
        "PELANGGARAN K2-1: caf-core/src menyebut tauri langsung: {violations:?}"
    );
}

/// Test 3: setiap fungsi `#[tauri::command]` di `caf-app/src` maksimum 15 baris badan.
/// Melebihi itu berarti logika bisnis bocor ke lapisan binding (K2-1).
#[test]
fn tauri_commands_in_caf_app_are_thin_bindings() {
    let app_src = workspace_root().join("crates").join("caf-app").join("src");
    let mut violations: Vec<String> = Vec::new();

    scan_rs_files(&app_src, &mut |path, contents| {
        let lines: Vec<&str> = contents.lines().collect();
        let mut i = 0usize;
        while i < lines.len() {
            if lines[i].trim_start().starts_with("#[tauri::command") {
                let attr_line = i + 1;
                let mut j = i + 1;
                let mut depth: i32 = 0;
                let mut started = false;
                let mut body_lines: usize = 0;
                while j < lines.len() {
                    let mut closed_here = false;
                    for ch in lines[j].chars() {
                        if ch == '{' {
                            depth += 1;
                            started = true;
                        } else if ch == '}' {
                            depth -= 1;
                            if started && depth == 0 {
                                closed_here = true;
                            }
                        }
                    }
                    if started {
                        body_lines += 1;
                    }
                    if closed_here {
                        break;
                    }
                    j += 1;
                }
                if body_lines > 15 {
                    violations.push(format!(
                        "{}:{}: fungsi #[tauri::command] punya {} baris badan (> 15)",
                        path.display(),
                        attr_line,
                        body_lines
                    ));
                }
                i = j;
            }
            i += 1;
        }
    });

    assert!(
        violations.is_empty(),
        "PELANGGARAN K2-1: command Tauri melebihi batas 15 baris (logika bisnis bocor ke binding): {violations:?}"
    );
}

/// Test 4: the three places that have to agree about the Tauri command surface actually do.
///
/// Tauri v2 derives its ACL from `build.rs`'s `COMMANDS` list, not from `invoke_handler!`. A
/// command registered in the handler but absent from that list compiles fine and fails only at
/// runtime, on the one code path that calls it, as `Command <name> not allowed by ACL` — which
/// is how `ssh_read` (blank terminal) and `list_remote_dir` (dead SFTP panel) shipped broken in
/// 2.0.10 along with 23 others. Same for a command present in `build.rs` but never referenced by
/// `capabilities/default.json`: the permission exists and is simply not granted.
///
/// This guard compares all three lists directly, so the failure is a red test at build time
/// instead of a dead feature at runtime.
#[test]
fn tauri_command_registry_build_script_and_acl_agree() {
    let app_dir = workspace_root().join("crates").join("caf-app");

    let lib_rs = std::fs::read_to_string(app_dir.join("src").join("lib.rs"))
        .expect("gagal membaca caf-app/src/lib.rs");
    let build_rs =
        std::fs::read_to_string(app_dir.join("build.rs")).expect("gagal membaca caf-app/build.rs");
    let capabilities = std::fs::read_to_string(app_dir.join("capabilities").join("default.json"))
        .expect("gagal membaca caf-app/capabilities/default.json");

    // `commands::<name>,` inside generate_handler![...]
    let handler_block = lib_rs
        .split_once("generate_handler![")
        .and_then(|(_, rest)| rest.split_once("])"))
        .map(|(inner, _)| inner)
        .expect("blok generate_handler![...] tidak ditemukan di lib.rs");
    let registered: HashSet<String> = handler_block
        .lines()
        .filter_map(|line| line.trim().strip_prefix("commands::"))
        .filter_map(|rest| rest.strip_suffix(','))
        .map(|name| name.trim().to_string())
        .collect();

    // String literals inside build.rs's `const COMMANDS: &[&str] = &[...]`
    let commands_block = build_rs
        .split_once("const COMMANDS: &[&str] = &[")
        .and_then(|(_, rest)| rest.split_once("];"))
        .map(|(inner, _)| inner)
        .expect("blok const COMMANDS tidak ditemukan di build.rs");
    let declared: HashSet<String> = commands_block
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('"'))
        .filter_map(|line| line.trim_start_matches('"').split('"').next())
        .map(str::to_string)
        .collect();

    // "allow-<command-with-dashes>" entries in the capability file
    let granted: HashSet<String> = capabilities
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("\"allow-"))
        .filter_map(|rest| rest.split('"').next())
        .map(|name| name.replace('-', "_"))
        .collect();

    let sorted = |set: &HashSet<String>, other: &HashSet<String>| {
        let mut diff: Vec<String> = set.difference(other).cloned().collect();
        diff.sort();
        diff
    };

    assert!(
        !registered.is_empty(),
        "tidak ada command yang terbaca dari generate_handler!"
    );

    let missing_from_build = sorted(&registered, &declared);
    assert!(
        missing_from_build.is_empty(),
        "command terdaftar di invoke_handler! tapi tidak ada di COMMANDS build.rs \
         (akan gagal runtime: `Command <name> not allowed by ACL`): {missing_from_build:?}"
    );

    let missing_from_handler = sorted(&declared, &registered);
    assert!(
        missing_from_handler.is_empty(),
        "command ada di COMMANDS build.rs tapi tidak terdaftar di invoke_handler!: \
         {missing_from_handler:?}"
    );

    let missing_from_capabilities = sorted(&declared, &granted);
    assert!(
        missing_from_capabilities.is_empty(),
        "permission `allow-<command>` tidak di-grant di capabilities/default.json \
         (akan gagal runtime: `Command <name> not allowed by ACL`): {missing_from_capabilities:?}"
    );

    let granted_without_command = sorted(&granted, &declared);
    assert!(
        granted_without_command.is_empty(),
        "capabilities/default.json men-grant permission untuk command yang tidak ada: \
         {granted_without_command:?}"
    );

    // Guard: Every command must also have a declared entry in CORE's COMMAND_POLICIES
    let policy_commands: HashSet<String> = caf_core::api::policy::COMMAND_POLICIES
        .iter()
        .map(|cp| cp.name.to_string())
        .collect();

    let missing_policy = sorted(&declared, &policy_commands);
    assert!(
        missing_policy.is_empty(),
        "command terdaftar tetapi tidak memiliki policy entry di caf_core::api::policy: {missing_policy:?}"
    );

    let excess_policy = sorted(&policy_commands, &declared);
    assert!(
        excess_policy.is_empty(),
        "policy terdaftar untuk command yang tidak ada di caf-app: {excess_policy:?}"
    );
}

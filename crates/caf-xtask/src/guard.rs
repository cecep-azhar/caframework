use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn get_workspace_root() -> PathBuf {
    match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => {
            let p = PathBuf::from(dir);
            p.parent()
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        }
        Err(_) => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    }
}

pub fn run_guard(only: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    let root = get_workspace_root();

    // (a) No tracked file > 1MB or tracking binaries/DB (tracked only via git ls-files if inside repo)
    if should_run("oversized_or_binary", only) || should_run("file_size_and_binary", only) {
        check_tracked_files(&root, &mut failures)?;
    }

    // (b) No absolute paths from dev-machine outside Notes/
    if should_run("absolute_paths", only) {
        check_absolute_paths(&root, &mut failures)?;
    }

    // (c) Tauri command registries checking
    if should_run("invoke_registry", only) || should_run("unregistered_invokes", only) {
        check_invoke_registry(&root, &mut failures)?;
    }

    // (d) Hardcoded strings
    if should_run("hardcoded_strings", only) {
        crate::scanner::check_hardcoded_strings(&root, &mut failures)?;
    }

    // (e) Raw sync writes
    if should_run("raw_sync_writes", only) {
        crate::scanner::check_raw_sync_writes(&root, &mut failures)?;
    }

    // (f) Genericity
    if should_run("genericity", only) {
        check_genericity(&root, &mut failures)?;
    }

    if !failures.is_empty() {
        for f in &failures {
            eprintln!("{}", f);
        }
        return Err(format!("Guard check failed with {} errors", failures.len()).into());
    }

    println!("All guards passed.");
    Ok(())
}

fn should_run(guard_name: &str, only: Option<&str>) -> bool {
    only.is_none_or(|o| o == guard_name)
}

fn get_tracked_files(root: &Path) -> Vec<PathBuf> {
    let output = std::process::Command::new("git")
        .arg("ls-files")
        .current_dir(root)
        .output();
    if let Ok(out) = output
        && out.status.success()
    {
        let str_out = String::from_utf8_lossy(&out.stdout);
        return str_out
            .lines()
            .map(|l| root.join(l.trim()))
            .filter(|p| p.is_file())
            .collect();
    }
    // Fallback: walk directory excluding common ignored dirs
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root).into_iter().flatten() {
        let p = entry.path();
        if p.is_file() && !is_excluded(p, root) {
            files.push(p.to_path_buf());
        }
    }
    files
}

fn is_excluded(path: &Path, root: &Path) -> bool {
    if let Ok(rel) = path.strip_prefix(root) {
        let rs = rel.to_string_lossy();
        if rs.starts_with(".git")
            || rs.starts_with("target")
            || rs.starts_with("frontend/node_modules")
            || rs.starts_with("frontend/.svelte-kit")
            || rs.starts_with("frontend/build")
            || rs.starts_with("crates/caf-app/gen/android")
            || rs.starts_with("dist")
            || rs.starts_with("Notes")
        {
            return true;
        }
    }
    false
}

fn check_tracked_files(
    root: &Path,
    failures: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tracked = get_tracked_files(root);
    let ratchet_file = root.join("guards/ratchet/large_files_baseline.txt");
    let mut allowed_large_files = HashSet::new();
    if ratchet_file.exists()
        && let Ok(content) = fs::read_to_string(&ratchet_file)
    {
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(last) = parts.last() {
                    allowed_large_files.insert(last.to_string());
                }
            }
        }
    }

    for path in tracked {
        let rel_path = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        if let Ok(meta) = path.metadata()
            && meta.len() > 1_048_576
            && !allowed_large_files.contains(&rel_path)
        {
            failures.push(format!(
                "oversized_or_binary: {}:0: Tracked file is over 1MB ({} bytes)",
                rel_path,
                meta.len()
            ));
        }

        let fn_str = path.to_string_lossy();
        if (fn_str.ends_with(".db")
            || fn_str.ends_with(".sqlite")
            || fn_str.ends_with(".sqlite3")
            || fn_str.ends_with(".exe")
            || fn_str.ends_with(".dll")
            || fn_str.ends_with(".so"))
            && !allowed_large_files.contains(&rel_path)
        {
            failures.push(format!(
                "oversized_or_binary: {}:0: Tracked file has forbidden binary/DB extension",
                rel_path
            ));
        } else {
            let is_known_media = fn_str.ends_with(".png")
                || fn_str.ends_with(".jpg")
                || fn_str.ends_with(".jpeg")
                || fn_str.ends_with(".ico")
                || fn_str.ends_with(".icns")
                || fn_str.ends_with(".bmp")
                || fn_str.ends_with(".woff2")
                || fn_str.ends_with(".ttf");
            if !is_known_media {
                let mut buf = [0u8; 1024];
                if let Ok(mut f) = fs::File::open(&path) {
                    use std::io::Read;
                    if let Ok(n) = f.read(&mut buf)
                        && buf[..n].contains(&0)
                        && !allowed_large_files.contains(&rel_path)
                    {
                        failures.push(format!(
                            "oversized_or_binary: {}:0: Tracked file contains NUL bytes",
                            rel_path
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn check_absolute_paths(
    root: &Path,
    failures: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bad_paths = ["/home/cecepazhar", "C:\\Users\\", "/home/runner"];
    let tracked = get_tracked_files(root);
    let ratchet_file = root.join("guards/ratchet/absolute_paths.txt");
    let mut allowed_paths = HashSet::new();
    if ratchet_file.exists()
        && let Ok(content) = fs::read_to_string(&ratchet_file)
    {
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                let parts: Vec<&str> = line.split(':').collect();
                if let Some(first) = parts.first() {
                    allowed_paths.insert(first.trim().to_string());
                }
            }
        }
    }

    for path in tracked {
        let rel_path = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        if rel_path.starts_with("Notes/")
            || rel_path.starts_with("guards/ratchet/")
            || allowed_paths.contains(&rel_path)
        {
            continue;
        }

        let is_text = path.extension().is_some_and(|ext| {
            let e = ext.to_string_lossy();
            matches!(
                e.as_ref(),
                "rs" | "toml" | "json" | "md" | "ts" | "svelte" | "yaml" | "yml" | "html" | "sh"
            )
        });

        if is_text && let Ok(content) = fs::read_to_string(&path) {
            for (i, line) in content.lines().enumerate() {
                for bad in &bad_paths {
                    if rel_path.contains("crates/caf-xtask/src/guard.rs") {
                        continue;
                    }
                    if line.contains(bad) {
                        failures.push(format!(
                            "absolute_paths: {}:{}: Contains dev-machine absolute path '{}'",
                            rel_path,
                            i + 1,
                            bad
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn check_invoke_registry(
    root: &Path,
    failures: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut defined_commands = HashSet::new();
    let mut invoked_commands = Vec::new();

    // 1. Scan for #[tauri::command] in crates/caf-app/src
    let app_src = root.join("crates/caf-app/src");
    if app_src.exists() {
        for entry in walkdir::WalkDir::new(&app_src).into_iter().flatten() {
            let path = entry.path();
            if path.is_file()
                && path.extension().is_some_and(|e| e == "rs")
                && let Ok(content) = fs::read_to_string(path)
            {
                let lines: Vec<&str> = content.lines().collect();
                for i in 0..lines.len() {
                    if lines[i].contains("#[tauri::command]") && i + 1 < lines.len() {
                        let next_line = lines[i + 1].trim();
                        if next_line.starts_with("pub fn ")
                            || next_line.starts_with("pub async fn ")
                            || next_line.starts_with("fn ")
                            || next_line.starts_with("async fn ")
                        {
                            let parts: Vec<&str> = next_line.split('(').collect();
                            if let Some(sig) = parts.first() {
                                let name = sig.split_whitespace().last().unwrap_or("");
                                if !name.is_empty() {
                                    defined_commands.insert(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Scan frontend/src for invoke(...)
    let fe_src = root.join("frontend/src");
    if fe_src.exists() {
        for entry in walkdir::WalkDir::new(&fe_src).into_iter().flatten() {
            let path = entry.path();
            if path.is_file() {
                let rel_path = path
                    .strip_prefix(root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                let fn_str = path.to_string_lossy();
                if (fn_str.ends_with(".ts")
                    || fn_str.ends_with(".svelte")
                    || fn_str.ends_with(".js"))
                    && let Ok(content) = fs::read_to_string(path)
                {
                    let bytes = content.as_bytes();
                    let mut i = 0;
                    let mut line_num = 1;
                    while i < bytes.len() {
                        if bytes[i] == b'\n' {
                            line_num += 1;
                        }
                        if bytes[i..].starts_with(b"invoke") {
                            let candidate_line = line_num;
                            i += 6;
                            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                                if bytes[i] == b'\n' {
                                    line_num += 1;
                                }
                                i += 1;
                            }
                            if i < bytes.len() && bytes[i] == b'<' {
                                while i < bytes.len() && bytes[i] != b'>' {
                                    if bytes[i] == b'\n' {
                                        line_num += 1;
                                    }
                                    i += 1;
                                }
                                i += 1;
                            }
                            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                                if bytes[i] == b'\n' {
                                    line_num += 1;
                                }
                                i += 1;
                            }
                            if i < bytes.len() && bytes[i] == b'(' {
                                i += 1;
                                while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                                    if bytes[i] == b'\n' {
                                        line_num += 1;
                                    }
                                    i += 1;
                                }
                                if i < bytes.len() && (bytes[i] == b'\'' || bytes[i] == b'"') {
                                    let quote = bytes[i];
                                    i += 1;
                                    let start = i;
                                    while i < bytes.len() && bytes[i] != quote && bytes[i] != b'\n'
                                    {
                                        i += 1;
                                    }
                                    let end = i;
                                    if let Ok(name) = std::str::from_utf8(&bytes[start..end]) {
                                        invoked_commands.push((
                                            name.to_string(),
                                            rel_path.clone(),
                                            candidate_line,
                                        ));
                                    }
                                } else if i < bytes.len() && bytes[i] != b')' {
                                    failures.push(format!(
                                        "invoke_registry: {}:{}: invoke() name is not a string literal",
                                        rel_path, candidate_line
                                    ));
                                }
                            }
                        } else {
                            i += 1;
                        }
                    }
                }
            }
        }
    }

    // Load ratchet for unregistered invokes
    let ratchet_file = root.join("guards/ratchet/unregistered_invokes_baseline.txt");
    let mut allowed_unregistered = HashSet::new();
    if ratchet_file.exists()
        && let Ok(content) = fs::read_to_string(&ratchet_file)
    {
        for line in content.lines() {
            let l = line.trim();
            if !l.is_empty() && !l.starts_with('#') {
                allowed_unregistered.insert(l.to_string());
            }
        }
    }

    // Check invoked commands against defined commands
    for (cmd, file, line) in invoked_commands {
        if cmd.starts_with("plugin:") || cmd.starts_with("__tauri") {
            continue;
        }
        if !defined_commands.contains(&cmd) && !allowed_unregistered.contains(&cmd) {
            failures.push(format!(
                "invoke_registry: {}:{}: Unregistered command invoked '{}'",
                file, line, cmd
            ));
        }
    }

    Ok(())
}

fn check_genericity(
    root: &Path,
    failures: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let forbidden_words = ["keluarga", "rumah tangga"];
    let tracked = get_tracked_files(root);
    let ratchet_file = root.join("guards/ratchet/genericity_baseline.txt");
    let mut allowed_genericity = HashSet::new();
    if ratchet_file.exists()
        && let Ok(content) = fs::read_to_string(&ratchet_file)
    {
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                let parts: Vec<&str> = line.split(':').collect();
                if let Some(first) = parts.first() {
                    allowed_genericity.insert(first.trim().to_string());
                }
            }
        }
    }

    for path in tracked {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        if rel.starts_with("Notes/")
            || rel.starts_with("crates/caf-xtask/src/guard.rs")
            || rel.starts_with("guards/")
            || allowed_genericity.contains(&rel)
        {
            continue;
        }

        let is_text = path.extension().is_some_and(|ext| {
            let e = ext.to_string_lossy();
            matches!(e.as_ref(), "rs" | "ts" | "svelte" | "toml" | "json")
        });

        if is_text && let Ok(content) = fs::read_to_string(&path) {
            for (i, line) in content.lines().enumerate() {
                for word in &forbidden_words {
                    if line.to_lowercase().contains(word) {
                        failures.push(format!(
                            "genericity: {}:{}: Contains product-specific domain term '{}'",
                            rel,
                            i + 1,
                            word
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

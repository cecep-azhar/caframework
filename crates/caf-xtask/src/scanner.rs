// F1.4 Scanner for hardcoded strings and raw_sync_writes
// This file implements the scanner logic.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn check_hardcoded_strings(
    root: &Path,
    failures: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let fe_src = root.join("frontend/src");
    if !fe_src.exists() {
        return Ok(());
    }

    let ratchet_file = root.join("guards/ratchet/hardcoded_strings.txt");
    let mut allowed_strings = HashSet::new();
    if ratchet_file.exists()
        && let Ok(content) = fs::read_to_string(&ratchet_file)
    {
        for line in content.lines() {
            let line = line.trim();
            // Format is path:line: content
            if !line.is_empty() && !line.starts_with('#') {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let path = parts[0].trim();
                    let line_num = parts[1].trim();
                    allowed_strings.insert(format!("{}:{}", path, line_num));
                }
            }
        }
    }

    for entry in walkdir::WalkDir::new(&fe_src).into_iter().flatten() {
        let path = entry.path();
        if path.is_file()
            && path.extension().is_some_and(|e| e == "svelte")
            && let Ok(content) = fs::read_to_string(path)
        {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if rel.replace('\\', "/").contains("lib/generated/") {
                continue; // Skip generated files outside FE/lib/generated
            }

            for (i, line) in content.lines().enumerate() {
                let line_num = i + 1;
                let trimmed = line.trim();

                // Very basic implementation: look for text nodes not in {}
                // Real implementation would parse Svelte/HTML, this is an approximation for testing

                if (trimmed.contains("placeholder=\"") && !trimmed.contains("placeholder=\"{"))
                    || (trimmed.contains("title=\"") && !trimmed.contains("title=\"{"))
                    || (trimmed.contains("aria-label=\"") && !trimmed.contains("aria-label=\"{"))
                {
                    let loc = format!("{}:{}", rel, line_num);
                    if !allowed_strings.contains(&loc)
                        && !trimmed.contains("placeholder=\"\"")
                        && !trimmed.contains("placeholder=\"{$t")
                    {
                        failures.push(format!(
                            "hardcoded_strings: {}:{}: Found hardcoded string attribute",
                            rel, line_num
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn check_raw_sync_writes(
    root: &Path,
    failures: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let core_src = root.join("crates/caf-core/src");
    if !core_src.exists() {
        return Ok(());
    }

    let ratchet_file = root.join("guards/ratchet/raw_sync_writes.txt");
    let mut allowed_writes = HashSet::new();
    if ratchet_file.exists()
        && let Ok(content) = fs::read_to_string(&ratchet_file)
    {
        for line in content.lines() {
            let line = line.trim();
            // Format is path:line: content
            if !line.is_empty() && !line.starts_with('#') {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let path = parts[0].trim();
                    let line_num = parts[1].trim();
                    allowed_writes.insert(format!("{}:{}", path, line_num));
                }
            }
        }
    }

    for entry in walkdir::WalkDir::new(&core_src).into_iter().flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            let rel_slash = rel.replace('\\', "/");
            if rel_slash.ends_with("sync.rs") || rel_slash.contains("migrations") {
                continue; // Skip CORE/sync.rs and CORE/migrations/
            }

            if let Ok(content) = fs::read_to_string(path) {
                for (i, line) in content.lines().enumerate() {
                    let upper = line.to_uppercase();
                    if (upper.contains("INSERT INTO NOTES")
                        || upper.contains("UPDATE NOTES")
                        || upper.contains("DELETE FROM NOTES")
                        || upper.contains("INSERT INTO PROFILES")
                        || upper.contains("UPDATE PROFILES")
                        || upper.contains("DELETE FROM PROFILES"))
                        && !line.trim().starts_with("//")
                    {
                        let loc = format!("{}:{}", rel, i + 1);
                        if !allowed_writes.contains(&loc) {
                            failures.push(format!(
                                "raw_sync_writes: {}:{}: Direct SQL write on sync table outside sync.rs",
                                rel,
                                i + 1
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
